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
            
            if x < 0 || x >= 8 || y < 0 || y >= 8 {
                return false;
            }
            
            if self.cells[y as usize][x as usize].is_some() {
                return false;
            }
        }
        true
    }
    
    // 检查是否可以放置方块（带容错范围）
    pub fn can_place_block_with_tolerance(&self, block: &BlockShape, grid_x: i32, grid_y: i32, tolerance: i32) -> (bool, i32, i32) {
        if self.can_place_block(block, grid_x, grid_y) {
            return (true, grid_x, grid_y);
        }
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for &(dx, dy) in &directions {
            let new_x = grid_x + dx;
            let new_y = grid_y + dy;
            if self.can_place_block(block, new_x, new_y) { return (true, new_x, new_y); }
        }
        let diagonals = [(-1, -1), (1, -1), (-1, 1), (1, 1)];
        for &(dx, dy) in &diagonals {
            let new_x = grid_x + dx;
            let new_y = grid_y + dy;
            if self.can_place_block(block, new_x, new_y) { return (true, new_x, new_y); }
        }
        if tolerance > 1 {
             for dy_offset in -tolerance..=tolerance {
                for dx_offset in -tolerance..=tolerance {
                    if (dx_offset.abs() <= 1 && dy_offset.abs() <= 1) || (dx_offset == 0 && dy_offset == 0) {
                        continue;
                    }
                    let new_x = grid_x + dx_offset;
                    let new_y = grid_y + dy_offset;
                    if self.can_place_block(block, new_x, new_y) {
                        return (true, new_x, new_y);
                    }
                }
            }
        }
        (false, grid_x, grid_y)
    }
    
    // 放置方块
    pub fn place_block(&mut self, block: &BlockShape, grid_x: i32, grid_y: i32) {
        for &(dx, dy) in &block.cells {
            let x = grid_x + dx;
            let y = grid_y + dy;
            if x >= 0 && x < 8 && y >=0 && y < 8 { 
                 self.cells[y as usize][x as usize] = Some(block.color);
            }
        }
    }
    
    // 检查并消除填满的行和列，返回清除的行和列的索引
    pub fn check_and_clear(&mut self) -> (Vec<usize>, Vec<usize>) {
        let mut rows_to_clear_indices = Vec::new();
        let mut cols_to_clear_indices = Vec::new();
        
        // 1. 标记要清除的行索引
        for y_idx in 0..8 {
            if (0..8).all(|x_idx| self.cells[y_idx][x_idx].is_some()) { 
                rows_to_clear_indices.push(y_idx);
            }
        }
        
        // 2. 标记要清除的列索引
        for x_idx in 0..8 {
            if (0..8).all(|y_idx| self.cells[y_idx][x_idx].is_some()) {
                cols_to_clear_indices.push(x_idx);
            }
        }

        if rows_to_clear_indices.is_empty() && cols_to_clear_indices.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let mut should_clear_cell = [[false; 8]; 8];
        for &y_idx in &rows_to_clear_indices {
            for x_idx in 0..8 {
                 should_clear_cell[y_idx][x_idx] = true;
            }
        }
        for &x_idx in &cols_to_clear_indices {
            for y_idx in 0..8 {
                should_clear_cell[y_idx][x_idx] = true;
            }
        }

        for y in 0..8 {
            for x in 0..8 {
                if should_clear_cell[y][x] { 
                    self.cells[y][x] = None;
                }
            }
        }
        
        (rows_to_clear_indices, cols_to_clear_indices)
    }
    
    // 绘制网格和方块
    pub fn draw_with_highlights(
        &self, 
        offset_x: f32, 
        offset_y: f32, 
        cell_size: f32, 
        active_targets: Option<&Vec<usize>>, // 目标行/列的索引
        is_target_rows: bool                // 如果 active_targets is Some, true 表示目标是行, false 表示列
    ) {
        let highlight_color = Color::new(1.0, 1.0, 0.0, 0.3); // 淡黄色半透明高亮

        for y_idx in 0..8 {
            for x_idx in 0..8 {
                let pos_x = offset_x + x_idx as f32 * cell_size;
                let pos_y = offset_y + y_idx as f32 * cell_size;
                
                // 检查是否需要高亮当前单元格
                let mut should_highlight = false;
                if let Some(targets) = active_targets {
                    if is_target_rows {
                        if targets.contains(&y_idx) {
                            should_highlight = true;
                        }
                    } else { // is_target_cols
                        if targets.contains(&x_idx) {
                            should_highlight = true;
                        }
                    }
                }

                // 如果需要高亮，先绘制高亮背景
                if should_highlight {
                    draw_rectangle(
                        pos_x, 
                        pos_y, 
                        cell_size, 
                        cell_size, 
                        highlight_color
                    );
                }

                // 绘制网格线
                draw_rectangle_lines(pos_x, pos_y, cell_size, cell_size, 1.0, BLACK);
                
                // 绘制已放置的方块 (恢复为 Option<Color>)
                if let Some(color) = self.cells[y_idx][x_idx] {
                    crate::drawing::draw_cube_block(pos_x, pos_y, cell_size, color);
                }
            }
        }
    }
} 