///         ______                     
///         _________        .---"""      """---.              
///         :______.-':      :  .--------------.  :             
///         | ______  |      | :                : |             
///         |:______B:|      | |  Hasher 0.2    | |             
///         |:______B:|      | |                | |             
///         |:______B:|      | |  #########>  | | |             
///         |         |      | |  ##########> | | |             
///         |:_____:  |      | |  ########>   | | |             
///         |    ==   |      | :                : |             
///         |       O |      :  '--------------'  :             
///         |       o |      :'---...______...---'              
///         |       o |-._.-i___/'             \._              
///         |'-.____o_|   '-.   '-...______...-'  `-._          
///         :_________:      `.____________________   `-.___.-. 
///                         .'.eeeeeeeeeeeeeeeeee.'.      :___:
///                       .'.eeeeeeeeeeeeeeeeeeeeee.'.         
///                      :____________________________:

use chrono::{DateTime, Utc};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde::Serialize;
use sha256::digest;
use std::thread;
use std::time::Instant;

const JOB_SIZE: u64 = 100_000_000;
// const SERVER_URL: &str = "http://ec2-3-104-2-89.ap-southeast-2.compute.amazonaws.com:9876";
const SERVER_URL: &str = "127.0.0.1:8080";

#[derive(Serialize, Deserialize)]
struct JobRequest {
    job_n: u64,
}

#[derive(Serialize, Deserialize)]
struct Solution {
    sha256: String,
    nounce: String,
    time: String,
}
#[derive(Serialize, Deserialize)]
struct Submittion {
    job_n: u64,
    uuid: String,
    student_number: String,
    hashs_per_second: f64,
    solutions: Vec<Solution>,
}

fn next_job() -> u64 {
    let job_request = reqwest::blocking::get(&format!("{}/request-job", SERVER_URL));
    let response = job_request.expect("unable to communicate with server");
    let job_request = response.json::<JobRequest>().expect("invalid reponse.");
    job_request.job_n
}

fn submit(submittion: Submittion) -> bool {
    // let body = json!(submittion);
    let response = reqwest::blocking::Client::new()
        .post(&format!("{}/submit-job", SERVER_URL))
        .json(&submittion)
        .send()
        .expect("unable to communicate with server");
    let job_accepted = response.json::<JobAccepted>().expect("invalid reponse");
    return job_accepted.job_accepted;
}

#[derive(Serialize, Deserialize)]
struct JobAccepted {
    job_accepted: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("hasher 0.1");
    let arg_student_number = std::env::args()
        .nth(1)
        .expect("invalid student number\nusage: ./hasher <student number> <threads>\n");
    let arg_thread_count = std::env::args()
        .nth(2)
        .expect("invalid thread count\nusage: ./hasher <student number> <threads>\n");
    let arg_thread_count = arg_thread_count
        .parse::<usize>()
        .expect("thread count must be a number\nusage: ./hasher <student number> <threads>\n");

    let m = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
        .progress_chars("##-");

    let uuid = {
        let now: DateTime<Utc> = Utc::now();
        let time = &format!("{}", now);
        digest(time)
    };

    let mut jobs = Vec::new();
    for thread_number in 0..arg_thread_count {
        let thread_uid = digest(String::clone(&uuid) + &thread_number.to_string());
        let student_number = String::clone(&arg_student_number);
        let pb = m.add(ProgressBar::new(JOB_SIZE));
        pb.set_style(sty.clone());
        jobs.push(thread::spawn(move || loop {
            let job_n_factor = next_job();
            let start = job_n_factor * JOB_SIZE;
            let end = start + JOB_SIZE;
            let mut solutions = vec![];
            let start_time = Instant::now();
            for n in start..end {
                if n % 100000 == 0 {
                    pb.inc(100000);
                    pb.set_message(&format!("Job: {}, sols: {}", job_n_factor, solutions.len()));
                }
                let n_string = hex_string(n);
                let hash = digest(String::clone(&student_number) + &n_string);
                let nounce = count_nounce(&hash);
                // Disregard results less than 8.
                if nounce <= 7 {
                    continue;
                }
                let now: DateTime<Utc> = Utc::now();
                let solution = Solution {
                    nounce: n_string,
                    sha256: hash,
                    time: format!("{}", now),
                };
                solutions.push(solution);
            }
            let duration = start_time.elapsed();
            let hashs_per_second = 1000f64 * JOB_SIZE as f64 / duration.as_millis() as f64;
            let submittion = Submittion {
                job_n: job_n_factor,
                student_number: String::clone(&student_number),
                hashs_per_second,
                uuid: String::clone(&&thread_uid),
                solutions,
            };
            pb.reset();
            submit(submittion);
        }));
    }
    m.join_and_clear().unwrap();
    jobs.into_iter()
        .for_each(|thread| thread.join().expect("didn't join."));

    Ok(())
}

fn count_nounce(hash: &String) -> usize {
    let mut length = 0;
    for c in hash.chars() {
        if c == '0' {
            length += 1;
        } else {
            break;
        }
    }
    length
}

fn hex_string(n: u64) -> String {
    let mut n_string = String::new();
    for &b in n.to_ne_bytes().iter() {
        let lower = b & 0x0F;
        let upper = b >> 4;
        n_string.push(to_char(upper));
        n_string.push(to_char(lower));
    }
    n_string
}

fn to_char(val: u8) -> char {
    match val {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'A',
        11 => 'B',
        12 => 'C',
        13 => 'D',
        14 => 'E',
        15 => 'F',
        _ => panic!("not a valid byte"),
    }
}
