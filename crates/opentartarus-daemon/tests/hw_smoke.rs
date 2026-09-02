#[test]
fn tartarus_v2_and_openrazer_sysfs() {
    if std::env::var("OPENTARTARUS_HW_TEST").ok().as_deref() != Some("1") {
        return;
    }
    let found = evdev::enumerate().any(|(_, d)| {
        let id = d.input_id();
        id.vendor() == 0x1532 && id.product() == 0x022b
    });
    assert!(found, "expected Tartarus V2 1532:022b");
    let hid = std::path::Path::new("/sys/bus/hid/drivers/razerkbd");
    let mut brightness = false;
    if let Ok(rd) = std::fs::read_dir(hid) {
        for e in rd.flatten() {
            let n = e.file_name();
            if n.to_string_lossy().contains("1532:022B") {
                brightness |= e.path().join("matrix_brightness").is_file();
            }
        }
    }
    assert!(brightness, "expected OpenRazer matrix_brightness on V2 iface");
}
