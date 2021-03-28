use chrono::{DateTime, Utc};
use sha256::digest;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

const JOB_SIZE: u128 = 1_000_000_000;
const SERVER_URL: &str = "ec2-3-104-2-89.ap-southeast-2.compute.amazonaws.com";

fn record_start() {
    let mut file_out = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open("start.txt")
        .unwrap();

    let now: DateTime<Utc> = Utc::now();
    if let Err(e) = writeln!(file_out, "start: {}", now) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn main() {
    let arg_student_number = std::env::args()
        .nth(1)
        .expect("invalid student number\nusage: ./hasher <student number> <threads>\n");
    let arg_thread_count = std::env::args()
        .nth(2)
        .expect("invalid thread count\nusage: ./hasher <student number> <threads>\n");
    let arg_thread_count = arg_thread_count
        .parse::<usize>()
        .expect("thread count must be a number\nusage: ./hasher <student number> <threads>\n");

    record_start();

    let mut jobs = Vec::new();
    for _ in 0..arg_thread_count {
        let student_number = arg_student_number.clone();
        jobs.push(thread::spawn(move || loop {
            let (start, end) = {
                let mut job_n_factor = job_n_local.lock().unwrap();
                println!("job {}", job_n_factor);
                let start = *job_n_factor * JOB_SIZE;
                let end = start + JOB_SIZE;
                *job_n_factor += 1;
                (start, end)
            };
            for n in start..end {
                let n_string = hex_string(n);
                let hash = digest(student_number + &n_string);
                let nounce = count_nounce(&hash);
                // Disregard results less than 7.
                if nounce <= 6 {
                    continue;
                }

                {
                    let _lock = local_file_lock.lock().unwrap();
                    let mut file_out = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&format!("results_{}.txt", nounce))
                        .unwrap();
                    let now: DateTime<Utc> = Utc::now();

                    println!("time: {} hash: {} n: {}", now, hash, n_string);

                    if let Err(e) =
                        writeln!(file_out, "time: {} hash: {} n: {}", now, hash, n_string)
                    {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }
            }
        }));
    }
    jobs.into_iter()
        .for_each(|thread| thread.join().expect("didn't join."))
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

fn hex_string(n: u128) -> String {
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
