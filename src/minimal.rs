// minimal.rs - 最小化的WebAssembly测试
// 不依赖任何外部库

// 导出一个简单的测试函数
#[no_mangle]
pub extern "C" fn test() -> i32 {
    42
}

// 这个函数只是打印一条消息，用于检查WASM是否正常工作
#[no_mangle]
pub extern "C" fn greet() {
    // 在WASM中，这个函数不会有任何实际输出
    // 但在本地调试时可用
} 