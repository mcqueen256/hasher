use std::sync::Arc;
use std::sync::Mutex;

use crate::log::Logger;

pub struct Application {
    pub student_number: String,
    pub name: String,
    pub quitting: bool,
    pub threads_cleaned_up: bool,
    pub threads: Vec<MiningThread>,
    pub expected_thread_count: usize,
    pub log: Logger,
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
    ShuttingDown,
}

#[derive(Clone, Copy)]
pub struct CurrentJob {
    pub progress: usize,
    pub size: u64,
    pub job_number: u64,
}

/// The thread will also hold its own state and the current_job.
pub struct MiningThread {
    pub current_job: Arc<Mutex<Option<CurrentJob>>>,
    pub state: Arc<Mutex<ThreadState>>,
    pub handle: std::thread::JoinHandle<()>,
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
        }
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