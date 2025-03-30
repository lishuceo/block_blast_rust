// 保存和加载游戏数据
use macroquad::prelude::*;

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
        // 简化版本：我们暂时不实际保存
        // 由于WASM环境中存储方式可能不同，这里只是定义接口
        println!("保存最高分: {}", self.high_score);
    }
    
    pub fn load() -> Self {
        // 简化版本：我们暂时总是返回默认数据
        // 实际应用中，可以从localStorage (Web)或本地文件系统加载
        SaveData::new()
    }
} 