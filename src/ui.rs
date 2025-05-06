game.title_bounce = (game.animation_time * 2.0).sin() * 5.0;
}

// 平滑过渡辅助函数
// pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
//     let t = f32::max(0.0, f32::min(1.0, (x - edge0) / (edge1 - edge0)));
//     t * t * (3.0 - 2.0 * t) // 平滑的三次函数
// }

// 新增绘制主菜单界面的函数
pub fn draw_main_menu(game: &mut Game) {
// ... existing code ...
} 