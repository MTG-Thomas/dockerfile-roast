use dockerfile_roast::limits::{
    MAX_DOCKERFILE_BYTES, MAX_DOCKERFILE_INSTRUCTIONS, MAX_DOCKERFILE_LINES, RESOURCE_LIMIT_RULE,
};
use dockerfile_roast::linter::{lint_content, lint_file, LintOptions};
use dockerfile_roast::parser::parse_document;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn library_rejects_content_over_the_shared_byte_budget() {
    let source = format!("FROM scratch\n#{}", "x".repeat(MAX_DOCKERFILE_BYTES));
    let result = lint_content(&source, "Dockerfile", &LintOptions::default());

    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule, RESOURCE_LIMIT_RULE);
    assert!(result.findings[0].message.contains("byte lint input limit"));
}

#[test]
fn file_lint_rejects_oversized_input_before_parsing() {
    let path = std::env::temp_dir().join(format!(
        "droast-resource-limit-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&path, vec![b'x'; MAX_DOCKERFILE_BYTES + 1]).unwrap();

    let error = match lint_file(&path, &LintOptions::default()) {
        Ok(_) => panic!("oversized Dockerfile should be rejected"),
        Err(error) => error,
    };
    std::fs::remove_file(&path).unwrap();

    assert!(error.to_string().contains("byte limit"));
}

#[test]
fn library_rejects_excessive_lines_and_instructions() {
    let lines = "#\n".repeat(MAX_DOCKERFILE_LINES + 1);
    let line_result = lint_content(&lines, "Dockerfile", &LintOptions::default());
    assert_eq!(line_result.findings[0].rule, RESOURCE_LIMIT_RULE);
    assert!(line_result.findings[0]
        .message
        .contains("line lint execution limit"));

    let instructions = "RUN true\n".repeat(MAX_DOCKERFILE_INSTRUCTIONS + 1);
    let instruction_result = lint_content(&instructions, "Dockerfile", &LintOptions::default());
    assert_eq!(instruction_result.findings[0].rule, RESOURCE_LIMIT_RULE);
    assert!(instruction_result.findings[0]
        .message
        .contains("instruction lint execution limit"));
}

#[test]
fn parser_preserves_spans_for_many_physical_lines() {
    let mut source = String::new();
    for _ in 0..20_000 {
        source.push_str("# comment\n");
    }
    source.push_str("FROM scratch\n");

    let document = parse_document(&source);
    assert_eq!(document.instructions[0].span.start.line, 20_001);
    assert_eq!(document.instructions[0].keyword_span.text(&source), "FROM");
}

#[test]
fn cli_bounds_stdin_before_allocating_the_complete_stream() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_droast"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&vec![b'x'; MAX_DOCKERFILE_BYTES + 1])
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("byte Dockerfile lint input limit"));
}
