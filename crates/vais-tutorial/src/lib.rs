use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod lessons;
pub mod runner;

#[derive(Error, Debug)]
pub enum TutorialError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lesson not found: {0}")]
    LessonNotFound(String),
    #[error("Chapter not found: {0}")]
    ChapterNotFound(usize),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TutorialError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub code_template: String,
    pub solution: String,
    pub test_cases: Vec<TestCase>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub description: String,
    pub expected_output: Option<String>,
    pub should_compile: bool,
    pub validation_fn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
pub struct Progress {
    pub completed_lessons: HashMap<String, bool>,
    pub current_chapter: usize,
    pub current_lesson: usize,
    pub hints_used: HashMap<String, usize>,
}


pub struct Tutorial {
    pub chapters: Vec<Chapter>,
    pub progress: Progress,
    progress_file: PathBuf,
}

impl Tutorial {
    pub fn new() -> Self {
        let chapters = lessons::create_chapters();
        let progress_file = Self::default_progress_file();
        let progress = Self::load_progress(&progress_file).unwrap_or_default();

        Self {
            chapters,
            progress,
            progress_file,
        }
    }

    pub fn with_progress_file<P: AsRef<Path>>(path: P) -> Self {
        let chapters = lessons::create_chapters();
        let progress_file = path.as_ref().to_path_buf();
        let progress = Self::load_progress(&progress_file).unwrap_or_default();

        Self {
            chapters,
            progress,
            progress_file,
        }
    }

    fn default_progress_file() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".vais_tutorial_progress.json")
    }

    fn load_progress(path: &Path) -> Result<Progress> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(Progress::default())
        }
    }

    pub fn save_progress(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.progress)?;
        fs::write(&self.progress_file, json)?;
        Ok(())
    }

    pub fn get_chapter(&self, id: usize) -> Option<&Chapter> {
        self.chapters.get(id)
    }

    pub fn get_lesson(&self, chapter_id: usize, lesson_idx: usize) -> Option<&Lesson> {
        self.chapters
            .get(chapter_id)
            .and_then(|ch| ch.lessons.get(lesson_idx))
    }

    pub fn mark_lesson_complete(&mut self, lesson_id: &str) {
        self.progress.completed_lessons.insert(lesson_id.to_string(), true);
        let _ = self.save_progress();
    }

    pub fn is_lesson_complete(&self, lesson_id: &str) -> bool {
        self.progress.completed_lessons.get(lesson_id).copied().unwrap_or(false)
    }

    pub fn use_hint(&mut self, lesson_id: &str) -> Option<String> {
        let chapter_id = self.progress.current_chapter;
        let lesson_idx = self.progress.current_lesson;

        // Get lesson info first to avoid borrowing issues
        let (hint_count, lesson_matches, hint_idx) = if let Some(lesson) = self.get_lesson(chapter_id, lesson_idx) {
            let current_hints = *self.progress.hints_used.get(lesson_id).unwrap_or(&0);
            (lesson.hints.len(), lesson.id == lesson_id, current_hints)
        } else {
            return None;
        };

        if lesson_matches && hint_idx < hint_count {
            // Get the hint text
            let hint = self.get_lesson(chapter_id, lesson_idx)?.hints[hint_idx].clone();

            // Update hint count
            *self.progress.hints_used.entry(lesson_id.to_string()).or_insert(0) += 1;
            let _ = self.save_progress();

            return Some(hint);
        }
        None
    }

    pub fn list_chapters(&self) {
        println!("\n{}", "Available Chapters:".bold().cyan());
        for chapter in &self.chapters {
            let completed = chapter.lessons.iter()
                .filter(|l| self.is_lesson_complete(&l.id))
                .count();
            let total = chapter.lessons.len();

            println!(
                "  {}. {} [{}/{}]",
                chapter.id,
                chapter.title.bold(),
                completed,
                total
            );
            println!("     {}", chapter.description.dimmed());
        }
        println!();
    }

    pub fn list_lessons(&self, chapter_id: usize) -> Result<()> {
        let chapter = self.get_chapter(chapter_id)
            .ok_or(TutorialError::ChapterNotFound(chapter_id))?;

        println!("\n{} {}", "Chapter:".bold().cyan(), chapter.title.bold());
        println!("{}\n", chapter.description.dimmed());

        for (idx, lesson) in chapter.lessons.iter().enumerate() {
            let status = if self.is_lesson_complete(&lesson.id) {
                "✓".green()
            } else {
                "○".yellow()
            };

            println!("  {} {}. {}", status, idx + 1, lesson.title.bold());
            println!("     {}", lesson.description.dimmed());
        }
        println!();
        Ok(())
    }

    pub fn validate_code(&self, code: &str, lesson: &Lesson) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            success: false,
            message: String::new(),
            passed_tests: 0,
            total_tests: lesson.test_cases.len(),
            errors: Vec::new(),
        };

        // Parse the code using vais_parser
        let parse_result = vais_parser::parse(code);

        match parse_result {
            Ok(_module) => {
                // Check if it compiles
                let should_compile = lesson.test_cases.iter().all(|tc| tc.should_compile);
                if !should_compile {
                    result.errors.push("Code should not compile".to_string());
                    return Ok(result);
                }

                // Run test cases
                for test_case in &lesson.test_cases {
                    if test_case.should_compile {
                        result.passed_tests += 1;
                    }
                }

                result.success = result.passed_tests == result.total_tests;
                if result.success {
                    result.message = "All tests passed!".to_string();
                } else {
                    result.message = format!(
                        "Passed {}/{} tests",
                        result.passed_tests,
                        result.total_tests
                    );
                }
            }
            Err(e) => {
                result.errors.push(format!("Parse error: {:?}", e));
                result.message = "Code failed to parse".to_string();
            }
        }

        Ok(result)
    }

    pub fn next_lesson(&self) -> Option<(usize, usize)> {
        let chapter_id = self.progress.current_chapter;
        let lesson_idx = self.progress.current_lesson;

        if let Some(chapter) = self.get_chapter(chapter_id) {
            if lesson_idx < chapter.lessons.len() {
                return Some((chapter_id, lesson_idx));
            } else if self.get_chapter(chapter_id + 1).is_some() {
                return Some((chapter_id + 1, 0));
            }
        }
        None
    }

    pub fn advance_lesson(&mut self) {
        if let Some(chapter) = self.get_chapter(self.progress.current_chapter) {
            if self.progress.current_lesson + 1 < chapter.lessons.len() {
                self.progress.current_lesson += 1;
            } else if self.progress.current_chapter + 1 < self.chapters.len() {
                self.progress.current_chapter += 1;
                self.progress.current_lesson = 0;
            }
            let _ = self.save_progress();
        }
    }

    pub fn goto_lesson(&mut self, chapter_id: usize, lesson_idx: usize) -> Result<()> {
        if chapter_id >= self.chapters.len() {
            return Err(TutorialError::ChapterNotFound(chapter_id));
        }

        let chapter = &self.chapters[chapter_id];
        if lesson_idx >= chapter.lessons.len() {
            return Err(TutorialError::LessonNotFound(format!(
                "Lesson {} in chapter {}",
                lesson_idx, chapter_id
            )));
        }

        self.progress.current_chapter = chapter_id;
        self.progress.current_lesson = lesson_idx;
        self.save_progress()?;
        Ok(())
    }
}

impl Default for Tutorial {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub success: bool,
    pub message: String,
    pub passed_tests: usize,
    pub total_tests: usize,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn print(&self) {
        if self.success {
            println!("\n{} {}", "✓".green().bold(), self.message.green());
        } else {
            println!("\n{} {}", "✗".red().bold(), self.message.yellow());
            for error in &self.errors {
                println!("  {}", error.red());
            }
        }

        println!(
            "Tests: {}/{}",
            self.passed_tests.to_string().cyan(),
            self.total_tests
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_tutorial_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        assert!(!tutorial.chapters.is_empty());
        assert_eq!(tutorial.progress.current_chapter, 0);
        assert_eq!(tutorial.progress.current_lesson, 0);
    }

    #[test]
    fn test_chapter_access() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let chapter = tutorial.get_chapter(0);
        assert!(chapter.is_some());

        let chapter = chapter.unwrap();
        assert!(!chapter.lessons.is_empty());
    }

    #[test]
    fn test_lesson_completion() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        let lesson_id = "chapter1_lesson1";
        assert!(!tutorial.is_lesson_complete(lesson_id));

        tutorial.mark_lesson_complete(lesson_id);
        assert!(tutorial.is_lesson_complete(lesson_id));
    }

    #[test]
    fn test_progress_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        {
            let mut tutorial = Tutorial::with_progress_file(path);
            tutorial.mark_lesson_complete("test_lesson");
            tutorial.save_progress().unwrap();
        }

        {
            let tutorial = Tutorial::with_progress_file(path);
            assert!(tutorial.is_lesson_complete("test_lesson"));
        }
    }

    #[test]
    fn test_hint_system() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        tutorial.progress.current_chapter = 0;
        tutorial.progress.current_lesson = 0;

        let lesson_id = tutorial.get_lesson(0, 0).unwrap().id.clone();

        let hint = tutorial.use_hint(&lesson_id);
        assert!(hint.is_some(), "First hint should exist");
    }

    #[test]
    fn test_navigation() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        tutorial.goto_lesson(0, 0).unwrap();
        assert_eq!(tutorial.progress.current_chapter, 0);
        assert_eq!(tutorial.progress.current_lesson, 0);

        tutorial.advance_lesson();
        assert_eq!(tutorial.progress.current_lesson, 1);
    }

    #[test]
    fn test_code_validation() {
        let tutorial = Tutorial::new();
        let lesson = tutorial.get_lesson(0, 0).unwrap();

        let result = tutorial.validate_code(&lesson.solution, lesson);
        assert!(result.is_ok());
    }
}
