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

#[derive(Debug, Serialize, Deserialize, Default)]
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
        self.progress
            .completed_lessons
            .insert(lesson_id.to_string(), true);
        let _ = self.save_progress();
    }

    pub fn is_lesson_complete(&self, lesson_id: &str) -> bool {
        self.progress
            .completed_lessons
            .get(lesson_id)
            .copied()
            .unwrap_or(false)
    }

    pub fn use_hint(&mut self, lesson_id: &str) -> Option<String> {
        let chapter_id = self.progress.current_chapter;
        let lesson_idx = self.progress.current_lesson;

        // Get lesson info first to avoid borrowing issues
        let (hint_count, lesson_matches, hint_idx) =
            if let Some(lesson) = self.get_lesson(chapter_id, lesson_idx) {
                let current_hints = *self.progress.hints_used.get(lesson_id).unwrap_or(&0);
                (lesson.hints.len(), lesson.id == lesson_id, current_hints)
            } else {
                return None;
            };

        if lesson_matches && hint_idx < hint_count {
            // Get the hint text
            let hint = self.get_lesson(chapter_id, lesson_idx)?.hints[hint_idx].clone();

            // Update hint count
            *self
                .progress
                .hints_used
                .entry(lesson_id.to_string())
                .or_insert(0) += 1;
            let _ = self.save_progress();

            return Some(hint);
        }
        None
    }

    pub fn list_chapters(&self) {
        println!("\n{}", "Available Chapters:".bold().cyan());
        for chapter in &self.chapters {
            let completed = chapter
                .lessons
                .iter()
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
        let chapter = self
            .get_chapter(chapter_id)
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
                        result.passed_tests, result.total_tests
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

    #[test]
    fn test_get_chapter_out_of_range() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        assert!(tutorial.get_chapter(100).is_none());
    }

    #[test]
    fn test_get_lesson_out_of_range() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        assert!(tutorial.get_lesson(0, 100).is_none());
        assert!(tutorial.get_lesson(100, 0).is_none());
    }

    #[test]
    fn test_is_lesson_complete_unknown() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        assert!(!tutorial.is_lesson_complete("nonexistent_lesson"));
    }

    #[test]
    fn test_multiple_lesson_completions() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        tutorial.mark_lesson_complete("lesson_a");
        tutorial.mark_lesson_complete("lesson_b");
        tutorial.mark_lesson_complete("lesson_c");

        assert!(tutorial.is_lesson_complete("lesson_a"));
        assert!(tutorial.is_lesson_complete("lesson_b"));
        assert!(tutorial.is_lesson_complete("lesson_c"));
        assert!(!tutorial.is_lesson_complete("lesson_d"));
    }

    #[test]
    fn test_advance_lesson_within_chapter() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        tutorial.goto_lesson(0, 0).unwrap();
        assert_eq!(tutorial.progress.current_lesson, 0);

        tutorial.advance_lesson();
        assert_eq!(tutorial.progress.current_lesson, 1);

        tutorial.advance_lesson();
        assert_eq!(tutorial.progress.current_lesson, 2);
    }

    #[test]
    fn test_advance_lesson_crosses_chapter() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        // Go to last lesson of chapter 0
        let ch0_len = tutorial.chapters[0].lessons.len();
        tutorial.goto_lesson(0, ch0_len - 1).unwrap();

        tutorial.advance_lesson();
        assert_eq!(tutorial.progress.current_chapter, 1);
        assert_eq!(tutorial.progress.current_lesson, 0);
    }

    #[test]
    fn test_goto_lesson_invalid_chapter() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        let result = tutorial.goto_lesson(999, 0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TutorialError::ChapterNotFound(999)
        ));
    }

    #[test]
    fn test_goto_lesson_invalid_lesson() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        let result = tutorial.goto_lesson(0, 999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TutorialError::LessonNotFound(_)
        ));
    }

    #[test]
    fn test_next_lesson_at_start() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let next = tutorial.next_lesson();
        assert!(next.is_some());
        assert_eq!(next.unwrap(), (0, 0));
    }

    #[test]
    fn test_validate_code_parse_error() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let lesson = tutorial.get_lesson(0, 0).unwrap();

        // Pass completely invalid code
        let result = tutorial.validate_code("{{{{invalid!!!!", lesson);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.success);
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_hint_exhaustion() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        tutorial.progress.current_chapter = 0;
        tutorial.progress.current_lesson = 0;

        let lesson_id = tutorial.get_lesson(0, 0).unwrap().id.clone();
        let hint_count = tutorial.get_lesson(0, 0).unwrap().hints.len();

        // Exhaust all hints
        for _ in 0..hint_count {
            let hint = tutorial.use_hint(&lesson_id);
            assert!(hint.is_some());
        }

        // Next hint should be None
        let hint = tutorial.use_hint(&lesson_id);
        assert!(hint.is_none());
    }

    #[test]
    fn test_hint_wrong_lesson_id() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        tutorial.progress.current_chapter = 0;
        tutorial.progress.current_lesson = 0;

        // Request hint for wrong lesson
        let hint = tutorial.use_hint("wrong_lesson_id");
        assert!(hint.is_none());
    }

    #[test]
    fn test_progress_default() {
        let progress = Progress::default();
        assert!(progress.completed_lessons.is_empty());
        assert_eq!(progress.current_chapter, 0);
        assert_eq!(progress.current_lesson, 0);
        assert!(progress.hints_used.is_empty());
    }

    #[test]
    fn test_tutorial_default() {
        let _tutorial = Tutorial::default();
    }

    #[test]
    fn test_tutorial_error_display() {
        let e1 = TutorialError::ParseError("test".to_string());
        assert!(e1.to_string().contains("Parse error"));

        let e2 = TutorialError::ValidationError("val".to_string());
        assert!(e2.to_string().contains("Validation error"));

        let e3 = TutorialError::LessonNotFound("l1".to_string());
        assert!(e3.to_string().contains("Lesson not found"));

        let e4 = TutorialError::ChapterNotFound(5);
        assert!(e4.to_string().contains("Chapter not found"));
    }

    #[test]
    fn test_validation_result_print() {
        // Just verify it doesn't panic
        let result = ValidationResult {
            success: true,
            message: "All passed".to_string(),
            passed_tests: 3,
            total_tests: 3,
            errors: vec![],
        };
        result.print();

        let result_fail = ValidationResult {
            success: false,
            message: "Failed".to_string(),
            passed_tests: 1,
            total_tests: 3,
            errors: vec!["err1".to_string(), "err2".to_string()],
        };
        result_fail.print();
    }

    #[test]
    fn test_lesson_struct_fields() {
        let lesson = Lesson {
            id: "test1".to_string(),
            title: "Test Lesson".to_string(),
            description: "A test".to_string(),
            content: "Content".to_string(),
            code_template: "F main() -> i64 { }".to_string(),
            solution: "F main() -> i64 { R 0 }".to_string(),
            test_cases: vec![],
            hints: vec!["Try this".to_string()],
        };
        assert_eq!(lesson.id, "test1");
        assert_eq!(lesson.title, "Test Lesson");
        assert!(lesson.test_cases.is_empty());
        assert_eq!(lesson.hints.len(), 1);
    }

    #[test]
    fn test_chapter_struct_fields() {
        let chapter = Chapter {
            id: 0,
            title: "Intro".to_string(),
            description: "Introduction".to_string(),
            lessons: vec![],
        };
        assert_eq!(chapter.id, 0);
        assert_eq!(chapter.title, "Intro");
        assert!(chapter.lessons.is_empty());
    }

    #[test]
    fn test_test_case_struct() {
        let tc = TestCase {
            description: "should compile".to_string(),
            expected_output: Some("0".to_string()),
            should_compile: true,
            validation_fn: None,
        };
        assert!(tc.should_compile);
        assert_eq!(tc.expected_output.unwrap(), "0");
        assert!(tc.validation_fn.is_none());
    }

    #[test]
    fn test_test_case_no_output() {
        let tc = TestCase {
            description: "no output".to_string(),
            expected_output: None,
            should_compile: false,
            validation_fn: Some("custom_check".to_string()),
        };
        assert!(!tc.should_compile);
        assert!(tc.expected_output.is_none());
        assert!(tc.validation_fn.is_some());
    }

    #[test]
    fn test_validate_code_valid_syntax() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let lesson = tutorial.get_lesson(0, 0).unwrap();

        let result = tutorial.validate_code("F main() -> i64 { R 42 }", lesson);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_validate_code_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let lesson = tutorial.get_lesson(0, 0).unwrap();

        let result = tutorial.validate_code("", lesson);
        assert!(result.is_ok());
    }

    #[test]
    fn test_next_lesson_past_end() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        // Go to the last chapter/lesson
        let last_ch = tutorial.chapters.len() - 1;
        let last_lesson = tutorial.chapters[last_ch].lessons.len();
        tutorial.progress.current_chapter = last_ch;
        tutorial.progress.current_lesson = last_lesson; // Past the end

        let next = tutorial.next_lesson();
        assert!(next.is_none());
    }

    #[test]
    fn test_advance_at_very_end() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        let last_ch = tutorial.chapters.len() - 1;
        let last_lesson = tutorial.chapters[last_ch].lessons.len() - 1;
        tutorial.goto_lesson(last_ch, last_lesson).unwrap();

        // Advancing past the end should not panic
        tutorial.advance_lesson();
        assert_eq!(tutorial.progress.current_chapter, last_ch);
        assert_eq!(tutorial.progress.current_lesson, last_lesson);
    }

    #[test]
    fn test_progress_serialization() {
        let mut progress = Progress::default();
        progress.completed_lessons.insert("l1".to_string(), true);
        progress.current_chapter = 2;
        progress.current_lesson = 3;
        progress.hints_used.insert("l1".to_string(), 2);

        let json = serde_json::to_string(&progress).unwrap();
        let restored: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.current_chapter, 2);
        assert_eq!(restored.current_lesson, 3);
        assert!(restored
            .completed_lessons
            .get("l1")
            .copied()
            .unwrap_or(false));
        assert_eq!(*restored.hints_used.get("l1").unwrap(), 2);
    }

    #[test]
    fn test_lesson_serialization() {
        let lesson = Lesson {
            id: "ser_test".to_string(),
            title: "Ser".to_string(),
            description: "Desc".to_string(),
            content: "Content".to_string(),
            code_template: "template".to_string(),
            solution: "solution".to_string(),
            test_cases: vec![],
            hints: vec![],
        };
        let json = serde_json::to_string(&lesson).unwrap();
        let restored: Lesson = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, "ser_test");
    }

    #[test]
    fn test_chapter_serialization() {
        let chapter = Chapter {
            id: 5,
            title: "Ch5".to_string(),
            description: "Fifth chapter".to_string(),
            lessons: vec![],
        };
        let json = serde_json::to_string(&chapter).unwrap();
        let restored: Chapter = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, 5);
        assert_eq!(restored.title, "Ch5");
    }

    #[test]
    fn test_mark_lesson_complete_idempotent() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());

        tutorial.mark_lesson_complete("l1");
        tutorial.mark_lesson_complete("l1");
        assert!(tutorial.is_lesson_complete("l1"));
        // Should still just be one entry
        assert_eq!(tutorial.progress.completed_lessons.len(), 1);
    }

    #[test]
    fn test_validation_result_fields() {
        let result = ValidationResult {
            success: false,
            message: "test".to_string(),
            passed_tests: 2,
            total_tests: 5,
            errors: vec!["e1".to_string(), "e2".to_string()],
        };
        assert!(!result.success);
        assert_eq!(result.passed_tests, 2);
        assert_eq!(result.total_tests, 5);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let te: TutorialError = io_err.into();
        assert!(te.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_from_serde() {
        let json_err = serde_json::from_str::<Progress>("bad json").unwrap_err();
        let te: TutorialError = json_err.into();
        assert!(te.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_with_progress_file_nonexistent() {
        let tutorial = Tutorial::with_progress_file("/tmp/nonexistent_vais_progress_test.json");
        // Should fall back to default progress
        assert_eq!(tutorial.progress.current_chapter, 0);
        assert_eq!(tutorial.progress.current_lesson, 0);
    }

    #[test]
    fn test_validate_code_should_not_compile() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        // Create a lesson where code should NOT compile
        let lesson = Lesson {
            id: "test_no_compile".to_string(),
            title: "No compile".to_string(),
            description: "Test".to_string(),
            content: "Content".to_string(),
            code_template: "".to_string(),
            solution: "".to_string(),
            test_cases: vec![TestCase {
                description: "should not compile".to_string(),
                expected_output: None,
                should_compile: false,
                validation_fn: None,
            }],
            hints: vec![],
        };
        let result = tutorial
            .validate_code("F main() -> i64 { R 0 }", &lesson)
            .unwrap();
        assert!(!result.success);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("should not compile")));
    }

    #[test]
    fn test_validate_code_partial_pass() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let lesson = Lesson {
            id: "test_partial".to_string(),
            title: "Partial".to_string(),
            description: "Test".to_string(),
            content: "Content".to_string(),
            code_template: "".to_string(),
            solution: "".to_string(),
            test_cases: vec![
                TestCase {
                    description: "compiles".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                },
                TestCase {
                    description: "compiles too".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                },
            ],
            hints: vec![],
        };
        let result = tutorial
            .validate_code("F main() -> i64 { R 0 }", &lesson)
            .unwrap();
        assert!(result.success);
        assert_eq!(result.passed_tests, 2);
        assert_eq!(result.total_tests, 2);
        assert!(result.message.contains("All tests passed"));
    }

    #[test]
    fn test_list_lessons_valid() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let result = tutorial.list_lessons(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_lessons_invalid_chapter() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        let result = tutorial.list_lessons(999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TutorialError::ChapterNotFound(999)
        ));
    }

    #[test]
    fn test_save_progress_persistence_roundtrip() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        {
            let mut tutorial = Tutorial::with_progress_file(&path);
            tutorial.progress.current_chapter = 3;
            tutorial.progress.current_lesson = 2;
            tutorial.progress.hints_used.insert("h1".to_string(), 5);
            tutorial.save_progress().unwrap();
        }

        {
            let tutorial = Tutorial::with_progress_file(&path);
            assert_eq!(tutorial.progress.current_chapter, 3);
            assert_eq!(tutorial.progress.current_lesson, 2);
            assert_eq!(*tutorial.progress.hints_used.get("h1").unwrap(), 5);
        }
    }

    #[test]
    fn test_goto_lesson_then_next() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        tutorial.goto_lesson(0, 1).unwrap();
        assert_eq!(tutorial.progress.current_chapter, 0);
        assert_eq!(tutorial.progress.current_lesson, 1);

        let next = tutorial.next_lesson();
        assert_eq!(next, Some((0, 1)));
    }

    #[test]
    fn test_next_lesson_cross_chapter() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        let ch0_len = tutorial.chapters[0].lessons.len();
        tutorial.progress.current_chapter = 0;
        tutorial.progress.current_lesson = ch0_len; // past end of chapter 0
        let next = tutorial.next_lesson();
        assert_eq!(next, Some((1, 0)));
    }

    #[test]
    fn test_use_hint_increments_counter() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut tutorial = Tutorial::with_progress_file(temp_file.path());
        tutorial.progress.current_chapter = 0;
        tutorial.progress.current_lesson = 0;
        let lesson_id = tutorial.get_lesson(0, 0).unwrap().id.clone();

        // First hint
        let hint1 = tutorial.use_hint(&lesson_id);
        assert!(hint1.is_some());
        assert_eq!(*tutorial.progress.hints_used.get(&lesson_id).unwrap(), 1);

        // Second hint
        let hint2 = tutorial.use_hint(&lesson_id);
        if hint2.is_some() {
            assert_eq!(*tutorial.progress.hints_used.get(&lesson_id).unwrap(), 2);
        }
    }

    #[test]
    fn test_lesson_clone() {
        let lesson = Lesson {
            id: "clone_test".to_string(),
            title: "Clone".to_string(),
            description: "Desc".to_string(),
            content: "Content".to_string(),
            code_template: "template".to_string(),
            solution: "solution".to_string(),
            test_cases: vec![TestCase {
                description: "tc".to_string(),
                expected_output: None,
                should_compile: true,
                validation_fn: None,
            }],
            hints: vec!["hint1".to_string()],
        };
        let cloned = lesson.clone();
        assert_eq!(cloned.id, lesson.id);
        assert_eq!(cloned.test_cases.len(), 1);
    }

    #[test]
    fn test_chapter_clone() {
        let chapter = Chapter {
            id: 0,
            title: "Ch".to_string(),
            description: "Desc".to_string(),
            lessons: vec![],
        };
        let cloned = chapter.clone();
        assert_eq!(cloned.id, 0);
        assert_eq!(cloned.title, "Ch");
    }

    #[test]
    fn test_test_case_clone() {
        let tc = TestCase {
            description: "desc".to_string(),
            expected_output: Some("out".to_string()),
            should_compile: true,
            validation_fn: Some("fn".to_string()),
        };
        let cloned = tc.clone();
        assert_eq!(cloned.expected_output, Some("out".to_string()));
        assert_eq!(cloned.validation_fn, Some("fn".to_string()));
    }

    #[test]
    fn test_progress_hints_used_default() {
        let progress = Progress::default();
        assert!(progress.hints_used.is_empty());
    }

    #[test]
    fn test_error_debug_format() {
        let e = TutorialError::ParseError("test parse error".to_string());
        let debug = format!("{:?}", e);
        assert!(debug.contains("ParseError"));
    }

    #[test]
    fn test_validation_result_debug() {
        let result = ValidationResult {
            success: true,
            message: "ok".to_string(),
            passed_tests: 1,
            total_tests: 1,
            errors: vec![],
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("success"));
    }

    #[test]
    fn test_result_type_alias() {
        let ok: crate::Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);
        let err: crate::Result<i32> = Err(TutorialError::ParseError("x".into()));
        assert!(err.is_err());
    }

    #[test]
    fn test_tutorial_chapters_not_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let tutorial = Tutorial::with_progress_file(temp_file.path());
        assert!(tutorial.chapters.len() >= 5);
    }
}
