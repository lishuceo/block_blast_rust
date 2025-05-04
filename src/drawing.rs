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

// 绘制基本圆角矩形
pub fn draw_rounded_rectangle(x: f32, y: f32, width: f32, height: f32, radius: f32, color: Color) {
    // 半径不能超过宽度或高度的一半
    let radius = radius.min(width / 2.0).min(height / 2.0);

    // 绘制中间矩形（水平方向）
    draw_rectangle(x + radius, y, width - radius * 2.0, height, color);

    // 绘制中间矩形（垂直方向）- 修复过度绘制问题
    // draw_rectangle(x, y + radius, width, height - radius * 2.0, color);
    // 只绘制左右两侧的填充矩形，避免中心区域重复绘制
    draw_rectangle(x, y + radius, radius, height - radius * 2.0, color);
    draw_rectangle(x + width - radius, y + radius, radius, height - radius * 2.0, color);


    // 绘制四个角落的圆弧（使用三角形模拟扇形）
    let segments = 16; // 圆角平滑度

    // 左上角 (180 to 270 degrees)
    draw_filled_arc(x + radius, y + radius, radius, 180.0, 270.0, segments, color);

    // 右上角 (270 to 360 degrees)
    draw_filled_arc(x + width - radius, y + radius, radius, 270.0, 360.0, segments, color);

    // 左下角 (90 to 180 degrees)
    draw_filled_arc(x + radius, y + height - radius, radius, 90.0, 180.0, segments, color);

    // 右下角 (0 to 90 degrees)
    draw_filled_arc(x + width - radius, y + height - radius, radius, 0.0, 90.0, segments, color);
}

// 绘制圆角矩形轮廓
pub fn draw_rounded_rectangle_lines(x: f32, y: f32, width: f32, height: f32, radius: f32, thickness: f32, color: Color) {
    // 半径不能超过宽度或高度的一半
    let radius = radius.min(width / 2.0).min(height / 2.0);
    
    // 绘制上下两条水平线
    draw_line(x + radius, y, x + width - radius, y, thickness, color);
    draw_line(x + radius, y + height, x + width - radius, y + height, thickness, color);
    
    // 绘制左右两条垂直线
    draw_line(x, y + radius, x, y + height - radius, thickness, color);
    draw_line(x + width, y + radius, x + width, y + height - radius, thickness, color);
    
    // 绘制四个角落的圆弧
    // 左上角
    draw_arc_lines(x + radius, y + radius, radius, 180.0, 270.0, thickness, color);
    
    // 右上角
    draw_arc_lines(x + width - radius, y + radius, radius, 270.0, 360.0, thickness, color);
    
    // 左下角
    draw_arc_lines(x + radius, y + height - radius, radius, 90.0, 180.0, thickness, color);
    
    // 右下角
    draw_arc_lines(x + width - radius, y + height - radius, radius, 0.0, 90.0, thickness, color);
}

// 绘制带阴影的圆角矩形
pub fn draw_rounded_rectangle_with_shadow(x: f32, y: f32, width: f32, height: f32, radius: f32, color: Color, shadow_offset: f32, shadow_color: Color) {
    // 先画阴影
    draw_rounded_rectangle(
        x + shadow_offset, 
        y + shadow_offset, 
        width, 
        height, 
        radius, 
        shadow_color
    );
    
    // 再画主矩形
    draw_rounded_rectangle(x, y, width, height, radius, color);
}

// 绘制带边框的圆角矩形
pub fn draw_rounded_rectangle_with_border(x: f32, y: f32, width: f32, height: f32, radius: f32, thickness: f32, fill_color: Color, border_color: Color) {
    // 先画填充
    draw_rounded_rectangle(x, y, width, height, radius, fill_color);
    
    // 再画边框
    draw_rounded_rectangle_lines(x, y, width, height, radius, thickness, border_color);
}

// 绘制扇形弧线（辅助函数）
fn draw_arc_lines(center_x: f32, center_y: f32, radius: f32, start_angle: f32, end_angle: f32, thickness: f32, color: Color) {
    // 确定弧线的分段数
    let segments = 16;
    let angle_step = (end_angle - start_angle) / segments as f32;
    
    // 将角度转换为弧度
    let start_rad = start_angle * std::f32::consts::PI / 180.0;
    
    // 绘制分段弧线
    for i in 0..segments {
        let angle1 = start_rad + angle_step * i as f32 * std::f32::consts::PI / 180.0;
        let angle2 = start_rad + angle_step * (i + 1) as f32 * std::f32::consts::PI / 180.0;
        
        let x1 = center_x + radius * angle1.cos();
        let y1 = center_y + radius * angle1.sin();
        let x2 = center_x + radius * angle2.cos();
        let y2 = center_y + radius * angle2.sin();
        
        draw_line(x1, y1, x2, y2, thickness, color);
    }
}

// 新增：绘制填充扇形（辅助函数 for filled rectangle corners）
fn draw_filled_arc(center_x: f32, center_y: f32, radius: f32, start_angle: f32, end_angle: f32, segments: i32, color: Color) {
    let angle_step = (end_angle - start_angle) / segments as f32;

    // 将角度转换为弧度
    let start_rad = start_angle.to_radians();

    let mut p1 = Vec2::new(center_x + radius * start_rad.cos(), center_y + radius * start_rad.sin());

    // 绘制分段三角形
    for i in 1..=segments {
        let current_angle = start_rad + (angle_step * i as f32).to_radians();
        let p2 = Vec2::new(center_x + radius * current_angle.cos(), center_y + radius * current_angle.sin());
        let center = Vec2::new(center_x, center_y);

        draw_triangle(center, p1, p2, color);

        p1 = p2; // 更新下一个三角形的起始点
    }
}

// 绘制带3D效果的圆角矩形
pub fn draw_rounded_rectangle_3d(x: f32, y: f32, width: f32, height: f32, radius: f32, color: Color, depth: f32) {
    // 基本的圆角矩形
    draw_rounded_rectangle(x, y, width, height, radius, color);
    
    // 亮色和暗色偏移量
    let light_factor = 0.4;
    let dark_factor = 0.4;
    
    // 创建亮色和暗色
    let light_color = Color::new(
        (color.r + light_factor).min(1.0),
        (color.g + light_factor).min(1.0),
        (color.b + light_factor).min(1.0),
        color.a
    );
    
    let dark_color = Color::new(
        (color.r - dark_factor).max(0.0),
        (color.g - dark_factor).max(0.0),
        (color.b - dark_factor).max(0.0),
        color.a
    );
    
    // 绘制顶部高光
    draw_rounded_rectangle_lines(x, y, width, height, radius, depth, light_color);
    
    // 绘制底部阴影
    draw_rounded_rectangle_lines(x, y + depth, width, height, radius, depth, dark_color);
} 