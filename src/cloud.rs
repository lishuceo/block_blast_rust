use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde_json;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// 创建一个简单的Future实现，用于等待指定秒数
struct WaitSeconds(f32);

impl Future for WaitSeconds {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.0 <= 0.0 {
            Poll::Ready(())
        } else {
            this.0 -= get_frame_time();
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// 简单模拟wait_seconds函数
async fn wait_seconds(seconds: f32) {
    WaitSeconds(seconds).await
}

// 声明JavaScript外部函数
#[cfg(target_arch = "wasm32")]
extern "C" {
    // 声明JavaScript中定义的函数
    fn js_invoke_string(js_code_ptr: *const u8, js_code_len: usize) -> i32;
    fn js_get_result_ptr() -> *const u8;
    fn js_get_result_len() -> usize;
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
        match std::str::from_utf8(result_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => r#"{"success":false,"message":"无效的UTF-8数据"}"#.to_string()
        }
    }
}

// 包装JavaScript调用，用于在WASM环境下调用JavaScript
fn invoke_js_with_result(js_code: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        call_js_function(js_code)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 非WASM环境下返回空对象
        "{}".to_string()
    }
}

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
        // 移除自定义等待，因为JS端已有加载处理逻辑
        // wait_seconds(0.5).await;
        
        // 使用SCE SDK初始化
        let js_code = r#"window.sce_init_sdk(window.SCE_CONFIG.developer_token, window.SCE_CONFIG.env || 'pd');"#;
        
        let result = invoke_js_with_result(js_code);
        
        // 解析初始化结果
        match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(json) => {
                if let Some(success) = json.get("success").and_then(|v| v.as_bool()) {
                    if success {
                        // 初始化成功，尝试登录用户
                        let login_js = r#"window.sce_login();"#;
                        let login_result = invoke_js_with_result(login_js);
                        
                        match serde_json::from_str::<serde_json::Value>(&login_result) {
                            Ok(login_json) => {
                                if let Some(login_success) = login_json.get("success").and_then(|v| v.as_bool()) {
                                    if login_success {
                                        // 登录成功，获取用户信息
                                        let user_id = login_json.get("userId")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(&format!("user_{}", rand::gen_range(10000, 99999)))
                                            .to_string();
                                            
                                        let user_name = login_json.get("userName")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(&format!("Player_{}", rand::gen_range(100, 999)))
                                            .to_string();
                                        
                                        return CloudState::Initialized {
                                            user_name,
                                            user_id,
                                            player_rank: None,
                                            top_ranks: Vec::new(),
                                        };
                                    }
                                }
                            },
                            Err(e) => {
                                return CloudState::Error(format!("登录失败: {}", e));
                            }
                        }
                        
                        // 登录失败，使用匿名用户
                        return CloudState::Initialized {
                            user_name: format!("访客_{}", rand::gen_range(100, 999)),
                            user_id: format!("guest_{}", rand::gen_range(10000, 99999)),
                            player_rank: None,
                            top_ranks: Vec::new(),
                        };
                    }
                }
                
                // 初始化失败，返回错误
                if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                    return CloudState::Error(message.to_string());
                }
            },
            Err(e) => {
                return CloudState::Error(format!("初始化SCE SDK失败: {}", e));
            }
        }
        
        CloudState::Error("初始化SCE SDK失败".to_string())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("在非WASM环境中运行，排行榜功能受限");
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
        // 确保云服务已初始化
        let cloud_state = CLOUD_STATE.lock().unwrap().clone();
        
        match cloud_state {
            CloudState::Initialized { .. } => {
                // 使用SCE SDK上传分数
                let js_code = format!("window.sce_upload_score({})", score);
                
                let result = invoke_js_with_result(&js_code);
                
                match serde_json::from_str::<serde_json::Value>(&result) {
                    Ok(json) => {
                        if let Some(success) = json.get("success").and_then(|v| v.as_bool()) {
                            if success {
                                return Ok(());
                            }
                        }
                        
                        // 如果有错误信息，返回错误信息
                        if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                            return Err(message.to_string());
                        }
                    },
                    Err(e) => {
                        return Err(format!("解析上传分数结果失败: {}", e));
                    }
                }
                
                Err("上传分数失败".to_string())
            },
            CloudState::Error(msg) => Err(format!("云服务处于错误状态: {}", msg)),
            _ => Err("云服务未初始化".to_string()),
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("在非WASM环境中运行，分数不会提交到在线排行榜");
        Ok(())
    }
}

// 获取排行榜数据
pub async fn get_leaderboard(limit: u32) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        // 获取排行榜数据
        let js_code = format!("window.sce_get_leaderboard({})", limit);
        let result = invoke_js_with_result(&js_code);
        
        match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(json) => {
                if let Some(success) = json.get("success").and_then(|v| v.as_bool()) {
                    if success {
                        if let Some(data) = json.get("data").and_then(|v| v.as_array()) {
                            let mut ranks = Vec::new();
                            
                            for (i, item) in data.iter().enumerate() {
                                let user_id = item.get("user_id").and_then(|v| v.as_str())
                                    .unwrap_or("unknown").to_string();
                                let name = item.get("user_name").and_then(|v| v.as_str())
                                    .unwrap_or("未知玩家").to_string();
                                let score = item.get("value").and_then(|v| v.as_u64())
                                    .unwrap_or(0) as u32;
                                let rank = item.get("rank").and_then(|v| v.as_u64())
                                    .unwrap_or((i + 1) as u64) as u32;
                                
                                ranks.push(PlayerRank {
                                    user_id,
                                    name,
                                    score,
                                    rank,
                                });
                            }
                            
                            // 更新全局状态
                            let mut state = CLOUD_STATE.lock().unwrap();
                            if let CloudState::Initialized { top_ranks: ref mut tr, .. } = *state {
                                *tr = ranks;
                            }
                            
                            return Ok(());
                        }
                    }
                    
                    // 如果有错误信息，返回错误信息
                    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                        return Err(message.to_string());
                    }
                }
            },
            Err(e) => {
                return Err(format!("解析排行榜数据失败: {}", e));
            }
        }
        
        Err("获取排行榜数据失败".to_string())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 在非WASM环境中使用模拟数据
        let ranks = vec![
            PlayerRank {
                user_id: "test1".to_string(),
                name: "测试玩家1".to_string(),
                score: 5000,
                rank: 1,
            },
            PlayerRank {
                user_id: "test2".to_string(),
                name: "测试玩家2".to_string(),
                score: 4500,
                rank: 2,
            },
            PlayerRank {
                user_id: "test3".to_string(),
                name: "测试玩家3".to_string(),
                score: 4000,
                rank: 3,
            }
        ];
        
        // 更新全局状态
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
                // 使用SCE SDK获取玩家排名
                let js_code = "window.sce_get_user_rank()";
                
                let result = invoke_js_with_result(js_code);
                
                match serde_json::from_str::<serde_json::Value>(&result) {
                    Ok(json) => {
                        if let Some(success) = json.get("success").and_then(|v| v.as_bool()) {
                            if success {
                                // 从结果中提取数据
                                let rank = json.get("rank").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let score = json.get("score").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                let result_user_id = json.get("userId").and_then(|v| v.as_str()).unwrap_or(&user_id).to_string();
                                
                                let player_rank = PlayerRank {
                                    user_id: result_user_id,
                                    name: user_name.clone(),
                                    score,
                                    rank,
                                };
                                
                                // 更新全局状态
                                let mut state = CLOUD_STATE.lock().unwrap();
                                if let CloudState::Initialized { player_rank: ref mut pr, .. } = *state {
                                    *pr = Some(player_rank);
                                }
                                
                                return Ok(());
                            }
                        }
                        
                        // 如果有错误信息，返回错误信息
                        if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                            return Err(message.to_string());
                        }
                    },
                    Err(e) => {
                        return Err(format!("解析玩家排名结果失败: {}", e));
                    }
                }
                
                Err("获取玩家排名失败".to_string())
            },
            CloudState::Error(msg) => Err(format!("云服务处于错误状态: {}", msg)),
            _ => Err("云服务未初始化".to_string()),
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 在非WASM环境中使用模拟数据
        let player_rank = PlayerRank {
            user_id: "local_user".to_string(),
            name: "本地玩家".to_string(),
            score: 0,
            rank: 10,
        };
        
        // 更新全局状态
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

// 添加与main.rs兼容的函数
pub async fn initialize_sdk() -> Result<(), String> {
    let state = init_cloud_service().await;
    match state {
        CloudState::Initialized { .. } => {
            // 更新全局状态
            *CLOUD_STATE.lock().unwrap() = state;
            Ok(())
        },
        CloudState::Error(msg) => Err(msg),
        _ => Err("初始化失败".to_string())
    }
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