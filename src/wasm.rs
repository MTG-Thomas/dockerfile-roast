use crate::{limits, parser, rules};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct WasmFinding {
    rule: String,
    severity: String,
    line: usize,
    message: String,
    roast: String,
}

#[derive(Serialize)]
struct WasmResult {
    total: usize,
    errors: usize,
    warnings: usize,
    infos: usize,
    findings: Vec<WasmFinding>,
}

/// Lint a Dockerfile passed as a string; returns findings as a JSON string.
#[wasm_bindgen]
pub fn lint(content: &str) -> String {
    let mut findings: Vec<rules::Finding> = Vec::new();
    if let Err(exceeded) = limits::validate_source(content) {
        findings.push(resource_limit_finding(exceeded.message));
        return serialize(findings);
    }
    let instructions = parser::parse(content);
    if let Err(exceeded) = limits::validate_instruction_count(instructions.len()) {
        findings.push(resource_limit_finding(exceeded.message));
        return serialize(findings);
    }
    for rule in rules::all_rules() {
        let rule_findings = (rule.func)(&instructions, content);
        if let Err(exceeded) =
            limits::validate_finding_count(findings.len().saturating_add(rule_findings.len()))
        {
            return serialize(vec![resource_limit_finding(exceeded.message)]);
        }
        findings.extend(rule_findings);
    }

    findings.sort_by(|a, b| a.line.cmp(&b.line).then(b.severity.cmp(&a.severity)));

    serialize(findings)
}

fn resource_limit_finding(message: String) -> rules::Finding {
    rules::Finding {
        rule: limits::RESOURCE_LIMIT_RULE.into(),
        severity: rules::Severity::Error,
        line: 1,
        column: 1,
        end_line: 1,
        end_column: 1,
        message,
        roast: "This Dockerfile exhausted its lint budget before it could exhaust the runner. Split or simplify it.".into(),
    }
}

fn serialize(findings: Vec<rules::Finding>) -> String {
    let result = WasmResult {
        total: findings.len(),
        errors: findings
            .iter()
            .filter(|f| f.severity == rules::Severity::Error)
            .count(),
        warnings: findings
            .iter()
            .filter(|f| f.severity == rules::Severity::Warning)
            .count(),
        infos: findings
            .iter()
            .filter(|f| f.severity == rules::Severity::Info)
            .count(),
        findings: findings
            .iter()
            .map(|f| WasmFinding {
                rule: f.rule.to_string(),
                severity: f.severity.to_string(),
                line: f.line,
                message: f.message.clone(),
                roast: f.roast.clone(),
            })
            .collect(),
    };

    serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
}
