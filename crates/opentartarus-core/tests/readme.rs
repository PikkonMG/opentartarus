#[test]
fn readme_uses_official_name_and_github() {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    let t = std::fs::read_to_string(p).unwrap();
    assert!(t.contains("OpenTartarus"));
    assert!(t.contains("https://github.com/PikkonMG/opentartarus"));
    assert!(!t.contains("OpenTaris"));
    assert!(!t.contains("systemctl"));
    assert!(!t.contains("sudo "));
}
