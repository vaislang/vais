use vais_tutorial::Tutorial;

fn main() {
    println!("Vais Tutorial System Demo\n");

    // Create a new tutorial
    let mut tutorial = Tutorial::new();

    // Display available chapters
    println!("=== Available Chapters ===");
    tutorial.list_chapters();

    // Navigate to first chapter and show lessons
    println!("\n=== Chapter 1 Lessons ===");
    tutorial.list_lessons(0).unwrap();

    // Get a specific lesson
    println!("\n=== Sample Lesson ===");
    let lesson_id = tutorial.get_lesson(0, 0).unwrap().id.clone();
    let lesson_title = tutorial.get_lesson(0, 0).unwrap().title.clone();
    let lesson_desc = tutorial.get_lesson(0, 0).unwrap().description.clone();
    let lesson_content = tutorial.get_lesson(0, 0).unwrap().content.clone();
    let lesson_template = tutorial.get_lesson(0, 0).unwrap().code_template.clone();
    let lesson_solution = tutorial.get_lesson(0, 0).unwrap().solution.clone();

    println!("Title: {}", lesson_title);
    println!("Description: {}", lesson_desc);
    println!("\nContent Preview:");
    println!(
        "{}",
        lesson_content
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!("...\n");

    println!("Code Template:");
    println!("{}", lesson_template);

    // Validate the solution
    println!("\n=== Validating Solution ===");
    let lesson = tutorial.get_lesson(0, 0).unwrap();
    match tutorial.validate_code(&lesson_solution, lesson) {
        Ok(result) => {
            println!("Success: {}", result.success);
            println!("Message: {}", result.message);
            println!("Passed: {}/{}", result.passed_tests, result.total_tests);
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }

    // Test hint system
    println!("\n=== Hint System ===");
    tutorial.progress.current_chapter = 0;
    tutorial.progress.current_lesson = 0;

    for i in 0..3 {
        match tutorial.use_hint(&lesson_id) {
            Some(hint) => println!("Hint {}: {}", i + 1, hint),
            None => {
                println!("No more hints available");
                break;
            }
        }
    }

    // Test lesson completion
    println!("\n=== Testing Progress Tracking ===");
    let test_lesson = "ch1_variables";
    println!(
        "Lesson '{}' completed: {}",
        test_lesson,
        tutorial.is_lesson_complete(test_lesson)
    );

    tutorial.mark_lesson_complete(test_lesson);
    println!(
        "After marking complete: {}",
        tutorial.is_lesson_complete(test_lesson)
    );

    // Show overall progress
    println!("\n=== Overall Progress ===");
    let total_lessons: usize = tutorial.chapters.iter().map(|ch| ch.lessons.len()).sum();
    let completed = tutorial.progress.completed_lessons.len();
    println!(
        "Completed: {}/{} lessons ({:.1}%)",
        completed,
        total_lessons,
        (completed as f64 / total_lessons as f64 * 100.0)
    );

    // Demonstrate navigation
    println!("\n=== Testing Navigation ===");
    println!(
        "Current position: Chapter {}, Lesson {}",
        tutorial.progress.current_chapter, tutorial.progress.current_lesson
    );

    if let Some((ch_id, lesson_idx)) = tutorial.next_lesson() {
        if let (Some(chapter), Some(lesson)) = (
            tutorial.get_chapter(ch_id),
            tutorial.get_lesson(ch_id, lesson_idx),
        ) {
            println!("Next lesson: {} - {}", chapter.title, lesson.title);
        }
    }

    tutorial.advance_lesson();
    println!(
        "After advance: Chapter {}, Lesson {}",
        tutorial.progress.current_chapter, tutorial.progress.current_lesson
    );

    // Test goto_lesson
    if tutorial.goto_lesson(1, 0).is_ok() {
        println!("Successfully jumped to Chapter 1, Lesson 0");
    }

    println!("\n=== Demo Complete ===");
    println!("\nTo run the interactive tutorial, use:");
    println!("  cargo run --example tutorial_interactive");
}
