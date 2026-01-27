use vais_tutorial::runner::TutorialRunner;
use std::process;

fn main() {
    match TutorialRunner::new() {
        Ok(mut runner) => {
            if let Err(e) = runner.run() {
                // Normal exit on user quit
                if e.to_string().contains("User quit") {
                    process::exit(0);
                }
                eprintln!("Tutorial error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize tutorial: {}", e);
            process::exit(1);
        }
    }
}
