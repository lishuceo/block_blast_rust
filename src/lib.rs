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
// pub mod random; // 已废弃，使用 macroquad::rand 代替
pub mod drawing;
pub mod constants;
pub mod utils;
pub mod wave;
 
// 注意：此库仅使用macroquad处理WASM导出 
// 此版本中不使用wasm-bindgen 

// 移除重复的 GameMode use 和 module
// pub use game_logic::GameMode;
// mod game_logic { 
//     #[derive(Debug, PartialEq, Copy, Clone)]
//     pub enum GameMode {
//         Easy,
//         Normal,
//         Happy,
//     }
// }
