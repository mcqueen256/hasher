use serde::Deserialize;
use serde::Serialize;

struct PoolStatusPacket {
    total_hash_rate: f64,
    total_shares: usize
}

/// Receive this packet when requesting a job.
/// 
/// The job_size will be the number of counts between the start and the end
/// nounce.
struct JobResponsePacket {
    job_number: u64,
    job_size: u64
    nounce_start: String,
    nounce_end: String
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
    uuid: String,
    student_number: String,
    hashs_per_second: f64,
    nounce_start: String,
    nounce_end: String
    solutions: Vec<Solution>,
}

/// Received from the server on job submission.
struct SubmittionResponsePacket {
    success: bool,
}

/// Send a message informing the cloud the machine is active.
#[derive(Serialize, Deserialize)]
pub struct BootRequest {
    pub uuid: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CommandResponse {
    pub ok: bool,
    pub msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ShutdownRequest {
    pub uuid: String,
}