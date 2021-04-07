pub mod packets;
// use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::application::App;

use self::packets::{Job, JobResponsePacket, SubmittionPacket, SubmittionResponsePacket, PoolStatusRequestPacket, PoolStatusResponsePacket};

const SERVER_URL: &str = "http://127.0.0.1:8080";

fn api<T, U>(mut app: App, uri: &str, packet: T) -> Result<U, ()>
where T: Serialize, U: DeserializeOwned
{
    let response = reqwest::blocking::Client::new()
        .post(&format!("{}{}", SERVER_URL, uri))
        .json(&packet)
        .send();

    if let Ok(response) = response {
        let status_code = response.status();
        if let Ok(response_packet) = response.json::<U>() {
            Ok(response_packet)
        } else {
            app.lock( |app| {
                app.log.error(
                    &format!(
                        "Response decode error @ \"{}\": invalid response. {}",
                        uri,
                        status_code
                    )
                );
            });
            Err(())
        }
    } else {
        app.lock( |app| {
            app.log.error("Network error: Could not register with the server.");
        });
        Err(())
    }
}

pub fn register_with_the_server(mut app: App) {
    let packet = app.lock( |app| packets::BootRequest {
        name: app.name.clone(),
        student_number: app.student_number.clone(),
    });
    let response = api::<_, packets::CommandResponse>(App::clone(&app), "/boot", packet);
    if let Ok(_) = response {
        app.lock(|app| app.log.info("Established connection with the server."));
    }
}

pub fn deregister_with_the_server(mut app: App) {
    let packet = app.lock( |app| packets::ShutdownRequest {
        name: app.name.clone(),
        student_number: app.student_number.clone(),
    });

    let _ = api::<_, packets::CommandResponse>(app, "/shutdown", packet);
}

pub fn request_job(mut app: App) -> Result<Job, ()> {
    // Build request data
    let packet = app.lock( |app| packets::JobRequestPacket {
        student_number: app.student_number.clone(),
        name: app.name.clone(),
    });

    let response = api::<_, packets::JobResponsePacket>(App::clone(&app), "/job/request", packet);
    if let Ok(job_packet) = response {
        match job_packet {
            JobResponsePacket::Success(job) => {
                Ok(job)
            }
            JobResponsePacket::Error(message) => {
                let error_message = format!( "Server error: {}", message);
                app.lock( |app| app.log.error(&error_message));
                Err(())
            }
        }
    } else {
        Err(())
    }
}

pub fn submit_job(app: App, packet: SubmittionPacket) -> Result<(), ()> {

    let response = api::<_, SubmittionResponsePacket>(App::clone(&app), "/job/submit", packet);
    if let Ok(_) = response {
        Ok(())
    } else {
        Err(())
    }
}

pub fn pool_status(mut app: App) -> Result<PoolStatusResponsePacket, ()> {
    let packet = {
        let student_number = app.lock(|app| app.student_number.clone());
        PoolStatusRequestPacket{
            student_number,
        }
    };
    
    api::<_, PoolStatusResponsePacket>(App::clone(&app), "/status", packet)
}