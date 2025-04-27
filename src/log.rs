//! 日志模块 - 提供跨平台日志功能
//! 在WASM环境中通过JavaScript输出到浏览器控制台

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    // 声明JavaScript定义的控制台日志函数 (已重命名避免冲突)
    fn game_log_js(text_ptr: *const u8, text_len: usize, log_level: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
pub fn log(message: &str, level: LogLevel) {
    unsafe {
        // 调用重命名后的函数
        game_log_js(
            message.as_ptr(),
            message.len(),
            level as i32,
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(message: &str, level: LogLevel) {
    // 非WASM环境下直接使用标准输出
    match level {
        LogLevel::Debug => println!("[DEBUG] {}", message),
        LogLevel::Info => println!("[INFO] {}", message),
        LogLevel::Warning => eprintln!("[WARN] {}", message),
        LogLevel::Error => eprintln!("[ERROR] {}", message),
    }
}

// 便捷宏

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log::log(&format!($($arg)*), $crate::log::LogLevel::Debug);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log::log(&format!($($arg)*), $crate::log::LogLevel::Info);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log::log(&format!($($arg)*), $crate::log::LogLevel::Warning);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log::log(&format!($($arg)*), $crate::log::LogLevel::Error);
    };
} 