// 网格模块，处理方块放置和消除逻辑
use macroquad::prelude::*;
use crate::block::{self, BlockShape};
use crate::drawing;
use crate::constants;

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
                                    base_shape_name: "HELPER_TEMP", // 临时名称
                                };

                                if self.can_place_block(&temp_block_to_check, place_grid_x, place_grid_y) {
                                    // 如果可以放置，我们找到了一个候选。
                                    candidate_blocks.push(BlockShape {
                                        cells: normalized_cells.clone(),
                                        color: block::get_random_block_color(),
                                        base_shape_name: "HELPER_SHAPE", // 标记为辅助形状
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

    // 新增：绘制网格和其中的方块（不带高亮）
    pub fn draw(&self, offset_x: f32, offset_y: f32, cell_size: f32) {
        for y_idx in 0..8 {
            for x_idx in 0..8 {
                let pos_x = offset_x + x_idx as f32 * cell_size;
                let pos_y = offset_y + y_idx as f32 * cell_size;

                // 为所有单元格绘制边框线
                // 使用 constants::COLOR_BORDER 或者一个适合网格线的颜色
                draw_rectangle_lines(pos_x, pos_y, cell_size, cell_size, 1.0, constants::COLOR_BORDER);
                
                // 如果单元格中有方块，则绘制它
                if let Some(block_color) = self.cells[y_idx][x_idx] {
                    drawing::draw_cube_block(
                        pos_x,
                        pos_y,
                        cell_size,
                        block_color,
                    );
                }
                // 如果单元格为空，则不执行额外绘制，它将显示网格背景和上面的边框线
            }
        }
    }

    /// 分析连通的空白区域
    /// 返回所有连通区域的信息
    pub fn analyze_connected_empty_regions(&self) -> Vec<RegionInfo> {
        let mut visited = [[false; 8]; 8];
        let mut regions = Vec::new();

        for y in 0..8 {
            for x in 0..8 {
                if self.cells[y][x].is_none() && !visited[y][x] {
                    // 发现一个新的连通区域
                    let mut region_cells = Vec::new();
                    self.flood_fill(x, y, &mut visited, &mut region_cells);
                    
                    if !region_cells.is_empty() {
                        let info = RegionInfo::from_cells(region_cells);
                        regions.push(info);
                    }
                }
            }
        }

        // 按区域大小降序排序
        regions.sort_by(|a, b| b.cell_count.cmp(&a.cell_count));
        regions
    }

    /// 使用洪水填充算法找出连通的空白区域
    fn flood_fill(&self, x: usize, y: usize, visited: &mut [[bool; 8]; 8], cells: &mut Vec<(usize, usize)>) {
        if x >= 8 || y >= 8 || visited[y][x] || self.cells[y][x].is_some() {
            return;
        }

        visited[y][x] = true;
        cells.push((x, y));

        // 检查四个方向的邻居
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for &(dx, dy) in &directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            
            if new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8 {
                self.flood_fill(new_x as usize, new_y as usize, visited, cells);
            }
        }
    }

    /// 基于连通区域分析判断棋盘的困难程度
    /// 返回一个 0.0 到 1.0 之间的值，越高表示越困难
    pub fn get_difficulty_score(&self) -> f32 {
        let regions = self.analyze_connected_empty_regions();
        
        if regions.is_empty() {
            return 1.0; // 没有空白区域，游戏结束
        }

        let largest_region = &regions[0];
        let total_empty_cells: usize = regions.iter().map(|r| r.cell_count).sum();
        
        // 基于多个因素计算困难度分数
        let mut difficulty = 0.0;

        // 1. 最大连通区域的大小 - 调整阈值使其更敏感
        if largest_region.cell_count < 6 {
            difficulty += 0.4;  // 小于6格子：很困难
        } else if largest_region.cell_count < 12 {
            difficulty += 0.25; // 6-11格子：中等困难
        } else if largest_region.cell_count < 20 {
            difficulty += 0.15; // 12-19格子：轻微困难
        } else if largest_region.cell_count < 30 {
            difficulty += 0.08; // 20-29格子：很轻微困难
        }

        // 2. 最大连通区域的形状 - 狭长形状更困难
        if largest_region.shape_score < 0.3 {
            difficulty += 0.25; // 降低权重，从0.3改为0.25
        } else if largest_region.shape_score < 0.5 {
            difficulty += 0.12; // 降低权重，从0.15改为0.12
        }

        // 3. 区域的碎片化程度 - 提高权重，使其更重要
        let fragmentation = regions.len() as f32 / (total_empty_cells as f32).max(1.0);
        difficulty += fragmentation * 0.35; // 提高权重，从0.2改为0.35

        // 4. 空白格子占比 - 新增因素
        let empty_ratio = total_empty_cells as f32 / 64.0;
        if empty_ratio < 0.3 {
            difficulty += 0.15; // 空白格子少于30%时增加困难度
        } else if empty_ratio < 0.5 {
            difficulty += 0.08; // 空白格子少于50%时轻微增加困难度
        }

        // 5. 是否有能容纳大块的区域
        let has_large_square_region = regions.iter().any(|r| r.can_fit_4x4_block());
        if !has_large_square_region {
            difficulty += 0.15; // 降低权重，从0.2改为0.15
        }

        // 确保返回值在 0.0 到 1.0 之间
        difficulty.min(1.0)
    }
}

/// 连通区域信息
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub cell_count: usize,
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
    pub width: usize,
    pub height: usize,
    pub shape_score: f32, // 0.0 到 1.0，越接近 1.0 表示越方正
    cells: Vec<(usize, usize)>,
}

impl RegionInfo {
    fn from_cells(cells: Vec<(usize, usize)>) -> Self {
        let mut min_x = 8;
        let mut max_x = 0;
        let mut min_y = 8;
        let mut max_y = 0;

        for &(x, y) in &cells {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        
        // 计算形状分数
        // 考虑长宽比和填充率
        let aspect_ratio = (width.min(height) as f32) / (width.max(height) as f32);
        let fill_ratio = cells.len() as f32 / (width * height) as f32;
        let shape_score = aspect_ratio * fill_ratio;

        RegionInfo {
            cell_count: cells.len(),
            min_x,
            max_x,
            min_y,
            max_y,
            width,
            height,
            shape_score,
            cells,
        }
    }

    /// 检查该区域是否能容纳 4x4 的方块
    pub fn can_fit_4x4_block(&self) -> bool {
        self.width >= 4 && self.height >= 4
    }

    /// 检查该区域是否偏向方形且足够大
    pub fn is_square_like(&self) -> bool {
        self.shape_score > 0.5 && self.cell_count >= 9
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::*;

    #[test]
    fn test_connected_regions_simple() {
        let mut grid = Grid::new();
        
        // 创建一个简单的测试场景：两个分离的空白区域
        // X表示填充，O表示空白
        // O O X X X X X X
        // O O X X X X X X
        // X X X X X X X X
        // X X X X X X X X
        // X X X X X X X X
        // X X X X O O O X
        // X X X X O O O X
        // X X X X O O O X
        
        let filled_color = Color::new(1.0, 0.0, 0.0, 1.0);
        
        // 填充大部分格子
        for y in 0..8 {
            for x in 0..8 {
                grid.cells[y][x] = Some(filled_color);
            }
        }
        
        // 创建两个空白区域
        grid.cells[0][0] = None;
        grid.cells[0][1] = None;
        grid.cells[1][0] = None;
        grid.cells[1][1] = None;
        
        grid.cells[5][4] = None;
        grid.cells[5][5] = None;
        grid.cells[5][6] = None;
        grid.cells[6][4] = None;
        grid.cells[6][5] = None;
        grid.cells[6][6] = None;
        grid.cells[7][4] = None;
        grid.cells[7][5] = None;
        grid.cells[7][6] = None;
        
        // 分析连通区域
        let regions = grid.analyze_connected_empty_regions();
        
        // 应该有两个区域
        assert_eq!(regions.len(), 2);
        
        // 第一个区域应该是较大的3x3区域（9个格子）
        assert_eq!(regions[0].cell_count, 9);
        assert_eq!(regions[0].width, 3);
        assert_eq!(regions[0].height, 3);
        assert!(regions[0].shape_score > 0.9); // 完美的正方形
        
        // 第二个区域应该是较小的2x2区域（4个格子）
        assert_eq!(regions[1].cell_count, 4);
        assert_eq!(regions[1].width, 2);
        assert_eq!(regions[1].height, 2);
        assert!(regions[1].shape_score > 0.9); // 完美的正方形
        
        // 测试困难度分数
        let difficulty = grid.get_difficulty_score();
        // 最大区域只有9个格子，应该有中等困难度
        assert!(difficulty > 0.2 && difficulty < 0.5);
    }

    #[test]
    fn test_difficulty_score_extreme_cases() {
        // 测试1：完全填满的网格
        let mut grid = Grid::new();
        let filled_color = Color::new(1.0, 0.0, 0.0, 1.0);
        for y in 0..8 {
            for x in 0..8 {
                grid.cells[y][x] = Some(filled_color);
            }
        }
        assert_eq!(grid.get_difficulty_score(), 1.0); // 最高难度
        
        // 测试2：完全空的网格
        let grid = Grid::new();
        let difficulty = grid.get_difficulty_score();
        assert!(difficulty < 0.2); // 最低难度
    }

    #[test]
    fn test_fragmented_regions() {
        let mut grid = Grid::new();
        let filled_color = Color::new(1.0, 0.0, 0.0, 1.0);
        
        // 创建棋盘格模式，形成多个孤立的1x1空白区域
        for y in 0..8 {
            for x in 0..8 {
                if (x + y) % 2 == 0 {
                    grid.cells[y][x] = Some(filled_color);
                }
            }
        }
        
        let regions = grid.analyze_connected_empty_regions();
        // 应该有32个单格区域
        assert_eq!(regions.len(), 32);
        
        // 每个区域都应该是1x1
        for region in &regions {
            assert_eq!(region.cell_count, 1);
            assert_eq!(region.width, 1);
            assert_eq!(region.height, 1);
        }
        
        // 困难度应该很高，因为都是碎片化的小区域
        let difficulty = grid.get_difficulty_score();
        assert!(difficulty > 0.7);
    }
} 