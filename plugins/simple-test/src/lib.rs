use extism_pdk::*;

#[plugin_fn]
pub fn greet(name: String) -> FnResult<String> {
    Ok(format!("Hello, {}!", name))
}

// 测试主入口返回类型
#[plugin_fn]
pub fn test_main() -> FnResult<String> {
    Ok("test".to_string())
}