use vais_tutorial::runner::TutorialRunner;

fn main() {
    match TutorialRunner::new() {
        Ok(mut runner) => {
            if let Err(e) = runner.run() {
                eprintln!("Tutorial error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize tutorial: {}", e);
            std::process::exit(1);
        }
    }
}
