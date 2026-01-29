use scrcpy_launcher_lib::scrcpy;

#[test]
fn test_adb_status_structure() {
    // 这是一个简单的冒烟测试，确保我们可以导入并使用库中的类型
    //以此验证库的导出是否正常
    let status = scrcpy::AdbStatus {
        connected: false,
        message: "Test".to_string(),
    };
    assert_eq!(status.connected, false);
    assert_eq!(status.message, "Test");
}
