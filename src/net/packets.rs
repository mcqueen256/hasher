use serde::Deserialize;
use serde::Serialize;

struct PoolStatusPacket {
    total_hash_rate: f64,
    total_shares: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequestPacket {
    pub student_number: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Job {
    pub number: u64,
    pub size: u64,
    pub nounce_start: u64,
    pub nounce_end: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JobResponsePacket {
    Success(Job),
    Error(String),
}

/// Solution info 
struct Solution {
    sha256: String,
    nounce: String,
    time: f64,
}

/// When the job is complete, this packet is sent to the pool.
struct SubmittionPacket {
    job_n: u64,
    name: String,
    student_number: String,
    hashs_per_second: f64,
    nounce_start: String,
    nounce_end: String,
    solutions: Vec<Solution>,
}

/// Received from the server on job submission.
struct SubmittionResponsePacket {
    success: bool,
}

/// Send a message informing the cloud the machine is active.
#[derive(Serialize, Deserialize)]
pub struct BootRequest {
    pub student_number: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandResponse {
    pub ok: bool,
    pub msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ShutdownRequest {
    pub name: String,
    pub student_number: String,
}