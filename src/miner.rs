use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use crate::{application::{App, Application, CurrentJob, HashRateHistory, MiningThread, ThreadState}, net::{
        request_job,
        submit_job,
        packets::{
            Solution,
            SubmittionPacket,
        },
    }};
use radix_fmt::radix;
use std::time::SystemTime;

const MINIMUN_ZERO_BIT_LENGTH: u8 = 26;

pub fn begin(app: Arc<Mutex<Application>>) -> std::thread::JoinHandle<()> {
    let maintaince_thread = std::thread::spawn(move || {
        crate::net::register_with_the_server(App::from(&app));
        loop {
            let expected_thread_count = {
                let app = app.lock().expect("Could not lock application.");
                (*app).expected_thread_count
            };
            let mut active_thread_count = {
                let app = app.lock().expect("Could not lock application.");
                (*app).threads.len()
            };

            // Check thread count matches number of running threads.
            if active_thread_count < expected_thread_count {
                for _id in active_thread_count..expected_thread_count {
                    let miner = create_mining_thread(Arc::clone(&app));
                    { // Lock App
                        let mut app = app.lock().expect("Could not lock application.");
                        (*app).threads.push(miner);
                    } // Unlock App
                }
            } else if active_thread_count > expected_thread_count {
                // Need to remove threads.
                // Threads will know to end if they need to.
                while active_thread_count > expected_thread_count {
                    let miner = { // Lock App
                        let mut app = app.lock().expect("Could not lock application.");
                        let m = (*app).threads.pop().unwrap();
                        active_thread_count = (*app).threads.len();
                        m
                    }; // Unlock App
                    // Signal to the miner to end.
                    {
                        let mut state = miner.state.lock().unwrap();
                        *state = ThreadState::StopSignal;
                    }
                    miner.handle.join().expect("Could not reduce mining thread.");
                }
            }

            // Check if the miners need to end. If so, clean up.
            {
                let mut app = app.lock().expect("Could not lock application.");
                if (*app).quitting {
                    (*app).expected_thread_count = 0;
                    if active_thread_count == 0 {
                        (*app).threads_cleaned_up = true;
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    });
    maintaince_thread
}

fn create_mining_thread(app: Arc<Mutex<Application>>) -> MiningThread {
    let state_for_return = Arc::new(Mutex::new(ThreadState::NotStated));
    let current_job_for_return = Arc::new(Mutex::new(None));
    let hash_rate_history_for_return = Arc::new(Mutex::new(HashRateHistory::new()));
    let current_job = Arc::clone(&current_job_for_return);
    let state = Arc::clone(&state_for_return);
    let hash_rate_history = Arc::clone(&hash_rate_history_for_return);
    let handle = std::thread::spawn(move || loop {
        // Check if the thread needs to shutdown.
        {
            let state = state.lock().unwrap();
            if *state == ThreadState::StopSignal {
                break;
            }
        };

        // Thread id good to run. Do work.
        mining_loop(
            App::from(&app),
            Arc::clone(&current_job),
            Arc::clone(&state),
            Arc::clone(&hash_rate_history),
        );

    });

    MiningThread {
        current_job: current_job_for_return,
        state: state_for_return,
        hash_rate_history: hash_rate_history_for_return,
        handle
    }
}

fn mining_loop(mut app: App, current_job: Arc<Mutex<Option<CurrentJob>>>, state: Arc<Mutex<ThreadState>>, hash_rate_history: Arc<Mutex<HashRateHistory>>) {

    // Fetch job from server
    let job_response = request_job(App::clone(&app));
    let job  = if let Ok(job_response) = job_response {
        let job_number = job_response.number;
        let job_size = job_response.size;
        let mut current_job = current_job.lock().unwrap();
        *current_job = Some(CurrentJob {
            job_number,
            size: job_size,
            progress: 0,
            solutions: 0,
        });
        job_response
    } else {
        // Error reported to log screen by request_job(...).
        app.lock(|app| app.log.error("Cannot aquire job. Waiting 10 seconds..."));
        for _ in 0..1000 {
            let state = state.lock().unwrap();
            if *state == ThreadState::StopSignal {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        return;
    };

    // Work on job.
    let mut buffer: Vec<u8> = vec![]; // To hash.
    // Add student number to buffer.
    let student_number = app.lock(|app| app.student_number.clone());
    student_number.chars().for_each(|c| buffer.push(c as u8));
    // Add Initial nounce to buffer.
    radix(job.nounce_start, 36).to_string().to_uppercase().chars().for_each(|c| buffer.push(c as u8));
    // save work by storing sn len
    let sn_len = student_number.len();
    // Store found solutions in
    let mut solutions = Vec::new();
    // use SHA256
    let mut sh = Sha256::default();
    // store the latest nounce
    let mut latest_nounce = job.nounce_start;
    // Capture batch job start time
    let start_time = get_time();
    // set state
    {
        let mut state = state.lock().unwrap();
        *state = ThreadState::Mining;
    }
    // Compute hashs - Start
    for nounce in job.nounce_start..job.nounce_end {
        // Check if thread must report its status
        if nounce % 10_000 == 0 {
            // Check if the thread needs to stop
            if *state.lock().unwrap() == ThreadState::StopSignal {
                break;
            }
            // Status update
            let mut current_job = current_job.lock().unwrap();
            *current_job = Some(CurrentJob {
                job_number: job.number,
                size: job.size,
                progress: nounce - job.nounce_start,
                solutions: solutions.len(),
            });
                // Calc hashs / sec
            let hashes = latest_nounce - job.nounce_start;
            let duration = get_time() - start_time;
            let hashs_per_second = hashes as f64 / duration;
            if !hashs_per_second.is_nan() {
                (*(hash_rate_history.lock().unwrap())).push_hashrate(hashs_per_second);
            }
        }
        latest_nounce = nounce;
        // calculate hash
        sh.update(&buffer);
        let sha256_buffer = sh.finalize_reset();
        let count = count_leading_zero_bits(&sha256_buffer);
        if count >= MINIMUN_ZERO_BIT_LENGTH {
            // Capture Solution.
            let solution = Solution {
                nounce: buffer[sn_len..].iter().map(|&b| b as char).collect(),
                sha256: sha245_to_string(&sha256_buffer),
                time: get_time(),
            };
            app.lock(|app| app.log.solution(
                &solution.sha256, &solution.nounce, count,
            ));
            solutions.push(solution);
        }
        increment_byte_string(&mut buffer, sn_len);
    } // Compute hashs - End

    // Capture batch job end time
    let end_time = get_time();

    // Calc hashs / sec
    let hashes = latest_nounce - job.nounce_start;
    let duration = end_time - start_time;
    let thread_hashes_per_second = hashes as f64 / duration;
    let total_hashes_per_second = app.lock(|app| app.total_hashrate());

    let submittion = SubmittionPacket {
        job_n: job.number,
        name: app.lock(|app| app.name.clone()),
        student_number: student_number,
        thread_hashes_per_second,
        total_hashes_per_second,
        nounce_start: job.nounce_start,
        nounce_end: latest_nounce + 1,
        solutions,
    };

    // Report at the end.
    let _ = submit_job(app, submittion);
}

fn sha245_to_string(sha256_buffer: &[u8]) -> String {
    let mut result = String::new();
    for byte in sha256_buffer {
        result += &format!("{:02x}", byte);
    }
    result
}


fn get_time() -> f64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs_f64(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}


fn count_leading_zero_bits(buffer: &[u8]) -> u8 {
    let mut leading_zero_bits = 0;
    for byte in buffer {
        match byte {
            0 => {
                leading_zero_bits += 8;
            }
            0b0000_0001 => {
                leading_zero_bits += 7;
                break;
            }
            0b0000_0010 ..= 0b0000_0011 => {
                leading_zero_bits += 6;
                break;
            }
            0b0000_0100 ..= 0b0000_0111 => {
                leading_zero_bits += 5;
                break;
            }
            0b0000_1000 ..= 0b0000_1111 => {
                leading_zero_bits += 4;
                break;
            }
            0b0001_0000 ..= 0b0001_1111 => {
                leading_zero_bits += 3;
                break;
            }
            0b0010_0000 ..= 0b0011_1111 => {
                leading_zero_bits += 2;
                break;
            }
            0b0100_0000 ..= 0b0111_1111 => {
                leading_zero_bits += 1;
                break;
            }
            0b1000_0000 ..= 0b1111_1111 => {
                break;
            }
        }
    }
    leading_zero_bits
}


fn next(ascii_decimal: u8) -> u8 {
    match ascii_decimal {
        48 ..= 56 => ascii_decimal + 1, // '0' to '8'
        57 => 65,                       // '9' -> 'A'
        65 ..= 89 => ascii_decimal + 1, // 'A' to 'Y'
        90 => 48,                       // 'Z' -> '0'
        x => panic!("uh uh ah, you didn't say the magic word! {}", x),
    }
}


fn increment_byte_string(s: &mut Vec<u8>, start_index: usize) {
    for i in start_index..s.len() {
        let c = s[i];
        let n = next(c);
        s[i] = n;
        if n != ('0' as u8) {
            return;
        }
    }
    // If this point is reached the number needs to be grown.
    s.push('1' as u8);
}
