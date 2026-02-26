use crate::{Result, Tutorial, TutorialError, ValidationResult};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::fs;
use std::path::Path;

pub struct TutorialRunner {
    tutorial: Tutorial,
    editor: DefaultEditor,
}

impl TutorialRunner {
    pub fn new() -> RustylineResult<Self> {
        Ok(Self {
            tutorial: Tutorial::new(),
            editor: DefaultEditor::new()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.print_welcome();
        self.tutorial.list_chapters();

        loop {
            match self.read_command() {
                Ok(cmd) => {
                    if let Err(e) = self.execute_command(&cmd) {
                        println!("{} {}", "Error:".red().bold(), e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Use 'quit' to exit");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(e) => {
                    println!("{} {}", "Error:".red().bold(), e);
                    break;
                }
            }
        }

        println!("\n{}", "Thanks for learning Vais!".cyan().bold());
        Ok(())
    }

    fn print_welcome(&self) {
        println!(
            "\n{}",
            "╔═══════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║   Welcome to Vais Interactive Tutorial   ║".cyan().bold()
        );
        println!("{}", "╚═══════════════════════════════════════════╝".cyan());
        println!();
        println!("Type {} to see available commands", "'help'".yellow());
        println!();
    }

    fn read_command(&mut self) -> RustylineResult<String> {
        let prompt = format!("{} ", ">>>".green().bold());
        let line = self.editor.readline(&prompt)?;
        self.editor.add_history_entry(&line)?;
        Ok(line.trim().to_string())
    }

    fn execute_command(&mut self, cmd: &str) -> Result<()> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "help" => self.show_help(),
            "chapters" | "ch" => self.tutorial.list_chapters(),
            "lessons" | "ls" => {
                let chapter_id = if parts.len() > 1 {
                    parts[1].parse().unwrap_or(0)
                } else {
                    self.tutorial.progress.current_chapter
                };
                self.tutorial.list_lessons(chapter_id)?;
            }
            "start" => {
                if parts.len() > 2 {
                    let chapter_id = parts[1].parse().unwrap_or(0);
                    let lesson_idx = parts[2].parse().unwrap_or(0);
                    self.tutorial.goto_lesson(chapter_id, lesson_idx)?;
                }
                self.start_lesson()?;
            }
            "next" | "n" => {
                self.tutorial.advance_lesson();
                self.start_lesson()?;
            }
            "hint" | "h" => self.show_hint(),
            "solution" | "sol" => self.show_solution(),
            "check" | "c" => {
                if parts.len() > 1 {
                    let file_path = parts[1..].join(" ");
                    self.check_file(&file_path)?;
                } else {
                    println!("{}", "Usage: check <file>".yellow());
                }
            }
            "verify" | "v" => {
                if parts.len() > 1 {
                    let code = parts[1..].join(" ");
                    self.verify_code(&code)?;
                } else {
                    println!("{}", "Usage: verify <code>".yellow());
                }
            }
            "progress" | "p" => self.show_progress(),
            "reset" => {
                if parts.len() > 1 && parts[1] == "confirm" {
                    self.reset_progress()?;
                } else {
                    println!("{}", "Use 'reset confirm' to reset all progress".yellow());
                }
            }
            "quit" | "exit" | "q" => {
                return Err(TutorialError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "User quit",
                )))
            }
            _ => {
                println!("{} {}", "Unknown command:".red(), cmd);
                println!("Type 'help' for available commands");
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\n{}", "Available Commands:".cyan().bold());
        println!();

        let commands = [
            ("help", "Show this help message"),
            ("chapters, ch", "List all chapters"),
            ("lessons, ls [chapter]", "List lessons in a chapter"),
            ("start [chapter] [lesson]", "Start a specific lesson"),
            ("next, n", "Move to the next lesson"),
            ("hint, h", "Show a hint for the current lesson"),
            ("solution, sol", "Show the solution"),
            ("check <file>", "Check code from a file"),
            ("verify <code>", "Verify inline code"),
            ("progress, p", "Show learning progress"),
            ("reset confirm", "Reset all progress"),
            ("quit, exit, q", "Exit the tutorial"),
        ];

        for (cmd, desc) in &commands {
            println!("  {:<25} {}", cmd.yellow(), desc.dimmed());
        }
        println!();
    }

    fn start_lesson(&mut self) -> Result<()> {
        let chapter_id = self.tutorial.progress.current_chapter;
        let lesson_idx = self.tutorial.progress.current_lesson;

        let lesson = self.tutorial.get_lesson(chapter_id, lesson_idx).ok_or(
            TutorialError::LessonNotFound(format!("Chapter {} Lesson {}", chapter_id, lesson_idx)),
        )?;

        let chapter = self.tutorial.get_chapter(chapter_id).unwrap();

        println!("\n{}", "═".repeat(60).cyan());
        println!(
            "{} {} - Lesson {}: {}",
            "Chapter".cyan().bold(),
            chapter.id + 1,
            lesson_idx + 1,
            lesson.title.bold()
        );
        println!("{}", "═".repeat(60).cyan());
        println!();
        println!("{}", lesson.description.italic());
        println!();
        println!("{}", lesson.content);
        println!();
        println!("{}", "Task:".yellow().bold());
        println!("{}", lesson.code_template.dimmed());
        println!();
        println!(
            "Type {} to see a hint, {} for the solution",
            "'hint'".yellow(),
            "'solution'".yellow()
        );
        println!();

        Ok(())
    }

    fn show_hint(&mut self) {
        let chapter_id = self.tutorial.progress.current_chapter;
        let lesson_idx = self.tutorial.progress.current_lesson;

        if let Some(lesson) = self.tutorial.get_lesson(chapter_id, lesson_idx) {
            let lesson_id = lesson.id.clone();
            if let Some(hint) = self.tutorial.use_hint(&lesson_id) {
                println!("\n{} {}", "Hint:".yellow().bold(), hint);
            } else {
                println!("\n{}", "No more hints available!".dimmed());
            }
        }
    }

    fn show_solution(&self) {
        let chapter_id = self.tutorial.progress.current_chapter;
        let lesson_idx = self.tutorial.progress.current_lesson;

        if let Some(lesson) = self.tutorial.get_lesson(chapter_id, lesson_idx) {
            println!("\n{}", "Solution:".green().bold());
            println!("{}", "─".repeat(60).dimmed());
            println!("{}", lesson.solution);
            println!("{}", "─".repeat(60).dimmed());
        }
    }

    fn check_file(&mut self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(TutorialError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", file_path),
            )));
        }

        let code = fs::read_to_string(path)?;
        self.verify_code(&code)
    }

    fn verify_code(&mut self, code: &str) -> Result<()> {
        let chapter_id = self.tutorial.progress.current_chapter;
        let lesson_idx = self.tutorial.progress.current_lesson;

        let lesson = self.tutorial.get_lesson(chapter_id, lesson_idx).ok_or(
            TutorialError::LessonNotFound(format!("Chapter {} Lesson {}", chapter_id, lesson_idx)),
        )?;

        println!("\n{}", "Checking your code...".cyan());

        let result = self.tutorial.validate_code(code, lesson)?;
        result.print();

        if result.success {
            let lesson_id = lesson.id.clone();
            self.tutorial.mark_lesson_complete(&lesson_id);

            println!("\n{}", "🎉 Lesson completed!".green().bold());
            println!("Type {} to continue", "'next'".yellow());
        } else {
            println!("\n{}", "Keep trying! Type 'hint' for help.".yellow());
        }

        Ok(())
    }

    fn show_progress(&self) {
        println!("\n{}", "Your Progress:".cyan().bold());
        println!();

        let total_lessons: usize = self
            .tutorial
            .chapters
            .iter()
            .map(|ch| ch.lessons.len())
            .sum();
        let completed = self.tutorial.progress.completed_lessons.len();

        println!(
            "Overall: {}/{} lessons completed ({:.1}%)",
            completed.to_string().green().bold(),
            total_lessons,
            (completed as f64 / total_lessons as f64 * 100.0)
        );
        println!();

        for chapter in &self.tutorial.chapters {
            let chapter_completed = chapter
                .lessons
                .iter()
                .filter(|l| self.tutorial.is_lesson_complete(&l.id))
                .count();

            let percentage = chapter_completed as f64 / chapter.lessons.len() as f64 * 100.0;
            let bar_length = 30;
            let filled = (bar_length as f64 * percentage / 100.0) as usize;
            let bar: String = (0..bar_length)
                .map(|i| if i < filled { '█' } else { '░' })
                .collect();

            println!(
                "{} {} [{}] {:.0}%",
                format!("Chapter {}:", chapter.id + 1).bold(),
                chapter.title,
                bar,
                percentage
            );
        }
        println!();
    }

    fn reset_progress(&mut self) -> Result<()> {
        self.tutorial.progress = Default::default();
        self.tutorial.save_progress()?;
        println!("{}", "Progress reset successfully!".green());
        Ok(())
    }
}

impl Default for TutorialRunner {
    fn default() -> Self {
        Self {
            tutorial: Tutorial::new(),
            editor: DefaultEditor::new().unwrap(),
        }
    }
}

pub struct CodeValidator {
    _lesson_id: String,
}

impl CodeValidator {
    pub fn new(lesson_id: String) -> Self {
        Self {
            _lesson_id: lesson_id,
        }
    }

    pub fn validate(&self, code: &str) -> Result<ValidationResult> {
        let parse_result = vais_parser::parse(code);

        let result = match parse_result {
            Ok(_) => ValidationResult {
                success: true,
                message: "Code compiled successfully".to_string(),
                passed_tests: 1,
                total_tests: 1,
                errors: Vec::new(),
            },
            Err(e) => ValidationResult {
                success: false,
                message: "Parse error".to_string(),
                passed_tests: 0,
                total_tests: 1,
                errors: vec![format!("{:?}", e)],
            },
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_tutorial_runner_creation() {
        let runner = TutorialRunner::new();
        assert!(runner.is_ok());
    }

    #[test]
    fn test_code_validator() {
        let validator = CodeValidator::new("test_lesson".to_string());
        let code = "let x = 42;";
        let result = validator.validate(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_check() {
        let mut temp_file = NamedTempFile::new().unwrap();
        use std::io::Write;
        writeln!(temp_file, "let answer = 42;").unwrap();

        let mut runner = TutorialRunner::new().unwrap();
        runner.tutorial.goto_lesson(0, 0).unwrap();

        let result = runner.check_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_parsing() {
        let _runner = TutorialRunner::new().unwrap();
        let cmd = "start 0 0";
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "start");
    }

    #[test]
    fn test_code_validator_valid_code() {
        let validator = CodeValidator::new("test".to_string());
        let result = validator.validate("F main() -> i64 { 42 }").unwrap();
        assert!(result.success);
        assert_eq!(result.passed_tests, 1);
        assert_eq!(result.total_tests, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_code_validator_invalid_code() {
        let validator = CodeValidator::new("test".to_string());
        let result = validator.validate("{{{invalid!!!").unwrap();
        assert!(!result.success);
        assert_eq!(result.passed_tests, 0);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_code_validator_empty_code() {
        let validator = CodeValidator::new("test".to_string());
        let result = validator.validate("").unwrap();
        // Empty code should parse successfully (empty module)
        assert!(result.success);
    }

    #[test]
    fn test_tutorial_runner_default() {
        let runner = TutorialRunner::default();
        // Default creation should succeed
        assert_eq!(runner.tutorial.progress.current_chapter, 0);
    }

    #[test]
    fn test_command_parsing_single_word() {
        let cmd = "help";
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "help");
    }

    #[test]
    fn test_command_parsing_empty() {
        let cmd = "";
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        assert!(parts.is_empty());
    }

    #[test]
    fn test_check_nonexistent_file() {
        let mut runner = TutorialRunner::new().unwrap();
        runner.tutorial.goto_lesson(0, 0).unwrap();
        let result = runner.check_file("/nonexistent/file.vais");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_code_with_valid_solution() {
        let mut runner = TutorialRunner::new().unwrap();
        runner.tutorial.goto_lesson(0, 0).unwrap();
        let solution = runner.tutorial.get_lesson(0, 0).unwrap().solution.clone();
        let result = runner.verify_code(&solution);
        assert!(result.is_ok());
    }
}
