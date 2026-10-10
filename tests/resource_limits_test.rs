use dockerfile_roast::limits::{MAX_DOCKERFILE_BYTES, RESOURCE_LIMIT_RULE};
use dockerfile_roast::linter::{lint_content, lint_file, LintOptions};
use dockerfile_roast::parser::parse_document;

#[test]
fn library_rejects_content_over_the_shared_byte_budget() {
    let source = format!("FROM scratch\n#{}", "x".repeat(MAX_DOCKERFILE_BYTES));
    let result = lint_content(&source, "Dockerfile", &LintOptions::default());

    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule, RESOURCE_LIMIT_RULE);
    assert!(result.findings[0].message.contains("byte limit"));
}

#[test]
fn file_lint_rejects_oversized_input_before_parsing() {
    let path = std::env::temp_dir().join(format!(
        "droast-resource-limit-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&path, vec![b'x'; MAX_DOCKERFILE_BYTES + 1]).unwrap();

    let error = lint_file(&path, &LintOptions::default()).unwrap_err();
    std::fs::remove_file(&path).unwrap();

    assert!(error.to_string().contains("byte limit"));
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
