use std::fs;

const RELEASE_IMAGE: &str =
    "1.7.0@sha256:4f31be49f26804c0a2f8a0ab0ae578719e30376323e8fd5951c6dcc79a478b40";

#[test]
fn action_defaults_to_immutable_image_and_read_only_workspace() {
    let action = fs::read_to_string("action.yml").expect("action.yml should be readable");

    assert!(action.contains(&format!("default: '{RELEASE_IMAGE}'")));
    assert!(!action.contains("default: 'latest'"));
    assert!(action.contains(r#"-v "$GITHUB_WORKSPACE:/workspace:ro""#));
    assert!(action.contains(r#""ghcr.io/immanuwell/droast:$INPUT_IMAGE_TAG""#));
}
