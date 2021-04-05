mod app;
mod miner;
mod ui;
mod com;
mod log;

#[allow(dead_code)]
mod util;

use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use num_cpus;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    student_number: String,
    machine_name: Option<String>,
    thread_count: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {

    let args = Cli::from_args();

    // Check the student number is correct
    if args.student_number.len() != 8 {
        eprintln!("Student number must be 8 numbers.");
        return Ok(());
    }
    for c in args.student_number.chars() {
        if !c.is_digit(10) {
            eprintln!("Student number should only contain numbers.");
            return Ok(());
        }
    }

    // Check miner name.
    let machine_name = if let Some(n) = args.machine_name {
        if n.len() == 0 {
            eprintln!("The machine name must not be empty.");
            return Ok(());
        }
        n
    } else {
        String::from("i-o-restful-authentication-0")
    };

    // Check thread count.
    let thread_count = if let Some(thread_count) = args.thread_count {
        thread_count
    } else {
        num_cpus::get() - 1
    };

    let app = Arc::new(
        Mutex::new(
            app::Application::start(
                args.student_number,
                thread_count,
                machine_name,
            )
        )
    );

    let minter_thread = miner::begin(Arc::clone(&app));
    ui::main_loop(Arc::clone(&app))?;
    minter_thread.join().expect("Could not finish mining threads");
    Ok(())
}
