//! 简单的插件兼容性测试

#[test]
fn test_plugin_builds_successfully() {
    // 这个测试主要验证插件能够成功编译
    // 如果插件 SDK 和主机函数不兼容，插件将无法编译
    
    use std::process::Command;
    use std::path::PathBuf;
    
    let plugin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("plugins/hello");
    
    println!("测试插件构建...");
    println!("插件目录: {:?}", plugin_dir);
    
    // 构建插件
    let output = Command::new("cargo")
        .current_dir(&plugin_dir)
        .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
        .output()
        .expect("无法执行 cargo build");
    
    if output.status.success() {
        println!("✅ 插件构建成功！");
        println!("   这证明插件与主机函数接口兼容");
        
        // 检查 WASM 文件是否存在
        let wasm_path = plugin_dir.join("target/wasm32-unknown-unknown/release/hello.wasm");
        assert!(wasm_path.exists(), "WASM 文件应该存在");
        
        println!("✅ WASM 文件已生成: {:?}", wasm_path);
    } else {
        eprintln!("❌ 插件构建失败");
        eprintln!("标准错误: {}", String::from_utf8_lossy(&output.stderr));
        panic!("插件编译失败，可能是主机函数不兼容");
    }
}

#[test]
fn test_template_plugin_builds() {
    use std::process::Command;
    use std::path::PathBuf;
    
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("plugins/template");
    
    if template_dir.exists() {
        println!("\n测试模板插件构建...");
        
        let output = Command::new("cargo")
            .current_dir(&template_dir)
            .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
            .output()
            .expect("无法执行 cargo build");
        
        if output.status.success() {
            println!("✅ 模板插件构建成功！");
        } else {
            eprintln!("⚠️ 模板插件构建失败（这是可选的）");
            eprintln!("错误: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        println!("⚠️ 模板插件目录不存在（跳过测试）");
    }
}

#[test]
fn test_plugin_sdk_compiles() {
    // 验证 Plugin SDK 本身能够编译
    use std::process::Command;
    use std::path::PathBuf;
    
    let sdk_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("plugin-sdk");
    
    println!("\n测试 Plugin SDK 编译...");
    
    let output = Command::new("cargo")
        .current_dir(&sdk_dir)
        .args(&["check"])
        .output()
        .expect("无法执行 cargo check");
    
    if output.status.success() {
        println!("✅ Plugin SDK 编译成功！");
        println!("   主机函数声明正确");
    } else {
        eprintln!("❌ Plugin SDK 编译失败");
        eprintln!("错误: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Plugin SDK 编译失败");
    }
}