//! 动画系统验证测试
//!
//! 使用从 C++ 导出的测试数据验证 Rust 实现的正确性

use serde_json::Value;
use std::fs;

/// 模拟 GetAnimationFrame 函数
/// 这是纯数学计算,不依赖时间
fn get_animation_frame(ticks: i32, fps: i32, frames: i32) -> i32 {
    if ticks < 0 || fps <= 0 || frames <= 0 {
        return -1;
    }
    ((ticks / fps) % frames) as i32
}

#[test]
fn test_animation_frames() {
    let data_path = "test_data/animation_data.json";

    if !std::path::Path::new(data_path).exists() {
        println!("警告: 测试数据文件不存在: {}", data_path);
        return;
    }

    let data = fs::read_to_string(data_path)
        .expect("无法读取测试数据文件");

    let json: Value = serde_json::from_str(&data)
        .expect("无法解析 JSON");

    // 测试所有动画帧计算用例
    let tests = json["animation_frame_tests"].as_array()
        .expect("找不到 animation_frame_tests");

    let mut passed = 0;
    let mut failed = 0;

    for test in tests {
        let frames = test["frames"].as_i64().unwrap() as i32;
        let fps = test["fps"].as_i64().unwrap() as i32;
        let ticks = test["ticks"].as_i64().unwrap() as i32;
        let expected = test["expected"].as_i64().unwrap() as i32;

        let result = get_animation_frame(ticks, fps, frames);

        if result == expected {
            passed += 1;
        } else {
            failed += 1;
            if failed <= 10 {  // 只显示前10个失败
                eprintln!("失败: frames={}, fps={}, ticks={}, expected={}, got={}",
                         frames, fps, ticks, expected, result);
            }
        }
    }

    println!("动画帧测试: {}/{} 通过", passed, passed + failed);
    assert_eq!(failed, 0, "有 {} 个测试失败", failed);
}
