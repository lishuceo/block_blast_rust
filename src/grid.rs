// 网格模块，处理方块放置和消除逻辑
use macroquad::prelude::*;
use crate::block::{self, BlockShape};

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

    /// 获取接近完成的行和列的索引
    /// cells_needed_to_complete: 还差多少个格子就完成
    /// 返回 (差一点就完成的行索引列表, 差一点就完成的列索引列表)
    pub fn get_almost_complete_lines(&self, cells_needed_to_complete: usize) -> (Vec<usize>, Vec<usize>) {
        let mut almost_complete_rows = Vec::new();
        let mut almost_complete_cols = Vec::new();

        // 检查行
        for r_idx in 0..8 {
            let mut empty_count = 0;
            for c_idx in 0..8 {
                if self.cells[r_idx][c_idx].is_none() {
                    empty_count += 1;
                }
            }
            if empty_count == cells_needed_to_complete {
                almost_complete_rows.push(r_idx);
            }
        }

        // 检查列
        for c_idx in 0..8 {
            let mut empty_count = 0;
            for r_idx in 0..8 {
                if self.cells[r_idx][c_idx].is_none() {
                    empty_count += 1;
                }
            }
            if empty_count == cells_needed_to_complete {
                almost_complete_cols.push(c_idx);
            }
        }
        (almost_complete_rows, almost_complete_cols)
    }

    /// 获取网格填充比例 (0.0 到 1.0)
    pub fn get_filled_ratio(&self) -> f32 {
        let mut filled_count = 0;
        for r_idx in 0..8 {
            for c_idx in 0..8 {
                if self.cells[r_idx][c_idx].is_some() {
                    filled_count += 1;
                }
            }
        }
        filled_count as f32 / 64.0
    }

    /// 查找可以放置在当前棋盘空格子上的特定小形状。
    /// max_cells_to_fill: 候选方块的最大格子数 (例如 1 或 2)。
    /// target_shapes: 一个包含基础形状定义（&'static [(i32, i32)])的切片。
    pub fn find_placeable_shapes_for_empty_spots(
        &self,
        max_cells_to_fill: usize,
        target_shapes: &[&'static [(i32, i32)]],
    ) -> Vec<BlockShape> {
        let mut candidate_blocks = Vec::new();
        if max_cells_to_fill == 0 || target_shapes.is_empty() {
            return candidate_blocks;
        }

        for r in 0..8 {
            for c in 0..8 {
                if self.cells[r][c].is_none() { // 找到一个空格子 (棋盘坐标 c, r)
                    for base_shape_ref in target_shapes {
                        if base_shape_ref.len() == 0 || base_shape_ref.len() > max_cells_to_fill {
                            continue; // 跳过不符合格子数要求的原始形状
                        }

                        let mut current_rotated_cells = base_shape_ref.to_vec();
                        for _rotation_idx in 0..4 { // 尝试0, 90, 180, 270度旋转
                            // 标准化当前旋转后的形状，使其左上角尽量靠近 (0,0)
                            // 这对于 can_place_block 的 grid_x, grid_y 计算很重要
                            let normalized_cells = block::normalize_cells(current_rotated_cells.clone());
                            
                            // 再次检查格子数，因为旋转和标准化理论上不改变格子数，但以防万一
                            if normalized_cells.is_empty() || normalized_cells.len() > max_cells_to_fill {
                                current_rotated_cells = block::rotate_90_clockwise(&normalized_cells);
                                continue;
                            }

                            // 尝试将这个形状的每个格子对齐到当前棋盘空位 (c, r)
                            // 并计算出对应的 block 的基准放置点 (grid_x, grid_y)
                            for &(anchor_dx, anchor_dy) in &normalized_cells {
                                let place_grid_x = c as i32 - anchor_dx;
                                let place_grid_y = r as i32 - anchor_dy;

                                // 构建一个临时的 BlockShape 用于检查
                                let temp_block_to_check = BlockShape {
                                    cells: normalized_cells.clone(),
                                    color: Color::new(0.0,0.0,0.0,0.0), // 颜色在此时不重要
                                };

                                if self.can_place_block(&temp_block_to_check, place_grid_x, place_grid_y) {
                                    // 如果可以放置，我们找到了一个候选。
                                    candidate_blocks.push(BlockShape {
                                        cells: normalized_cells.clone(),
                                        color: block::get_random_block_color(),
                                    });
                                    // 找到一种放置方式后，可以不用再试这个形状的其他锚点了（针对当前空位）
                                    // 但一个形状的不同旋转仍然需要尝试
                                    break; 
                                }
                            }
                            // 准备下一次旋转
                            current_rotated_cells = block::rotate_90_clockwise(&normalized_cells);
                            // 如果旋转一周后回到原点（或变成空），则停止
                            if block::normalize_cells(current_rotated_cells.clone()) == block::normalize_cells(base_shape_ref.to_vec()) && _rotation_idx > 0 {
                                if base_shape_ref.len() > 1 { // 对于非单点方块，旋转一圈后通常不同，除非高度对称
                                     // 对于SHAPE_O这种高度对称的，可能提早结束，这是OK的
                                }
                            }
                            if current_rotated_cells.is_empty() && !base_shape_ref.is_empty() {
                                break; // 防止空形状无限旋转
                            }
                        } // 结束旋转循环
                    } // 结束目标形状循环
                } // 结束 if cell is none
            } // 结束列循环
        } // 结束行循环

        // 去重：基于形状的 cell 坐标列表进行去重
        if !candidate_blocks.is_empty() {
            candidate_blocks.sort_unstable_by_key(|b| {
                let mut key_vec = b.cells.clone();
                key_vec.sort_unstable(); // 确保内部排序一致性以进行比较
                key_vec
            });
            candidate_blocks.dedup_by_key(|b| {
                let mut key_vec = b.cells.clone();
                key_vec.sort_unstable();
                key_vec
            });
        }
        candidate_blocks
    }
} 