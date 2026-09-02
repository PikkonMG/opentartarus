#[test]
fn udev_rule_allow_list_only() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packaging/udev/99-opentartarus.rules");
    let text = std::fs::read_to_string(&root).expect("udev rule");
    assert!(text.contains("idProduct}==\"022b\""));
    assert!(text.contains("idProduct}==\"0244\""));
    assert!(text.contains("GROUP=\"opentartarus\""));
    assert!(text.contains("KERNEL==\"uinput\""));
    assert!(!text.contains("008f"));
    assert!(!text.contains("0090"));
}
