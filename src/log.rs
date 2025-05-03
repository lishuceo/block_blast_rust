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
    // 声明用于JavaScript调用的函数
    fn js_invoke_string(js_code_ptr: *const u8, js_code_len: usize) -> i32;
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

// 新增: 专门用于在屏幕上显示消息的函数
#[cfg(target_arch = "wasm32")]
pub fn show_message(message: &str, level: LogLevel) {
    // 在WASM环境下，直接使用JavaScript显示函数
    unsafe {
        // 调用cloud.rs中已定义的JavaScript函数
        let js_code = format!("window.showGameMessage(\"{}\", {})", message.replace("\"", "\\\""), level as i32);
        
        // 将JavaScript代码传递给JS环境
        let js_code_bytes = js_code.as_bytes();
        js_invoke_string(js_code_bytes.as_ptr(), js_code_bytes.len());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_message(message: &str, level: LogLevel) {
    // 在非WASM环境下，目前仅输出到控制台，未来可添加图形界面显示
    log(message, level);
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

// 新增: 用于在屏幕上显示消息的宏
#[macro_export]
macro_rules! show_debug {
    ($($arg:tt)*) => {
        $crate::log::show_message(&format!($($arg)*), $crate::log::LogLevel::Debug);
    };
}

#[macro_export]
macro_rules! show_info {
    ($($arg:tt)*) => {
        $crate::log::show_message(&format!($($arg)*), $crate::log::LogLevel::Info);
    };
}

#[macro_export]
macro_rules! show_warn {
    ($($arg:tt)*) => {
        $crate::log::show_message(&format!($($arg)*), $crate::log::LogLevel::Warning);
    };
}

#[macro_export]
macro_rules! show_error {
    ($($arg:tt)*) => {
        $crate::log::show_message(&format!($($arg)*), $crate::log::LogLevel::Error);
    };
} 