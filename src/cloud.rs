#[cfg(target_arch = "wasm32")]
use macroquad::miniquad;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde_json;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// 导入宏
use crate::{log_debug, log_info, log_warn, log_error};

// 声明JavaScript外部函数
#[cfg(target_arch = "wasm32")]
extern "C" {
    // 声明JavaScript中定义的函数
    fn js_invoke_string(js_code_ptr: *const u8, js_code_len: usize) -> i32; // 同步调用
    fn js_invoke_async_string(js_code_ptr: *const u8, js_code_len: usize) -> i32; // 异步调用启动
    fn js_get_result_ptr() -> *const u8;
    fn js_get_result_len() -> usize;
    // 声明JavaScript定义的控制台日志函数 (已重命名避免冲突)
    fn game_log_js(text_ptr: *const u8, text_len: usize, log_level: i32) -> i32;
    // 内存管理函数由下面的 Rust 实现导出，无需在此声明
    // fn allocate_memory(size: usize) -> *mut u8;
    // fn deallocate_memory(ptr: *mut u8, size: usize);
}

// 封装JavaScript调用的安全函数
#[cfg(target_arch = "wasm32")]
fn call_js_function(js_code: &str) -> String {
    unsafe {
        // 将JavaScript代码传递给JS环境
        let js_code_bytes = js_code.as_bytes();
        js_invoke_string(js_code_bytes.as_ptr(), js_code_bytes.len());
        
        // 获取结果
        let result_ptr = js_get_result_ptr();
        let result_len = js_get_result_len();
        
        if result_ptr.is_null() || result_len == 0 {
            return r#"{"success":false,"message":"JavaScript调用失败"}"#.to_string();
        }
        
        // 从指针创建字符串
        let result_bytes = std::slice::from_raw_parts(result_ptr, result_len);
        log_info!("call_js_function result_len: {}", result_len);
        match std::str::from_utf8(result_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => r#"{"success":false,"message":"无效的UTF-8数据"}"#.to_string()
        }
    }
}

// 包装JavaScript调用，用于在WASM环境下调用JavaScript
// 注意：这个函数现在只用于同步调用
fn invoke_js_with_result(js_code: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        // 调用同步JS函数
        let result_code = unsafe {
            let js_code_bytes = js_code.as_bytes();
            js_invoke_string(js_code_bytes.as_ptr(), js_code_bytes.len())
        };
        // 获取结果（无论同步调用成功还是失败，结果都存储在last_js_result中）
        get_js_result()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 非WASM环境下返回空对象
        "{}".to_string()
    }
}

// -- 新增异步调用辅助函数 --

// 启动异步JS调用
#[cfg(target_arch = "wasm32")]
fn start_async_js_call(js_code: &str) -> Result<(), String> {
    log_debug!("准备启动异步JS: {}", js_code);
    unsafe {
        let js_code_bytes = js_code.as_bytes();
        let result_code = js_invoke_async_string(js_code_bytes.as_ptr(), js_code_bytes.len());
        if result_code == 1 {
            log_debug!("异步JS成功启动");
            Ok(())
        } else {
            // 尝试获取错误信息
            let error_message = get_js_result();
            log_error!("启动异步JS失败，错误信息: {}", error_message);
            Err(format!("启动异步JS失败: {}", error_message))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_async_js_call(_js_code: &str) -> Result<(), String> {
    Ok(()) // 非WASM环境下模拟成功
}

// 获取JS调用的结果（可能是空的，表示正在运行）
#[cfg(target_arch = "wasm32")]
fn get_js_result() -> String {
    unsafe {
        let result_ptr = js_get_result_ptr();
        let result_len = js_get_result_len();
        if result_ptr.is_null() || result_len == 0 {
            return "".to_string(); // 返回空表示未就绪或无结果
        }
        let result_bytes = std::slice::from_raw_parts(result_ptr, result_len);
        match std::str::from_utf8(result_bytes) {
            Ok(s) => s.to_string(),
            Err(e) => {
                log_error!("JS返回无效UTF-8数据: {}", e);
                // 返回一个标准的错误JSON
                r#"{"success":false,"message":"无效的UTF-8数据"}"#.to_string()
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_js_result() -> String {
    "{}".to_string() // 非WASM环境下返回模拟空对象
}

// -- 结束新增异步调用辅助函数 --

// 用于存储排行榜数据的结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRank {
    pub user_id: String,
    pub name: String,
    pub score: u32,
    pub rank: u32,
}

// 全局状态
static CLOUD_STATE: Lazy<Mutex<CloudState>> = Lazy::new(|| Mutex::new(CloudState::new()));

// 用于表示云服务状态的枚举
#[derive(Debug, Clone)]
pub enum CloudState {
    NotInitialized,
    Initializing,
    Initialized {
        user_name: String,
        user_id: String,
        player_rank: Option<PlayerRank>,
        top_ranks: Vec<PlayerRank>,
    },
    Error(String),
}

impl CloudState {
    fn new() -> Self {
        CloudState::NotInitialized
    }
}

// SDK 初始化函数
pub async fn init_cloud_service() -> CloudState {
    #[cfg(target_arch = "wasm32")]
    {
        log_info!("开始初始化云服务...");
        
        // 1. 同步调用初始化 SDK
        let init_js = r#"window.sce_init_sdk(window.SCE_CONFIG.developer_token, window.SCE_CONFIG.env || 'pd');"#;
        let init_result_str = invoke_js_with_result(init_js);
        log_info!("SDK 初始化结果 (同步): {}", init_result_str);
        
        // 2. 解析初始化结果
        match serde_json::from_str::<serde_json::Value>(&init_result_str) {
            Ok(json) => {
                if let Some(success) = json.get("success").and_then(|v| v.as_bool()) {
                    if success {
                        log_info!("SDK 初始化成功，尝试异步登录...");
                        
                        // 3. 启动异步登录调用 (login 是异步的)
                        let login_js = r#"window.sce_login();"#;
                        if let Err(e) = start_async_js_call(login_js) {
                             log_error!("启动登录失败: {}", e);
                             // 即使登录启动失败，也认为初始化基本完成，使用访客身份
                             return CloudState::Initialized {
                                user_name: format!("访客_{}", rand::gen_range(100, 999)),
                                user_id: format!("guest_{}", rand::gen_range(10000, 99999)),
                                player_rank: None,
                                top_ranks: Vec::new(),
                             };
                        }

                        // 4. 轮询等待登录结果
                        let login_result_str = loop {
                            let result = get_js_result();
                            if result.is_empty() {
                                next_frame().await;
                                continue;
                            } else {
                                log_info!("登录结果 (轮询): {}", result);
                                break result;
                            }
                        };

                        // 5. 解析登录结果
                        match serde_json::from_str::<serde_json::Value>(&login_result_str) {
                            Ok(login_json) => {
                                if let Some(login_success) = login_json.get("success").and_then(|v| v.as_bool()) {
                                    if login_success {
                                        log_info!("登录成功");
                                        let user_id = login_json.get("user_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(&format!("user_{}", rand::gen_range(10000, 99999)))
                                            .to_string();
                                        let user_name = login_json.get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(&format!("Player_{}", rand::gen_range(100, 999)))
                                            .to_string();
                                        return CloudState::Initialized {
                                            user_name, user_id, player_rank: None, top_ranks: Vec::new(),
                                        };
                                    }
                                }
                                // 登录API调用成功但业务逻辑失败 (e.g., 用户取消)
                                log_warn!("登录未成功完成 (可能用户取消或 API 内部错误): {}", login_result_str);
                            },
                            Err(e) => {
                                log_error!("解析登录结果失败: {}, 原始字符串: {}", e, login_result_str);
                            }
                        }
                        
                        // 登录失败或解析失败，使用匿名用户
                        log_info!("使用访客身份");
                        return CloudState::Initialized {
                            user_name: format!("访客_{}", rand::gen_range(100, 999)),
                            user_id: format!("guest_{}", rand::gen_range(10000, 99999)),
                            player_rank: None, top_ranks: Vec::new(),
                        };
                    }
                }
                // 初始化 API 调用成功但业务逻辑失败
                let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("未知初始化错误");
                log_error!("SDK 初始化失败: {}", message);
                return CloudState::Error(message.to_string());
            },
            Err(e) => {
                log_error!("解析 SDK 初始化结果失败: {}, 原始字符串: {}", e, init_result_str);
                return CloudState::Error(format!("解析初始化结果失败: {}", e));
            }
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log_info!("在非WASM环境中运行，排行榜功能受限");
        CloudState::Initialized {
            user_name: format!("本地玩家_{}", rand::gen_range(100, 999)),
            user_id: "local_user".to_string(),
            player_rank: None,
            top_ranks: Vec::new(),
        }
    }
}

// 提交分数
pub async fn submit_score(score: u32) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let cloud_state = CLOUD_STATE.lock().unwrap().clone();
        match cloud_state {
            CloudState::Initialized { .. } => {
                log_info!("准备提交分数: {}", score);
                let js_code = format!("window.sce_upload_score({})", score);
                start_async_js_call(&js_code)?;

                let result_str = loop {
                    let result = get_js_result();
                    if result.is_empty() {
                        next_frame().await;
                        continue;
                    } else {
                        log_info!("提交分数结果: {}", result);
                        break result;
                    }
                };

                match serde_json::from_str::<serde_json::Value>(&result_str) {
                    Ok(json) => {
                        if let Some(true) = json.get("success").and_then(|v| v.as_bool()) {
                            log_info!("分数提交成功");
                            return Ok(());
                        }
                        let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
                        log_error!("提交分数失败: {}", message);
                        Err(message.to_string())
                    },
                    Err(e) => {
                        log_error!("解析提交分数结果失败: {}, 原始字符串: {}", e, result_str);
                        Err(format!("解析结果失败: {}", e))
                    }
                }
            },
            CloudState::Error(msg) => Err(format!("云服务处于错误状态: {}", msg)),
            _ => Err("云服务未初始化".to_string()),
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log_info!("在非WASM环境中运行，分数不会提交到在线排行榜");
        Ok(())
    }
}

// 获取排行榜数据
pub async fn get_leaderboard(limit: u32) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        log_info!("准备获取排行榜数据 (limit={})", limit);
        let js_code = format!("window.sce_get_leaderboard({})", limit);
        start_async_js_call(&js_code)?;

        let result_str = loop {
            let result = get_js_result();
            if result.is_empty() {
                next_frame().await;
                continue;
            } else {
                log_info!("获取排行榜数据结果: {}", result);
                break result;
            }
        };

        match serde_json::from_str::<serde_json::Value>(&result_str) {
            Ok(json) => {
                if let Some(true) = json.get("success").and_then(|v| v.as_bool()) {
                    if let Some(data) = json.get("data").and_then(|v| v.as_array()) {
                        let mut ranks = Vec::new();
                        for (i, item) in data.iter().enumerate() {
                            let user_id = item.get("user_id").and_then(|v| v.as_str())
                                .unwrap_or("unknown").to_string();
                            let name = item.get("name").and_then(|v| v.as_str())
                                .unwrap_or("未知玩家").to_string();
                            let score = item.get("value").and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            let rank = item.get("rank").and_then(|v| v.as_u64())
                                .unwrap_or((i + 1) as u64) as u32;
                            
                            // log_info!("解析排名: user_id={}, name={}, score={}, rank={}", user_id, name, score, rank);
                            ranks.push(PlayerRank { user_id, name, score, rank });
                        }
                        
                        let mut state = CLOUD_STATE.lock().unwrap();
                        if let CloudState::Initialized { top_ranks: ref mut tr, .. } = *state {
                            *tr = ranks;
                            log_info!("排行榜数据已更新");
                        }
                        return Ok(());
                    } else {
                        log_error!("排行榜数据格式错误: 缺少 'data' 数组");
                        return Err("数据格式错误".to_string());
                    }
                }
                let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
                log_error!("获取排行榜失败: {}", message);
                Err(message.to_string())
            },
            Err(e) => {
                log_error!("解析排行榜数据失败: {}, 原始字符串: {}", e, result_str);
                Err(format!("解析结果失败: {}", e))
            }
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 在非WASM环境中使用模拟数据
        log_info!("使用模拟排行榜数据");
        let ranks = vec![
            PlayerRank { user_id: "test1".to_string(), name: "测试玩家1".to_string(), score: 5000, rank: 1, },
            PlayerRank { user_id: "test2".to_string(), name: "测试玩家2".to_string(), score: 4500, rank: 2, },
            PlayerRank { user_id: "test3".to_string(), name: "测试玩家3".to_string(), score: 4000, rank: 3, }
        ];
        let mut state = CLOUD_STATE.lock().unwrap();
        if let CloudState::Initialized { top_ranks: ref mut tr, .. } = *state {
            *tr = ranks;
        }
        Ok(())
    }
}

// 获取玩家排名
pub async fn get_player_rank() -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let cloud_state = CLOUD_STATE.lock().unwrap().clone();
        match cloud_state {
            CloudState::Initialized { user_name, user_id, .. } => {
                log_info!("准备获取玩家排名 (user_id={})", user_id);
                let js_code = "window.sce_get_user_rank()";
                start_async_js_call(js_code)?;

                let result_str = loop {
                    let result = get_js_result();
                    if result.is_empty() {
                        next_frame().await;
                        continue;
                    } else {
                        log_info!("获取玩家排名结果: {}", result);
                        break result;
                    }
                };

                match serde_json::from_str::<serde_json::Value>(&result_str) {
                    Ok(json) => {
                        if let Some(true) = json.get("success").and_then(|v| v.as_bool()) {
                            let rank = json.get("rank").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                            let score = json.get("score").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                            let result_user_id = json.get("user_id").and_then(|v| v.as_str()).unwrap_or(&user_id).to_string();
                            
                            let player_rank = PlayerRank {
                                user_id: result_user_id,
                                name: user_name.clone(),
                                score,
                                rank,
                            };
                            
                            let mut state = CLOUD_STATE.lock().unwrap();
                            if let CloudState::Initialized { player_rank: ref mut pr, .. } = *state {
                                *pr = Some(player_rank);
                                log_info!("玩家排名已更新");
                            }
                            return Ok(());
                        }
                        let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
                        log_error!("获取玩家排名失败: {}", message);
                        Err(message.to_string())
                    },
                    Err(e) => {
                        log_error!("解析玩家排名结果失败: {}, 原始字符串: {}", e, result_str);
                        Err(format!("解析结果失败: {}", e))
                    }
                }
            },
            CloudState::Error(msg) => Err(format!("云服务处于错误状态: {}", msg)),
            _ => Err("云服务未初始化".to_string()),
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log_info!("使用模拟玩家排名数据");
        let player_rank = PlayerRank { user_id: "local_user".to_string(), name: "本地玩家".to_string(), score: 0, rank: 10, };
        let mut state = CLOUD_STATE.lock().unwrap();
        if let CloudState::Initialized { player_rank: ref mut pr, .. } = *state {
            *pr = Some(player_rank);
        }
        Ok(())
    }
}

// 检查云服务是否已初始化
pub fn is_cloud_initialized() -> bool {
    let state = CLOUD_STATE.lock().unwrap();
    matches!(*state, CloudState::Initialized { .. })
}

// 获取排行榜数据
pub fn get_leaderboard_data() -> (bool, Option<String>, Vec<PlayerRank>, Option<PlayerRank>) {
    let state = CLOUD_STATE.lock().unwrap();
    
    match &*state {
        CloudState::Initialized { top_ranks, player_rank, .. } => {
            (false, None, top_ranks.clone(), player_rank.clone())
        },
        CloudState::Initializing => {
            (true, None, Vec::new(), None)
        },
        CloudState::Error(msg) => {
            (false, Some(msg.clone()), Vec::new(), None)
        },
        CloudState::NotInitialized => {
            (false, Some("云服务未初始化".to_string()), Vec::new(), None)
        }
    }
}

// 在WASM环境中初始化SDK (此函数现在是初始化过程的入口)
#[cfg(target_arch = "wasm32")]
pub async fn initialize_sdk() -> Result<(), String> {
    log_info!("调用 initialize_sdk (入口)");
    // 调用包含轮询逻辑的 init_cloud_service
    let final_state = init_cloud_service().await;

    // 将最终状态更新到全局 CLOUD_STATE
    *CLOUD_STATE.lock().unwrap() = final_state.clone();
    
    // 根据最终状态返回结果
    match final_state {
        CloudState::Initialized { .. } => {
            log_info!("initialize_sdk 完成: 成功 (全局状态已更新)");
            Ok(())
        },
        CloudState::Error(msg) => {
            log_error!("initialize_sdk 完成: 失败 - {} (全局状态已更新)", msg);
            Err(msg)
        },
        _ => {
            log_error!("initialize_sdk 完成: 失败 - 意外状态 (全局状态已更新)");
            Err("初始化失败: 意外状态".to_string())
        }
    }
}

// 为非WebAssembly环境添加initialize_sdk函数实现
#[cfg(not(target_arch = "wasm32"))]
pub async fn initialize_sdk() -> Result<(), String> {
    log_info!("非WASM环境下模拟初始化SDK");
    // 调用已有的init_cloud_service函数
    let final_state = init_cloud_service().await;
    // 更新全局状态
    *CLOUD_STATE.lock().unwrap() = final_state.clone();
    // 返回成功
    Ok(())
}

// 添加别名函数
pub async fn upload_score(score: u32) -> Result<(), String> {
    submit_score(score).await
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn allocate_memory(size: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    // 防止缓冲区被释放
    std::mem::forget(buffer);
    ptr
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn deallocate_memory(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size);
        // 让Vec的析构函数释放内存
    }
}


