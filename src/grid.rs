// 网格模块，处理方块放置和消除逻辑
use macroquad::prelude::*;
use crate::block::BlockShape;

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
    
    // 检查并消除填满的行和列 (修复同时满足行列时的Bug)
    pub fn check_and_clear(&mut self) -> (u32, u32) {
        let mut rows_to_clear = [false; 8];
        let mut cols_to_clear = [false; 8];
        
        // 1. 标记要清除的行
        for y in 0..8 {
            if (0..8).all(|x| self.cells[y][x].is_some()) {
                rows_to_clear[y] = true;
            }
        }
        
        // 2. 标记要清除的列
        for x in 0..8 {
            if (0..8).all(|y| self.cells[y][x].is_some()) {
                cols_to_clear[x] = true;
            }
        }

        // 3. 计算清除的数量 (在实际清除前计算)
        let rows_cleared = rows_to_clear.iter().filter(|&&clear| clear).count() as u32;
        let cols_cleared = cols_to_clear.iter().filter(|&&clear| clear).count() as u32;

        // 如果没有需要清除的，直接返回
        if rows_cleared == 0 && cols_cleared == 0 {
            return (0, 0);
        }

        // 4. 执行清除
        for y in 0..8 {
            for x in 0..8 {
                if rows_to_clear[y] || cols_to_clear[x] {
                    self.cells[y][x] = None;
                }
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
                    crate::drawing::draw_cube_block(pos_x, pos_y, cell_size, color);
                }
            }
        }
    }
} 