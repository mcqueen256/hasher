use crate::application::App;
use std::time::Duration;
use std::thread;
use crate::net::pool_status;

pub fn begin(mut app: App) -> std::thread::JoinHandle<()> {

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5000));
        'status_loop : loop {
            // Fetch the pool status
            let pool_status_result= pool_status(App::clone(&app));
            if let Ok(pool_status) = pool_status_result {
                app.lock(|app| {
                    app.pool_status = Some(pool_status);
                });
        }

            // Sleep for 5 seconds
            for _ in 0..50 {
                if app.lock(|app| app.quitting) {
                    break 'status_loop;
                }
                thread::sleep(Duration::from_millis(100));
            }
            
        }
    })
}