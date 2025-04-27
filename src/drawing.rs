use macroquad::prelude::*;

// 更高效的立体感方块绘制函数
pub fn draw_cube_block(x: f32, y: f32, size: f32, color: Color) {
    // 亮色和暗色偏移量
    let light_factor = 0.4;
    let dark_factor = 0.4;
    let mid_light_factor = 0.2; // 中等亮色偏移量
    let mid_dark_factor = 0.2; // 中等暗色偏移量
    
    // 边缘厚度
    let border = size * 0.15;
    
    // 创建亮色和暗色
    let light_color = Color::new(
        (color.r + light_factor).min(1.0),
        (color.g + light_factor).min(1.0),
        (color.b + light_factor).min(1.0),
        color.a
    );
    
    // 创建中等亮色（比light暗）
    let mid_light_color = Color::new(
        (color.r + mid_light_factor).min(1.0),
        (color.g + mid_light_factor).min(1.0),
        (color.b + mid_light_factor).min(1.0),
        color.a
    );
    
    // 创建中等暗色（比dark浅）
    let mid_dark_color = Color::new(
        (color.r - mid_dark_factor).max(0.0),
        (color.g - mid_dark_factor).max(0.0),
        (color.b - mid_dark_factor).max(0.0),
        color.a
    );
    
    let dark_color = Color::new(
        (color.r - dark_factor).max(0.0),
        (color.g - dark_factor).max(0.0),
        (color.b - dark_factor).max(0.0),
        color.a
    );
    
    // 1. 先绘制主体
    draw_rectangle(x, y, size, size, color);
    
    // 2. 绘制四个边（只需要4次绘制调用）
    // 上边 - 亮色
    draw_triangle(
        Vec2::new(x, y), 
        Vec2::new(x + size, y), 
        Vec2::new(x + size - border, y + border),
        light_color
    );
    draw_triangle(
        Vec2::new(x, y), 
        Vec2::new(x + border, y + border), 
        Vec2::new(x + size - border, y + border),
        light_color
    );
    
    // 左边 - 中等亮色
    draw_triangle(
        Vec2::new(x, y), 
        Vec2::new(x, y + size), 
        Vec2::new(x + border, y + size - border),
        mid_light_color
    );
    draw_triangle(
        Vec2::new(x, y), 
        Vec2::new(x + border, y + border), 
        Vec2::new(x + border, y + size - border),
        mid_light_color
    );
    
    // 右边 - 中等暗色
    draw_triangle(
        Vec2::new(x + size, y), 
        Vec2::new(x + size, y + size), 
        Vec2::new(x + size - border, y + size - border),
        mid_dark_color
    );
    draw_triangle(
        Vec2::new(x + size, y), 
        Vec2::new(x + size - border, y + border), 
        Vec2::new(x + size - border, y + size - border),
        mid_dark_color
    );
    
    // 下边 - 暗色
    draw_triangle(
        Vec2::new(x, y + size), 
        Vec2::new(x + size, y + size), 
        Vec2::new(x + size - border, y + size - border),
        dark_color
    );
    draw_triangle(
        Vec2::new(x, y + size), 
        Vec2::new(x + border, y + size - border), 
        Vec2::new(x + size - border, y + size - border),
        dark_color
    );
} 