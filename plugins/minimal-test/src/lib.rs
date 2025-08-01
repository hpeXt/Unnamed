use extism_pdk::*;

#[plugin_fn]
pub fn test() -> FnResult<String> {
    // 测试错误类型
    let result: Result<String, Error> = Err(Error::msg("test error"));
    
    // 尝试转换
    match result {
        Ok(s) => Ok(s),
        Err(e) => Err(e.into()), // 转换为 WithReturnCode
    }
}