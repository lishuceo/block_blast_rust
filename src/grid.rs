// 网格模块，处理方块放置和消除逻辑
use macroquad::prelude::*;
use crate::block::BlockShape;

// 更高效的立体感方块绘制函数
fn draw_cube_block(x: f32, y: f32, size: f32, color: Color) {
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

pub struct Grid {
    pub cells: [[Option<Color>; 8]; 8],
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            cells: [[None; 8]; 8],
        }
    }
    
    // 检查是否可以放置方块
    pub fn can_place_block(&self, block: &BlockShape, grid_x: i32, grid_y: i32) -> bool {
        for &(dx, dy) in &block.cells {
            let x = grid_x + dx;
            let y = grid_y + dy;
            
            // 检查边界
            if x < 0 || x >= 8 || y < 0 || y >= 8 {
                return false;
            }
            
            // 检查是否已被占用
            if self.cells[y as usize][x as usize].is_some() {
                return false;
            }
        }
        true
    }
    
    // 检查是否可以放置方块（带容错范围）
    pub fn can_place_block_with_tolerance(&self, block: &BlockShape, grid_x: i32, grid_y: i32, tolerance: i32) -> (bool, i32, i32) {
        // 首先尝试在原位置放置
        if self.can_place_block(block, grid_x, grid_y) {
            return (true, grid_x, grid_y);
        }
        
        // 如果原位置不行，先尝试上下左右四个方向
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)]; // 上、下、左、右
        for &(dx, dy) in &directions {
            let new_x = grid_x + dx;
            let new_y = grid_y + dy;
            
            if self.can_place_block(block, new_x, new_y) {
                return (true, new_x, new_y);
            }
        }
        
        // 如果上下左右都不行，再尝试对角线方向
        let diagonals = [(-1, -1), (1, -1), (-1, 1), (1, 1)]; // 左上、右上、左下、右下
        for &(dx, dy) in &diagonals {
            let new_x = grid_x + dx;
            let new_y = grid_y + dy;
            
            if self.can_place_block(block, new_x, new_y) {
                return (true, new_x, new_y);
            }
        }
        
        // 如果容错范围大于1，尝试更远的位置
        if tolerance > 1 {
            for dy in -tolerance..=tolerance {
                for dx in -tolerance..=tolerance {
                    // 跳过已经检查过的位置（原始位置、上下左右和对角线）
                    if (dx.abs() <= 1 && dy.abs() <= 1) || (dx == 0 && dy == 0) {
                        continue;
                    }
                    
                    let new_x = grid_x + dx;
                    let new_y = grid_y + dy;
                    
                    if self.can_place_block(block, new_x, new_y) {
                        return (true, new_x, new_y);
                    }
                }
            }
        }
        
        // 如果所有位置都不行，返回原始位置和失败标志
        (false, grid_x, grid_y)
    }
    
    // 放置方块
    pub fn place_block(&mut self, block: &BlockShape, grid_x: i32, grid_y: i32) {
        for &(dx, dy) in &block.cells {
            let x = grid_x + dx;
            let y = grid_y + dy;
            self.cells[y as usize][x as usize] = Some(block.color);
        }
    }
    
    // 检查并消除填满的行和列 (只有完全填满才消除)
    pub fn check_and_clear(&mut self) -> (u32, u32) {
        let mut rows_cleared = 0;
        let mut cols_cleared = 0;
        
        // 检查行
        for y in 0..8 {
            if (0..8).all(|x| self.cells[y][x].is_some()) {
                // 清除这一行
                for x in 0..8 {
                    self.cells[y][x] = None;
                }
                rows_cleared += 1;
            }
        }
        
        // 检查列
        for x in 0..8 {
            if (0..8).all(|y| self.cells[y][x].is_some()) {
                // 清除这一列
                for y in 0..8 {
                    self.cells[y][x] = None;
                }
                cols_cleared += 1;
            }
        }
        
        (rows_cleared, cols_cleared)
    }
    
    // 绘制网格和方块
    pub fn draw(&self, offset_x: f32, offset_y: f32, cell_size: f32) {
        for y in 0..8 {
            for x in 0..8 {
                let pos_x = offset_x + x as f32 * cell_size;
                let pos_y = offset_y + y as f32 * cell_size;
                
                // 绘制网格线 - 改为黑色
                draw_rectangle_lines(pos_x, pos_y, cell_size, cell_size, 1.0, BLACK);
                
                // 绘制已放置的方块
                if let Some(color) = self.cells[y][x] {
                    // 使用draw_cube_block函数绘制方块（包含3D效果）
                    draw_cube_block(pos_x, pos_y, cell_size, color);
                }
            }
        }
    }
} 