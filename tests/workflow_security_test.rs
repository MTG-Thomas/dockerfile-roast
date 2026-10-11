use std::fs;
use std::path::Path;

fn repository_file(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn publishing_workflows_use_locked_or_checksum_verified_tooling() {
    let vscode = repository_file(".github/workflows/vscode-publish.yml");
    assert!(vscode.contains("npm ci --ignore-scripts"));
    assert!(vscode.contains("./node_modules/.bin/vsce publish"));
    assert!(!vscode.contains("npm install -g"));
    assert!(!vscode.contains("\n    env:\n      VSCE_PAT:"));

    let package_lock = repository_file("vscode-extension/package-lock.json");
    assert!(package_lock.contains("\"@vscode/vsce\": \"4.0.0\""));
    assert!(package_lock.contains("\"integrity\":"));

    for workflow in [
        repository_file(".github/workflows/pages.yml"),
        repository_file(".github/workflows/docker-publish.yml"),
    ] {
        assert!(workflow.contains("sha256sum -c -"));
        assert!(!workflow.contains("curl https://rustwasm.github.io/wasm-pack/installer/init.sh"));
        assert!(!workflow.contains("curl https://get.wasmer.io"));
    }
}
