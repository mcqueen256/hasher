use std::sync::Arc;
use std::sync::Mutex;

use crate::{log::Logger, net::packets::PoolStatusResponsePacket};

pub struct Application {
    pub student_number: String,
    pub name: String,
    pub quitting: bool,
    pub threads_cleaned_up: bool,
    pub threads: Vec<MiningThread>,
    pub expected_thread_count: usize,
    pub log: Logger,
    pub pool_status: Option<PoolStatusResponsePacket>,
}


/// The thread state represents the lifecycle of the thread.
/// First the thread is not started, this happens when it is created.
/// When the thread moves to its working state, the thread changes to Mining.
/// Externally, the application may change the state to StopSignal to inform
/// the thread to stop.
/// When the thread knows to stop, it is in the ShuttingDown state. When
/// shutting down, it may clean itself up and send data to the server.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ThreadState {
    NotStated,
    Mining,
    StopSignal,
}

#[derive(Clone, Copy)]
pub struct CurrentJob {
    pub progress: u64,
    pub size: u64,
    pub job_number: u64,
    pub solutions: usize,
}

/// The thread will also hold its own state and the current_job.
pub struct MiningThread {
    pub current_job: Arc<Mutex<Option<CurrentJob>>>,
    pub state: Arc<Mutex<ThreadState>>,
    pub hash_rate_history: Arc<Mutex<HashRateHistory>>,
    pub handle: std::thread::JoinHandle<()>,
}

pub struct HashRateHistory (Vec<f64>);

impl HashRateHistory {
    pub fn new() -> Self {
        HashRateHistory (vec![])
    }

    pub fn get_hashrate(&self) -> f64 {
        let mut sum = 0.0;
        for hashrate in self.0.iter() {
            sum += hashrate;
        }
        // Avoid divide by zero.
        if self.0.len() == 0 {
            return 0.0;
        }
        sum / self.0.len() as f64
    }

    pub fn push_hashrate(&mut self, hashrate: f64) {
        if hashrate.is_nan() {
            return;
        }
        if self.0.len() > 100 {
            self.0.remove(0);
        }
        self.0.push(hashrate);
    }
}

impl Application {

    pub fn start(student_number: String, thread_count: usize, name: String) -> Self {
        Self {
            name,
            student_number,
            quitting: false,
            threads_cleaned_up: false,
            threads: vec![],
            expected_thread_count: thread_count,
            log: Logger::new(),
            pool_status: None,
        }
    }

    pub fn total_hashrate(&self) -> f64 {
        // Protect against div by zero
        if self.threads.len() == 0 {
            return  0.0;
        }
        let mut sum = 0.0;
        for thread in self.threads.iter() {
            let hash_rate_history = thread.hash_rate_history.lock().unwrap();
            let thread_hashrate = hash_rate_history.get_hashrate();
            if !thread_hashrate.is_nan() {
                sum +=  thread_hashrate;
            }
        }
        sum / self.threads.len() as f64
    }
}

pub struct App(pub Arc<Mutex<Application>>);

impl App {
    pub fn from(arc: &Arc<Mutex<Application>>) -> Self {
        App (Arc::clone(arc))
    }

    pub fn lock<T>(&mut self, callback: impl Fn(&mut Application) -> T) -> T {
        let mut app = self.0.lock().unwrap();
        callback(&mut (*app))
    }

    pub fn clone(app: &Self) -> Self {
        App (app.0.clone())
    }
}