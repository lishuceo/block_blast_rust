// 保存和加载游戏数据
use macroquad::prelude::*;

// 导入宏
use crate::{log_debug, log_info, log_warn, log_error};

#[derive(Debug, Clone)]
pub struct SaveData {
    pub high_score: u32,
}

impl SaveData {
    pub fn new() -> Self {
        SaveData {
            high_score: 0,
        }
    }
    
    pub fn save(&self) {
        // 简单的存档实现 - 没有实际持久化
        log_info!("保存最高分: {}", self.high_score);
        // TODO: 实现真正的存档功能
    }
    
    pub fn load() -> Self {
        // 简单的加载实现 - 没有实际持久化
        SaveData {
            high_score: 0,
        }
    }
} 