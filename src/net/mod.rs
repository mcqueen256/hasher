mod packets;

use crate::application::App;

use self::packets::{Job, JobResponsePacket};

const SERVER_URL: &str = "http://127.0.0.1:8080";

pub fn register_with_the_server(mut app: App) {
    let packet = app.lock( |app| packets::BootRequest {
        name: app.name.clone(),
        student_number: app.student_number.clone(),
    });

    let response = reqwest::blocking::Client::new()
        .post(&format!("{}/boot", SERVER_URL))
        .json(&packet)
        .send();

    // Got a response.
    if let Ok(response) = response {
        let status_code = response.status();
        if let Ok(decoded) = response.json::<packets::CommandResponse>() {
            app.lock( |app| {
                app.log.info("Established connection with server.");
            });
        } else {
            app.lock( |app| {
                app.log.error(
                    &format!(
                        "Server error: invalid response. {}",
                        status_code
                    )
                );
            });
        }
    } else {
        app.lock( |app| {
            app.log.error("Network error: Could not register with the server.");
        });
    }
}

pub fn deregister_with_the_server(mut app: App) {
    let packet = app.lock( |app| packets::ShutdownRequest {
        name: app.name.clone(),
        student_number: app.student_number.clone(),
    });

    let response = reqwest::blocking::Client::new()
        .post(&format!("{}/shutdown", SERVER_URL))
        .json(&packet)
        .send();

    // Got a response.
    if let Ok(response) = response {
        let status_code = response.status();
        if let Ok(decoded) = response.json::<packets::CommandResponse>() {
            app.lock( |app| {
                app.log.info("Shutting down...");
            });
        } else {
            app.lock( |app| {
                app.log.error(
                    &format!(
                        "Server error: invalid response. {}",
                        status_code
                    )
                );
            });
        }
    } else {
        app.lock( |app| {
            app.log.error("Network error: Could not register with the server.");
        });
    }
}

pub fn request_job(mut app: App) -> Result<Job, ()> {
    // Build request data
    let packet = app.lock( |app| packets::JobRequestPacket {
        student_number: app.student_number.clone(),
        name: app.name.clone(),
    });

    // Send request.
    let response = reqwest::blocking::Client::new()
        .post(&format!("{}/job/request", SERVER_URL))
        .json(&packet)
        .send();

    // Process response.
    if let Ok(response) = response {
        let status_code = response.status();
        if let Ok(job_packet) = response.json::<packets::JobResponsePacket>() {
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
            app.lock( |app| {
                app.log.error(
                    &format!(
                        "Server error: invalid response. {}",
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