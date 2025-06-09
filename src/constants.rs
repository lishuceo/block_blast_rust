use macroquad::prelude::{Color, WHITE, BLACK, GRAY, LIGHTGRAY, DARKGRAY, GREEN, SKYBLUE};

// 全局颜色常量 - 基于 #3C569E 的配色方案
pub const COLOR_PRIMARY: Color = Color { r: 0.235, g: 0.337, b: 0.62, a: 1.0 };         // 主色 #3C569E
pub const COLOR_PRIMARY_DARK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 };   // 主色30%亮度，用于网格区域
pub const COLOR_PRIMARY_OVERLAY: Color = Color { r: 0.118, g: 0.169, b: 0.31, a: 0.9 }; // 主色50%亮度，用于半透明覆盖层
pub const COLOR_BORDER: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.95 };               // 边框色 #050505
pub const COLOR_TITLE: Color = Color { r: 1.0, g: 0.4, b: 0.2, a: 1.0 };                // 标题色

// 新增的颜色常量
pub const GOLD: Color = Color::new(1.0, 0.843, 0.0, 1.0);      // 金色 #FFD700
pub const ORANGE: Color = Color::new(1.0, 0.647, 0.0, 1.0);    // 橙色 #FFA500

// pub const SKYBLUE: Color = Color { r: 0.5, g: 0.7, b: 1.0, a: 1.0 }; // Already imported from prelude
// 直接使用导入的常量
// pub const WHITE: Color = Color::WHITE;
// pub const BLACK: Color = Color::BLACK;
// pub const GRAY: Color = Color::GRAY;
// pub const LIGHTGRAY: Color = Color::LIGHTGRAY;
// pub const DARKGRAY: Color = Color::DARKGRAY;
// pub const GREEN: Color = Color::GREEN; 