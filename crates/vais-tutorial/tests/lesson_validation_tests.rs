use vais_tutorial::Tutorial;

#[test]
fn test_chapter1_lesson1_solution() {
    let tutorial = Tutorial::new();
    let lesson = tutorial.get_lesson(0, 0).unwrap();

    let result = tutorial.validate_code(&lesson.solution, lesson);
    assert!(result.is_ok());
}

#[test]
fn test_chapter1_lesson2_solution() {
    let tutorial = Tutorial::new();
    let lesson = tutorial.get_lesson(0, 1).unwrap();

    let result = tutorial.validate_code(&lesson.solution, lesson);
    assert!(result.is_ok());
}

#[test]
fn test_chapter2_lesson1_solution() {
    let tutorial = Tutorial::new();
    let lesson = tutorial.get_lesson(1, 0).unwrap();

    let result = tutorial.validate_code(&lesson.solution, lesson);
    assert!(result.is_ok());
}

#[test]
fn test_all_lesson_solutions_parse() {
    let tutorial = Tutorial::new();

    for (chapter_idx, chapter) in tutorial.chapters.iter().enumerate() {
        for (lesson_idx, lesson) in chapter.lessons.iter().enumerate() {
            let result = tutorial.validate_code(&lesson.solution, lesson);
            assert!(
                result.is_ok(),
                "Chapter {} Lesson {} solution failed to validate: {:?}",
                chapter_idx,
                lesson_idx,
                result.err()
            );
        }
    }
}

#[test]
fn test_lesson_structure_consistency() {
    let tutorial = Tutorial::new();

    for chapter in &tutorial.chapters {
        // Each chapter should have at least one lesson
        assert!(
            !chapter.lessons.is_empty(),
            "Chapter {} has no lessons",
            chapter.id
        );

        for lesson in &chapter.lessons {
            // Lesson should have non-empty fields
            assert!(!lesson.id.is_empty(), "Lesson has empty ID");
            assert!(!lesson.title.is_empty(), "Lesson has empty title");
            assert!(
                !lesson.description.is_empty(),
                "Lesson has empty description"
            );
            assert!(!lesson.content.is_empty(), "Lesson has empty content");
            assert!(!lesson.solution.is_empty(), "Lesson has empty solution");

            // At least one test case
            assert!(
                !lesson.test_cases.is_empty(),
                "Lesson {} has no test cases",
                lesson.id
            );

            // At least one hint
            assert!(
                !lesson.hints.is_empty(),
                "Lesson {} has no hints",
                lesson.id
            );
        }
    }
}

#[test]
fn test_chapter_progression() {
    let tutorial = Tutorial::new();

    // Verify chapters are numbered sequentially
    for (idx, chapter) in tutorial.chapters.iter().enumerate() {
        assert_eq!(chapter.id, idx, "Chapter ID doesn't match index");
    }
}

#[test]
fn test_hint_quality() {
    let tutorial = Tutorial::new();

    for chapter in &tutorial.chapters {
        for lesson in &chapter.lessons {
            // Each lesson should have multiple hints
            assert!(
                lesson.hints.len() >= 2,
                "Lesson {} has too few hints ({})",
                lesson.id,
                lesson.hints.len()
            );

            // Hints should be progressively more helpful
            // First hint should be shorter/more general
            // Last hint should be the most specific
            let first_hint = &lesson.hints[0];
            let last_hint = &lesson.hints[lesson.hints.len() - 1];

            assert!(!first_hint.is_empty(), "First hint is empty");
            assert!(!last_hint.is_empty(), "Last hint is empty");
        }
    }
}

#[test]
fn test_code_templates_not_solutions() {
    let tutorial = Tutorial::new();

    for chapter in &tutorial.chapters {
        for lesson in &chapter.lessons {
            // Code template should not be the same as solution
            assert_ne!(
                lesson.code_template.trim(),
                lesson.solution.trim(),
                "Lesson {} template is the same as solution",
                lesson.id
            );
        }
    }
}
