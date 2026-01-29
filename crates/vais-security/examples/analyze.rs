//! Example usage of the Vais security analyzer

use vais_lexer::tokenize;
use vais_parser::Parser;
use vais_security::{SecurityAnalyzer, Severity};

fn main() {
    // Example vulnerable code
    let source = r#"
        F handle_request(user_input: String) -> i64 {
            # Vulnerable: buffer overflow risk
            ptr := malloc(100)
            store_i64(ptr + user_input, 42)

            # Vulnerable: command injection
            cmd := "rm -rf " + user_input
            system(cmd)

            # Vulnerable: hardcoded credentials
            password := "super_secret_password_123"
            authenticate("admin", password)

            free(ptr)
            0
        }
    "#;

    println!("Analyzing Vais code for security vulnerabilities...\n");
    println!("Source code:");
    println!("{}\n", source);

    // Parse the code
    let tokens = tokenize(source).expect("Failed to tokenize");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Failed to parse");

    // Analyze for security issues
    let mut analyzer = SecurityAnalyzer::new();
    let findings = analyzer.analyze(&module);

    // Report findings
    println!("Security Analysis Results:");
    println!("==========================\n");

    if findings.is_empty() {
        println!("No security issues found!");
    } else {
        println!("Found {} security issue(s):\n", findings.len());

        // Group by severity
        let mut critical = Vec::new();
        let mut high = Vec::new();
        let mut medium = Vec::new();
        let mut low = Vec::new();

        for finding in &findings {
            match finding.severity {
                Severity::Critical => critical.push(finding),
                Severity::High => high.push(finding),
                Severity::Medium => medium.push(finding),
                Severity::Low => low.push(finding),
                Severity::Info => {},
            }
        }

        for (severity_name, group) in [
            ("CRITICAL", critical),
            ("HIGH", high),
            ("MEDIUM", medium),
            ("LOW", low),
        ] {
            if !group.is_empty() {
                println!("{} Severity ({} issue(s)):", severity_name, group.len());
                println!("{}", "=".repeat(50));
                for finding in group {
                    println!("{}", finding);
                }
            }
        }

        // Summary
        println!("\nSummary:");
        println!("--------");
        println!("Total issues: {}", findings.len());
        println!("Critical: {}", findings.iter().filter(|f| f.severity == Severity::Critical).count());
        println!("High: {}", findings.iter().filter(|f| f.severity == Severity::High).count());
        println!("Medium: {}", findings.iter().filter(|f| f.severity == Severity::Medium).count());
        println!("Low: {}", findings.iter().filter(|f| f.severity == Severity::Low).count());
    }
}
