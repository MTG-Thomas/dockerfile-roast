//! Shared resource limits for every Dockerfile lint entry point.

pub const RESOURCE_LIMIT_RULE: &str = "DF080";
pub const MAX_DOCKERFILE_BYTES: usize = 1024 * 1024;
pub const MAX_DOCKERFILE_LINES: usize = 50_000;
pub const MAX_DOCKERFILE_INSTRUCTIONS: usize = 10_000;
pub const MAX_DOCKERFILE_FINDINGS: usize = 20_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitExceeded {
    pub message: String,
}

pub fn validate_source(content: &str) -> Result<(), LimitExceeded> {
    if content.len() > MAX_DOCKERFILE_BYTES {
        return Err(LimitExceeded {
            message: format!("Dockerfile exceeds the {MAX_DOCKERFILE_BYTES}-byte lint input limit"),
        });
    }
    let lines = content.bytes().filter(|byte| *byte == b'\n').count()
        + usize::from(!content.is_empty() && !content.ends_with('\n'));
    if lines > MAX_DOCKERFILE_LINES {
        return Err(LimitExceeded {
            message: format!(
                "Dockerfile exceeds the {MAX_DOCKERFILE_LINES}-line lint execution limit"
            ),
        });
    }
    Ok(())
}

pub fn validate_instruction_count(count: usize) -> Result<(), LimitExceeded> {
    if count > MAX_DOCKERFILE_INSTRUCTIONS {
        return Err(LimitExceeded {
            message: format!(
                "Dockerfile exceeds the {MAX_DOCKERFILE_INSTRUCTIONS}-instruction lint execution limit"
            ),
        });
    }
    Ok(())
}

pub fn validate_finding_count(count: usize) -> Result<(), LimitExceeded> {
    if count > MAX_DOCKERFILE_FINDINGS {
        return Err(LimitExceeded {
            message: format!(
                "Dockerfile exceeds the {MAX_DOCKERFILE_FINDINGS}-finding lint output limit"
            ),
        });
    }
    Ok(())
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn read_stdin() -> anyhow::Result<String> {
    use std::io::Read;

    let mut bytes = Vec::new();
    std::io::stdin()
        .take(MAX_DOCKERFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_DOCKERFILE_BYTES {
        anyhow::bail!("stdin exceeds the {MAX_DOCKERFILE_BYTES}-byte Dockerfile lint input limit");
    }
    String::from_utf8(bytes).map_err(|error| anyhow::anyhow!("stdin is not UTF-8: {error}"))
}
