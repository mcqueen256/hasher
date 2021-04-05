use std::sync::Arc;
use std::sync::Mutex;
use crate::app::Application;
use crate::app::ThreadState;
use crate::app::MiningThread;

pub fn begin(app: Arc<Mutex<Application<'static>>>) -> std::thread::JoinHandle<()> {
    let maintaince_thread = std::thread::spawn(move || {
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
                for id in active_thread_count..expected_thread_count {
                    let miner = create_mining_thread(id, Arc::clone(&app));
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
        }
    });
    maintaince_thread
}

fn create_mining_thread(thread_id: usize, app: Arc<Mutex<Application>>) -> MiningThread {
    let state_for_return = Arc::new(Mutex::new(ThreadState::NotStated));
    let current_job_for_return = Arc::new(Mutex::new(None));
    let current_job = Arc::clone(&current_job_for_return);
    let state = Arc::clone(&state_for_return);
    let handle = std::thread::spawn(move || loop {
        // Check if the thread needs to shutdown.
        {
            // let app = app.lock().expect("Could not lock application.");
            // let thread_count = (*app).threads.len();

            // if thread_id >= thread_count {
            //     break;
            // }
            
            // if (*app).quitting {
            //     break;
            // }
            let state = state.lock().unwrap();
            if *state == ThreadState::StopSignal {
                break;
            }
        };

        // Thread id good to run. Do work.
        // TODO: mine
        std::thread::sleep_ms(1000);

    });

    MiningThread {
        current_job: current_job_for_return,
        state: state_for_return,
        handle
    }
}