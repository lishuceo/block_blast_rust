use macroquad::prelude::{screen_width, screen_height};

// 获取设备DPI缩放比例的辅助函数
pub fn get_dpi_scale() -> f32 {
    #[cfg(target_os = "ios")]
    {
        // iOS设备通常有更高的像素密度，使用更高的缩放比
        // iPhone的Retina显示屏通常是2x或3x缩放
        return 3.0;
    }
    
    #[cfg(target_os = "android")]
    {
        // Android设备根据屏幕密度调整
        // 通常为1.5x到4x之间
        return 2.0;
    }
    
    // 桌面平台，根据实际DPI动态计算
    // macroquad没有直接提供获取系统DPI的API，所以我们使用推断
    let (w, h) = (screen_width(), screen_height());
    if w > 2000.0 || h > 2000.0 {
        // 4K或高分辨率显示器
        2.0
    } else if w > 1200.0 || h > 1200.0 {
        // 高清显示器
        1.5
    } else {
        // 标准显示器
        1.0
    }
}

// 平滑过渡辅助函数
pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = f32::max(0.0, f32::min(1.0, (x - edge0) / (edge1 - edge0)));
    t * t * (3.0 - 2.0 * t) // 平滑的三次函数
} 