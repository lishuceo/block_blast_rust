// 方块消除游戏库 
// 导出所有模块以供main.rs使用 

// --- 公共类型定义 ---
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GameMode {
    Easy,
    Normal,
    Happy,
}

// --- 模块导出 ---
// 仅使用模块导出 
pub mod block; 
pub mod grid; 
pub mod save; 
pub mod effects;
pub mod cloud;
pub mod log;
pub mod random;
pub mod drawing;
 
// 注意：此库仅使用macroquad处理WASM导出 
// 此版本中不使用wasm-bindgen 
