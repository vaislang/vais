use vais_tutorial::{Tutorial, TutorialError};
use tempfile::NamedTempFile;

#[test]
fn test_tutorial_initialization() {
    let tutorial = Tutorial::new();
    assert_eq!(tutorial.chapters.len(), 5);
    assert_eq!(tutorial.progress.current_chapter, 0);
    assert_eq!(tutorial.progress.current_lesson, 0);
}

#[test]
fn test_chapter_navigation() {
    let tutorial = Tutorial::new();

    // Test valid chapter access
    let chapter = tutorial.get_chapter(0);
    assert!(chapter.is_some());
    assert_eq!(chapter.unwrap().id, 0);

    // Test invalid chapter access
    let chapter = tutorial.get_chapter(100);
    assert!(chapter.is_none());
}

#[test]
fn test_lesson_navigation() {
    let tutorial = Tutorial::new();

    // Test valid lesson access
    let lesson = tutorial.get_lesson(0, 0);
    assert!(lesson.is_some());

    // Test invalid lesson access
    let lesson = tutorial.get_lesson(0, 100);
    assert!(lesson.is_none());
}

#[test]
fn test_lesson_completion() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    let lesson_id = "test_lesson";
    assert!(!tutorial.is_lesson_complete(lesson_id));

    tutorial.mark_lesson_complete(lesson_id);
    assert!(tutorial.is_lesson_complete(lesson_id));
}

#[test]
fn test_progress_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create tutorial and mark lesson complete
    {
        let mut tutorial = Tutorial::with_progress_file(path);
        tutorial.mark_lesson_complete("lesson1");
        tutorial.save_progress().unwrap();
    }

    // Load tutorial and check progress
    {
        let tutorial = Tutorial::with_progress_file(path);
        assert!(tutorial.is_lesson_complete("lesson1"));
    }
}

#[test]
fn test_hint_system() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    // Set current position
    tutorial.progress.current_chapter = 0;
    tutorial.progress.current_lesson = 0;

    if let Some(lesson) = tutorial.get_lesson(0, 0) {
        let lesson_id = lesson.id.clone();
        let hint_count = lesson.hints.len();

        // Request hints
        for i in 0..hint_count {
            let hint = tutorial.use_hint(&lesson_id);
            assert!(hint.is_some(), "Hint {} should exist", i);
        }

        // No more hints available
        let hint = tutorial.use_hint(&lesson_id);
        assert!(hint.is_none(), "No more hints should be available");
    }
}

#[test]
fn test_lesson_advancement() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    let initial_chapter = tutorial.progress.current_chapter;
    let initial_lesson = tutorial.progress.current_lesson;

    tutorial.advance_lesson();

    // Should advance to next lesson or chapter
    assert!(
        tutorial.progress.current_lesson > initial_lesson
            || tutorial.progress.current_chapter > initial_chapter
    );
}

#[test]
fn test_goto_lesson() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    // Go to valid lesson
    let result = tutorial.goto_lesson(1, 0);
    assert!(result.is_ok());
    assert_eq!(tutorial.progress.current_chapter, 1);
    assert_eq!(tutorial.progress.current_lesson, 0);

    // Go to invalid chapter
    let result = tutorial.goto_lesson(100, 0);
    assert!(result.is_err());

    // Go to invalid lesson
    let result = tutorial.goto_lesson(0, 100);
    assert!(result.is_err());
}

#[test]
fn test_all_chapters_have_content() {
    let tutorial = Tutorial::new();

    for chapter in &tutorial.chapters {
        assert!(!chapter.title.is_empty());
        assert!(!chapter.description.is_empty());
        assert!(!chapter.lessons.is_empty());

        for lesson in &chapter.lessons {
            assert!(!lesson.id.is_empty());
            assert!(!lesson.title.is_empty());
            assert!(!lesson.description.is_empty());
            assert!(!lesson.content.is_empty());
            assert!(!lesson.code_template.is_empty());
            assert!(!lesson.solution.is_empty());
            assert!(!lesson.test_cases.is_empty());
        }
    }
}

#[test]
fn test_lesson_ids_are_unique() {
    let tutorial = Tutorial::new();
    let mut ids = std::collections::HashSet::new();

    for chapter in &tutorial.chapters {
        for lesson in &chapter.lessons {
            assert!(
                ids.insert(&lesson.id),
                "Duplicate lesson ID: {}",
                lesson.id
            );
        }
    }
}

#[test]
fn test_code_validation_basic() {
    let tutorial = Tutorial::new();

    if let Some(lesson) = tutorial.get_lesson(0, 0) {
        // Test with valid solution
        let result = tutorial.validate_code(&lesson.solution, lesson);
        assert!(result.is_ok());

        // Test with invalid code - validation returns Ok but with errors
        let invalid_code = "this is not valid Vais code !!!";
        let result = tutorial.validate_code(invalid_code, lesson);
        assert!(result.is_ok()); // Function returns Ok, but result.success should be false
        if let Ok(validation) = result {
            assert!(!validation.success);
            assert!(!validation.errors.is_empty());
        }
    }
}

#[test]
fn test_next_lesson_iteration() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    let mut lesson_count = 0;
    while tutorial.next_lesson().is_some() {
        lesson_count += 1;
        tutorial.advance_lesson();

        // Prevent infinite loop
        if lesson_count > 100 {
            break;
        }
    }

    assert!(lesson_count > 0, "Should have at least one lesson");
}

#[test]
fn test_progress_statistics() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    // Collect lesson IDs first
    let lesson_ids: Vec<String> = if let Some(chapter) = tutorial.get_chapter(0) {
        chapter.lessons.iter().map(|l| l.id.clone()).collect()
    } else {
        vec![]
    };

    // Mark lessons complete
    for lesson_id in lesson_ids {
        tutorial.mark_lesson_complete(&lesson_id);
    }

    let completed = tutorial.progress.completed_lessons.len();
    assert!(completed > 0);
}

#[test]
fn test_chapter_listing() {
    let tutorial = Tutorial::new();

    // This should not panic
    tutorial.list_chapters();
}

#[test]
fn test_lesson_listing() {
    let tutorial = Tutorial::new();

    // Valid chapter
    let result = tutorial.list_lessons(0);
    assert!(result.is_ok());

    // Invalid chapter
    let result = tutorial.list_lessons(100);
    assert!(result.is_err());
}

#[test]
fn test_validation_result() {
    use vais_tutorial::ValidationResult;

    let result = ValidationResult {
        success: true,
        message: "Test passed".to_string(),
        passed_tests: 5,
        total_tests: 5,
        errors: vec![],
    };

    // Should not panic
    result.print();
}

#[test]
fn test_error_types() {
    // Test ParseError
    let err = TutorialError::ParseError("test error".to_string());
    assert!(err.to_string().contains("test error"));

    // Test ValidationError
    let err = TutorialError::ValidationError("validation failed".to_string());
    assert!(err.to_string().contains("validation failed"));

    // Test LessonNotFound
    let err = TutorialError::LessonNotFound("lesson1".to_string());
    assert!(err.to_string().contains("lesson1"));

    // Test ChapterNotFound
    let err = TutorialError::ChapterNotFound(5);
    assert!(err.to_string().contains("5"));
}

#[test]
fn test_multiple_tutorials_same_progress_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // First tutorial marks lesson complete
    {
        let mut tutorial = Tutorial::with_progress_file(path);
        tutorial.mark_lesson_complete("shared_lesson");
    }

    // Second tutorial should see the completion
    {
        let tutorial = Tutorial::with_progress_file(path);
        assert!(tutorial.is_lesson_complete("shared_lesson"));
    }
}

#[test]
fn test_hint_count_tracking() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut tutorial = Tutorial::with_progress_file(temp_file.path());

    tutorial.progress.current_chapter = 0;
    tutorial.progress.current_lesson = 0;

    if let Some(lesson) = tutorial.get_lesson(0, 0) {
        let lesson_id = lesson.id.clone();

        // First hint
        tutorial.use_hint(&lesson_id);
        assert_eq!(tutorial.progress.hints_used.get(&lesson_id), Some(&1));

        // Second hint
        tutorial.use_hint(&lesson_id);
        assert_eq!(tutorial.progress.hints_used.get(&lesson_id), Some(&2));
    }
}
