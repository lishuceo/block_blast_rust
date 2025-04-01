use macroquad::prelude::*;

pub mod block;
pub mod grid;
pub mod save;
pub mod effects;

// 移除不必要的导入
// use wasm_bindgen::prelude::*;

// 不需要导入自己定义的模块，因为它们已经在当前文件中声明
// use crate::block;
// use crate::grid;
// use crate::save;
// use crate::effects;

// 更高效的立体感方块绘制函数
pub fn draw_cube_block(x: f32, y: f32, size: f32, color: Color) {
    // 亮色和暗色偏移量
    let light_factor = 0.4;
    let dark_factor = 0.4;
    
    // 边缘厚度
    let border = size * 0.15;
    
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
    
    // 左边 - 亮色
    draw_triangle(
        Vec2::new(x, y), 
        Vec2::new(x, y + size), 
        Vec2::new(x + border, y + size - border),
        light_color
    );
    draw_triangle(
        Vec2::new(x, y), 
        Vec2::new(x + border, y + border), 
        Vec2::new(x + border, y + size - border),
        light_color
    );
    
    // 右边 - 暗色
    draw_triangle(
        Vec2::new(x + size, y), 
        Vec2::new(x + size, y + size), 
        Vec2::new(x + size - border, y + size - border),
        dark_color
    );
    draw_triangle(
        Vec2::new(x + size, y), 
        Vec2::new(x + size - border, y + border), 
        Vec2::new(x + size - border, y + size - border),
        dark_color
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

// 绘制中文文本的辅助函数(转为英文)
fn draw_chinese_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    // 将中文替换为对应的英文
    let english_text = match text {
        "俄罗斯方块消除" => "Tetris Block Blast",
        "可拖拽方块" => "Draggable Blocks",
        "分数: " => "Score: ",
        s if s.starts_with("分数: ") => {
            let score_part = &s["分数: ".len()..];
            &format!("Score: {}", score_part)
        },
        "连击: " => "Combo: ",
        s if s.starts_with("连击: ") => {
            let combo_part = &s["连击: ".len()..];
            &format!("Combo: {}", combo_part)
        },
        "最高分: " => "High Score: ",
        s if s.starts_with("最高分: ") => {
            let high_score_part = &s["最高分: ".len()..];
            &format!("High Score: {}", high_score_part)
        },
        "点击开始游戏" => "Click to Start",
        "将方块拖放到网格中，填满行或列即可消除" => "Drag blocks to the grid, fill rows or columns to clear them",
        "游戏结束" => "Game Over",
        s if s.starts_with("最终得分: ") => {
            let final_score_part = &s["最终得分: ".len()..];
            &format!("Final Score: {}", final_score_part)
        },
        "点击重新开始" => "Click to Restart",
        "简单模式" => "Easy Mode",
        "普通模式" => "Normal Mode",
        "按空格键切换游戏难度" => "Press Space to Change Difficulty",
        "1/2:调整方块概率 3/4:调整方块数量" => "1/2: Adjust Block Chances 3/4: Adjust Block Count",
        s if s.starts_with("消除: ") => {
            let info = &s["消除: ".len()..];
            &format!("Clear: {}", info)
        },
        _ => text  // 如果找不到对应英文，就保留原文本
    };
    
    // 使用默认字体绘制英文文本
    draw_text(english_text, x, y, font_size, color);
}

// 游戏状态枚举
#[derive(PartialEq)]
enum GameState {
    Menu,
    Playing,
    GameOver,
}

// 游戏数据
struct Game {
    state: GameState,
    grid: grid::Grid,
    current_blocks: Vec<block::BlockShape>,  // 当前可选方块
    score: u32,
    combo: u32,
    drag_block_idx: Option<usize>,    // 当前拖拽的方块索引
    drag_pos: Option<Vec2>,           // 拖拽位置
    drag_offset: Vec2,                // 新增：拖动偏移量，记录手指与方块的初始偏移
    save_data: save::SaveData,
    easy_mode: bool,                  // 简单模式标志
    simple_block_chance: i32,         // 简单方块生成概率 (0-100)
    standard_block_chance: i32,       // 标准方块生成概率 (0-100)
    blocks_per_generation: usize,     // 每次生成的方块数量 (1-5)
    effects: effects::Effects,         // 特效系统
}

impl Game {
    fn new() -> Self {
        Game {
            state: GameState::Menu,
            grid: grid::Grid::new(),
            current_blocks: Vec::new(),
            score: 0,
            combo: 0,
            drag_block_idx: None,
            drag_pos: None,
            drag_offset: Vec2::new(0.0, 0.0), // 初始化为零偏移
            save_data: save::SaveData::load(),
            easy_mode: true,          // 默认开启简单模式
            simple_block_chance: 30,  // 简单方块30%概率
            standard_block_chance: 60, // 标准方块60%概率
            blocks_per_generation: 3, // 默认生成3个方块
            effects: effects::Effects::new(), // 初始化特效系统
        }
    }
    
    // 生成随机方块
    fn generate_blocks(&mut self) {
        self.current_blocks.clear();
        
        // 随机生成设定数量的方块
        for _ in 0..self.blocks_per_generation {
            let block = block::BlockShape::random_with_chances(self.simple_block_chance, self.standard_block_chance);
            self.current_blocks.push(block);
        }
    }
    
    // 处理拖拽开始
    fn start_drag(&mut self, mouse_pos: Vec2) {
        // 检查是否点击了某个可选方块 - 竖屏模式下的布局
        let grid_size = screen_width() * 0.9;
        let cell_size = grid_size / 8.0;
        
        // 计算可拖拽方块区域的位置
        // 使用动态计算的顶部偏移
        let grid_offset_y = screen_height() * 0.07;
        
        // 检测小屏幕
        let is_small_screen = screen_height() < 600.0;
        let spacing = if is_small_screen { 20.0 } else { 30.0 };
        let separator_y = grid_offset_y + grid_size + 15.0 + spacing;
        let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
        
        // 确保底部区域最小高度
        let min_bottom_height = screen_height() * 0.2;
        let bottom_area_height = (screen_height() - bottom_area_top).max(min_bottom_height);
        let blocks_y = bottom_area_top + bottom_area_height / 2.0; // 垂直居中
        
        // 计算方块布局 - 根据最大方块数量(blocks_per_generation)确定尺寸
        let max_block_size = cell_size * 4.0; // 最大方块尺寸
        let block_size = if self.blocks_per_generation <= 2 {
            max_block_size // 对于1-2个最大方块，使用最大尺寸
        } else {
            // 对于更多方块，减小尺寸以适应屏幕
            // 考虑屏幕大小，在小屏幕上进一步减小尺寸
            let width_factor = if is_small_screen { 0.80 } else { 0.85 };
            (screen_width() * width_factor) / (self.blocks_per_generation as f32 * 1.2)
        };
        
        let block_margin = block_size * 0.2; // 方块之间的间距根据方块大小缩放
        let total_width = block_size * self.current_blocks.len() as f32 + block_margin * (self.current_blocks.len() as f32 - 1.0);
        let start_x = (screen_width() - total_width) / 2.0;
        
        for (idx, _block) in self.current_blocks.iter().enumerate() {
            // 计算每个方块的中心位置
            let block_pos_x = start_x + block_size/2.0 + idx as f32 * (block_size + block_margin);
            let block_pos_y = blocks_y;
            
            // 增加容错范围 - 使点击判定区域比实际方块大一些
            let tolerance_factor = 1.4; // 增加40%的判定区域
            let touch_width = block_size * tolerance_factor;
            let touch_height = block_size * tolerance_factor;
            
            // 创建扩大后的判定区域，保持中心点不变
            let block_rect = Rect::new(
                block_pos_x - touch_width/2.0, 
                block_pos_y - touch_height/2.0,
                touch_width, 
                touch_height
            );
            
            if block_rect.contains(mouse_pos) {
                self.drag_block_idx = Some(idx);
                
                // 计算向上的偏移量 - 使方块在手指上方显示
                // 核心改动：计算触摸点与方块中心的偏移，然后添加向上的额外偏移
                let touch_offset_y = -cell_size * 2.0; // 向上偏移2个格子的距离
                self.drag_offset = Vec2::new(
                    block_pos_x - mouse_pos.x,  // 水平偏移保持原样，保持触摸点在方块中心下方
                    block_pos_y - mouse_pos.y + touch_offset_y // 垂直方向添加额外向上偏移
                );
                
                // 设置初始拖动位置，应用偏移
                let adjusted_pos = Vec2::new(
                    mouse_pos.x + self.drag_offset.x,
                    mouse_pos.y + self.drag_offset.y
                );
                self.drag_pos = Some(adjusted_pos);
                break;
            }
        }
    }
    
    // 处理拖拽结束 - 这个方法已在update_game中实现
    fn end_drag(&mut self) {
        // 注意：这个方法不再被使用，逻辑已转移到update_game函数中
        // 保留此方法是为了避免大量修改代码结构
    }
    
    // 检查游戏结束条件
    fn check_game_over(&self) -> bool {
        // 检查每个当前方块是否有位置可以放置
        for block in &self.current_blocks {
            for y in 0..8 {
                for x in 0..8 {
                    if self.grid.can_place_block(block, x, y) {
                        return false; // 找到至少一个可放置位置
                    }
                }
            }
        }
        true // 没有可放置位置，游戏结束
    }
}

// 绘制函数
fn draw_game(game: &Game) {
    // 修改窗口背景为深灰色
    clear_background(Color::new(0.2, 0.2, 0.22, 1.0));
    
    // 绘制游戏标题
    draw_chinese_text("俄罗斯方块消除", 
             screen_width() / 2.0 - 120.0, 
             30.0, 
             25.0, // 减小字体大小
             WHITE);
    
    // 绘制游戏内容
    // 计算网格尺寸和位置
    let grid_size = screen_width() * 0.9;
    let cell_size = grid_size / 8.0;
    let grid_offset_x = (screen_width() - grid_size) / 2.0;
    
    // 根据屏幕大小动态计算顶部偏移
    let grid_offset_y = screen_height() * 0.07; // 使用屏幕高度的7%左右，而不是固定的60像素
    
    // 绘制游戏网格背景
    draw_rectangle(
        grid_offset_x - 5.0,
        grid_offset_y - 5.0,
        grid_size + 10.0,
        grid_size + 10.0,
        Color::new(0.1, 0.1, 0.12, 1.0)
    );
    
    // 绘制游戏网格
    game.grid.draw(grid_offset_x, grid_offset_y, cell_size);
    
    // 更新粒子效果系统
    game.effects.draw();
    
    // 显示游戏分数
    let score_y = grid_offset_y + grid_size + 15.0;
    draw_chinese_text(
        &format!("分数: {}", game.score), 
        20.0, 
        score_y, 
        20.0, 
        WHITE
    );
    
    // 显示最高分
    draw_chinese_text(
        &format!("最高分: {}", game.save_data.high_score), 
        screen_width() - 150.0, 
        score_y, 
        20.0, 
        WHITE
    );
    
    // 绘制分隔线
    // 检测小屏幕并调整间距
    let is_small_screen = screen_height() < 600.0;
    let spacing = if is_small_screen { 20.0 } else { 30.0 };
    let separator_y = grid_offset_y + grid_size + 15.0 + spacing;
    let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
    draw_line(
        10.0,
        separator_y,
        screen_width() - 10.0,
        separator_y,
        2.0,
        Color::new(0.3, 0.3, 0.3, 1.0)
    );
    
    // 绘制下方区域的背景
    let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
    // 确保底部区域至少有屏幕高度的一定比例
    let min_bottom_height = screen_height() * 0.2; // 至少占屏幕高度的20%
    let bottom_area_height = (screen_height() - bottom_area_top).max(min_bottom_height);
    draw_rectangle(
        0.0,
        bottom_area_top,
        screen_width(),
        bottom_area_height,
        Color::new(0.15, 0.15, 0.17, 1.0)
    );
    
    // 绘制可选方块区域的标题
    draw_chinese_text(
        "可拖拽方块", 
        20.0, 
        bottom_area_top + (if is_small_screen { 15.0 } else { 25.0 }), 
        20.0, 
        WHITE
    );
    
    // 绘制当前可选方块 - 在竖屏模式下水平排列
    // 计算垂直位置，使方块位于底部区域的中间
    let blocks_y = bottom_area_top + bottom_area_height / 2.0; // 可拖拽方块位于底部区域的垂直中心
    
    // 计算方块布局 - 根据最大方块数量(blocks_per_generation)确定尺寸，而非当前方块数量
    // 这样即使放置了方块，剩余方块的大小也不会突然变化
    let max_block_size = cell_size * 4.0; // 最大方块尺寸
    let block_size = if game.blocks_per_generation <= 2 {
        max_block_size // 对于1-2个最大方块，使用最大尺寸
    } else {
        // 对于更多方块，减小尺寸以适应屏幕
        // 考虑屏幕大小，在小屏幕上进一步减小尺寸
        let width_factor = if is_small_screen { 0.80 } else { 0.85 };
        (screen_width() * width_factor) / (game.blocks_per_generation as f32 * 1.2)
    };
    
    let block_margin = block_size * 0.2; // 方块之间的间距根据方块大小缩放
    let total_width = block_size * game.current_blocks.len() as f32 + block_margin * (game.current_blocks.len() as f32 - 1.0);
    let start_x = (screen_width() - total_width) / 2.0;
    
    for (idx, block) in game.current_blocks.iter().enumerate() {
        let block_pos_x = start_x + block_size/2.0 + idx as f32 * (block_size + block_margin);
        let block_pos_y = blocks_y;
        
        // 绘制方块 - 根据方块尺寸调整单元格大小
        let cell_scale = block_size / (cell_size * 5.0); // 调整单元格大小与方块大小的比例
        for &(dx, dy) in &block.cells {
            let x = block_pos_x + dx as f32 * cell_size * cell_scale;
            let y = block_pos_y + dy as f32 * cell_size * cell_scale;
            draw_cube_block(x - cell_size * cell_scale / 2.0, y - cell_size * cell_scale / 2.0, 
                          cell_size * cell_scale, block.color);
        }
    }
    
    // 绘制拖拽中的方块
    if let (Some(block_idx), Some(pos)) = (game.drag_block_idx, game.drag_pos) {
        // 确保索引有效
        if block_idx < game.current_blocks.len() {
            let block = &game.current_blocks[block_idx];
            
            // 修改：使用左上角第一个格子作为基准点
            // 计算最小的x和y偏移，即左上角的格子位置
            let min_dx = block.cells.iter().map(|(dx, _)| *dx).min().unwrap_or(0);
            let min_dy = block.cells.iter().map(|(_, dy)| *dy).min().unwrap_or(0);
            
            // 计算基准点在网格中的位置
            let block_base_x = pos.x - cell_size / 2.0;
            let block_base_y = pos.y - cell_size / 2.0;
            
            // 使用左上角的格子位置判断网格位置
            let grid_x = ((block_base_x - grid_offset_x) / cell_size).floor() as i32 - min_dx;
            let grid_y = ((block_base_y - grid_offset_y) / cell_size).floor() as i32 - min_dy;
            
            // 判断是否在有效网格范围内
            let is_valid_pos = grid_x >= 0 && grid_x < 8 && grid_y >= 0 && grid_y < 8;
            let can_place = is_valid_pos && game.grid.can_place_block(block, grid_x, grid_y);
            
            // 为所有单元格绘制预览
            for &(dx, dy) in &block.cells {
                let preview_x = grid_offset_x + (grid_x + dx) as f32 * cell_size;
                let preview_y = grid_offset_y + (grid_y + dy) as f32 * cell_size;
                
                // 仅当预览位置在有效范围内时才绘制
                if (grid_x + dx) >= 0 && (grid_x + dx) < 8 && (grid_y + dy) >= 0 && (grid_y + dy) < 8 {
                    // 根据能否放置绘制不同颜色
                    if can_place {
                        // 半透明绿色
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.2, 0.8, 0.2, 0.4));
                    } else {
                        // 半透明红色
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.8, 0.2, 0.2, 0.4));
                    }
                }
            }
            
            // 在网格上拖动时使用标准大小
            for &(dx, dy) in &block.cells {
                let x = pos.x + dx as f32 * cell_size - min_dx as f32 * cell_size;
                let y = pos.y + dy as f32 * cell_size - min_dy as f32 * cell_size;
                draw_cube_block(x - cell_size/2.0, y - cell_size/2.0, cell_size, block.color);
            }
        }
    }
    
    // 绘制菜单/游戏结束界面
    match game.state {
        GameState::Menu => {
            // 绘制半透明背景
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
            
            // 绘制大标题
            draw_chinese_text("俄罗斯方块消除", 
                     screen_width() / 2.0 - 150.0, 
                     screen_height() / 3.0, 
                     30.0, 
                     WHITE);
            
            // 绘制开始提示
            draw_chinese_text("点击开始游戏", 
                     screen_width() / 2.0 - 100.0, 
                     screen_height() / 2.0, 
                     25.0, 
                     Color::new(1.0, 0.8, 0.2, 1.0));
            
            // 绘制最高分
            draw_chinese_text(&format!("最高分: {}", game.save_data.high_score), 
                     screen_width() / 2.0 - 80.0, 
                     screen_height() / 2.0 + 80.0, 
                     22.0, 
                     Color::new(0.2, 0.8, 1.0, 1.0));
            
            // 绘制难度选择
            let mode_text = if game.easy_mode { "简单模式" } else { "普通模式" };
            draw_chinese_text(mode_text, 
                     screen_width() / 2.0 - 80.0, 
                     screen_height() / 2.0 + 120.0, 
                     22.0, 
                     if game.easy_mode { GREEN } else { YELLOW });
            
            draw_chinese_text("按空格键切换游戏难度", 
                     screen_width() / 2.0 - 140.0, 
                     screen_height() / 2.0 + 150.0, 
                     18.0, 
                     WHITE);
            
            draw_chinese_text("1/2:调整方块概率 3/4:调整方块数量", 
                     screen_width() / 2.0 - 180.0, 
                     screen_height() / 2.0 + 180.0, 
                     18.0, 
                     GRAY);
        },
        GameState::GameOver => {
            // 绘制半透明背景
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
            
            // 绘制游戏结束文本
            draw_chinese_text("游戏结束", 
                     screen_width() / 2.0 - 80.0, 
                     screen_height() / 3.0, 
                     30.0, 
                     WHITE);
            
            // 绘制最终得分
            draw_chinese_text(&format!("最终得分: {}", game.score), 
                     screen_width() / 2.0 - 100.0, 
                     screen_height() / 2.0, 
                     25.0, 
                     Color::new(1.0, 0.8, 0.2, 1.0));
            
            // 绘制最高分
            let new_record = game.score > game.save_data.high_score;
            let high_score_text = if new_record {
                format!("新纪录! {}", game.score)
            } else {
                format!("最高分: {}", game.save_data.high_score)
            };
            
            draw_chinese_text(&high_score_text, 
                     screen_width() / 2.0 - 100.0, 
                     screen_height() / 2.0 + 40.0, 
                     22.0, 
                     if new_record { Color::new(1.0, 0.5, 0.0, 1.0) } else { Color::new(0.2, 0.8, 1.0, 1.0) });
            
            // 绘制重新开始提示
            draw_chinese_text("点击重新开始", 
                     screen_width() / 2.0 - 100.0, 
                     screen_height() / 2.0 + 100.0, 
                     25.0, 
                     WHITE);
        },
        _ => {}
    }
}

// 更新游戏状态
fn update_game(game: &mut Game) {
    // 更新粒子效果
    game.effects.update(get_frame_time());
    
    // 检测按空格键切换难度模式
    if is_key_pressed(KeyCode::Space) {
        game.easy_mode = !game.easy_mode;
    }
    
    // 调整简单方块概率 (±10%)
    if is_key_pressed(KeyCode::Key1) && game.simple_block_chance > 0 {
        game.simple_block_chance = (game.simple_block_chance - 10).max(0);
        game.standard_block_chance = ((100 - game.simple_block_chance) as f32 * 0.8) as i32;
    }
    if is_key_pressed(KeyCode::Key2) && game.simple_block_chance < 100 {
        game.simple_block_chance = (game.simple_block_chance + 10).min(100);
        game.standard_block_chance = ((100 - game.simple_block_chance) as f32 * 0.8) as i32;
    }
    
    // 调整每次生成的方块数量 (1-5)
    if is_key_pressed(KeyCode::Key3) && game.blocks_per_generation > 1 {
        game.blocks_per_generation -= 1;
        if game.current_blocks.len() > game.blocks_per_generation {
            game.current_blocks.pop();
        }
    }
    if is_key_pressed(KeyCode::Key4) && game.blocks_per_generation < 5 {
        game.blocks_per_generation += 1;
        if game.state == GameState::Playing && game.current_blocks.len() < game.blocks_per_generation {
            game.current_blocks.push(block::BlockShape::random_with_chances(
                game.simple_block_chance, game.standard_block_chance));
        }
    }
    
    match game.state {
        GameState::Menu => {
            if is_mouse_button_pressed(MouseButton::Left) {
                game.state = GameState::Playing;
                game.grid = grid::Grid::new();
                game.score = 0;
                game.combo = 0;
                game.generate_blocks();
            }
        },
        GameState::Playing => {
            // 获取鼠标位置
            let mouse_pos: Vec2 = mouse_position().into();
            
            // 计算网格位置和大小
            let grid_size = screen_width() * 0.9;
            let cell_size = grid_size / 8.0;
            let grid_offset_x = (screen_width() - grid_size) / 2.0;
            let grid_offset_y = screen_height() * 0.07;
            
            // 检测小屏幕并调整间距
            let is_small_screen = screen_height() < 600.0;
            let spacing = if is_small_screen { 20.0 } else { 30.0 };
            let separator_y = grid_offset_y + grid_size + 15.0 + spacing;
            let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
            
            // 处理拖拽逻辑
            if is_mouse_button_pressed(MouseButton::Left) {
                // 只能从底部区域开始拖动
                if mouse_pos.y > separator_y {
                    game.start_drag(mouse_pos);
                }
            }
            
            // 处理拖动中的方块
            if is_mouse_button_down(MouseButton::Left) && game.drag_block_idx.is_some() {
                if let Some(block_idx) = game.drag_block_idx {
                    // 检查索引是否有效
                    if block_idx < game.current_blocks.len() {
                        // 核心改动：应用偏移量使方块位于手指上方
                        let adjusted_pos = Vec2::new(
                            mouse_pos.x + game.drag_offset.x,
                            mouse_pos.y + game.drag_offset.y
                        );
                        game.drag_pos = Some(adjusted_pos);
                    } else {
                        // 索引无效，重置拖拽状态
                        game.drag_block_idx = None;
                        game.drag_pos = None;
                    }
                } else {
                    // 这个分支不应该发生，但以防万一也应用偏移
                    let adjusted_pos = Vec2::new(
                        mouse_pos.x + game.drag_offset.x,
                        mouse_pos.y + game.drag_offset.y
                    );
                    game.drag_pos = Some(adjusted_pos);
                }
            }
            
            // 在鼠标释放时处理方块放置
            if is_mouse_button_released(MouseButton::Left) && game.drag_block_idx.is_some() {
                if let Some(block_idx) = game.drag_block_idx {
                    if let Some(pos) = game.drag_pos {
                        let block = &game.current_blocks[block_idx];
                        
                        // 计算网格大小和位置
                        let grid_size = screen_width() * 0.9;
                        let cell_size = grid_size / 8.0;
                        let grid_offset_x = (screen_width() - grid_size) / 2.0;
                        let grid_offset_y = screen_height() * 0.07;
                        
                        // 修改：使用与预览相同的坐标计算逻辑
                        // 计算最小的x和y偏移，即左上角的格子位置
                        let min_dx = block.cells.iter().map(|(dx, _)| *dx).min().unwrap_or(0);
                        let min_dy = block.cells.iter().map(|(_, dy)| *dy).min().unwrap_or(0);
                        
                        // 计算基准点在网格中的位置
                        let block_base_x = pos.x - cell_size / 2.0;
                        let block_base_y = pos.y - cell_size / 2.0;
                        
                        // 使用左上角的格子位置判断网格位置
                        let grid_x = ((block_base_x - grid_offset_x) / cell_size).floor() as i32 - min_dx;
                        let grid_y = ((block_base_y - grid_offset_y) / cell_size).floor() as i32 - min_dy;
                        
                        // 检查并处理方块放置
                        if grid_x >= 0 && grid_x < 8 && grid_y >= 0 && grid_y < 8 {
                            if game.grid.can_place_block(block, grid_x, grid_y) {
                                // 执行放置
                                game.grid.place_block(block, grid_x, grid_y);
                                
                                // 首先记录当前哪些行和列是满的（将被消除）
                                let mut filled_rows = [false; 8];
                                let mut filled_cols = [false; 8];
                                
                                // 检查哪些行是满的
                                for y in 0..8 {
                                    let mut row_filled = true;
                                    for x in 0..8 {
                                        if game.grid.cells[y][x].is_none() {
                                            row_filled = false;
                                            break;
                                        }
                                    }
                                    filled_rows[y] = row_filled;
                                }
                                
                                // 检查哪些列是满的
                                for x in 0..8 {
                                    let mut col_filled = true;
                                    for y in 0..8 {
                                        if game.grid.cells[y][x].is_none() {
                                            col_filled = false;
                                            break;
                                        }
                                    }
                                    filled_cols[x] = col_filled;
                                }
                                
                                // 检查消除行和列
                                let (rows_cleared, cols_cleared) = game.grid.check_and_clear();
                                let cleared = rows_cleared + cols_cleared;
                                
                                if cleared > 0 {
                                    // 只在实际被消除的格子位置显示粒子效果
                                    // 对于被消除的行
                                    for y in 0..8 {
                                        if filled_rows[y] {
                                            for x in 0..8 {
                                                let effect_x = grid_offset_x + x as f32 * cell_size + cell_size/2.0;
                                                let effect_y = grid_offset_y + y as f32 * cell_size + cell_size/2.0;
                                                // 使用方块的颜色
                                                game.effects.show_clear_effect(effect_x, effect_y, block.color);
                                            }
                                        }
                                    }
                                    
                                    // 对于被消除的列
                                    for x in 0..8 {
                                        if filled_cols[x] {
                                            for y in 0..8 {
                                                // 避免重复在行列交点添加两次粒子效果
                                                if !filled_rows[y] {
                                                    let effect_x = grid_offset_x + x as f32 * cell_size + cell_size/2.0;
                                                    let effect_y = grid_offset_y + y as f32 * cell_size + cell_size/2.0;
                                                    // 使用方块的颜色
                                                    game.effects.show_clear_effect(effect_x, effect_y, block.color);
                                                }
                                            }
                                        }
                                    }
                                    
                                    // 高combo时显示特殊效果
                                    if game.combo >= 2 {
                                        let combo_x = screen_width() / 2.0;
                                        let combo_y = grid_offset_y + grid_size / 2.0;
                                        game.effects.show_combo_effect(game.combo, combo_x, combo_y);
                                    }
                                    
                                    // 更新分数和连击
                                    game.combo += 1;
                                    game.score += cleared * 100 * game.combo;
                                    
                                    // 更新最高分
                                    if game.score > game.save_data.high_score {
                                        game.save_data.high_score = game.score;
                                        game.save_data.save();
                                    }
                                } else {
                                    // 重置连击
                                    game.combo = 0;
                                }
                                
                                // 移除已使用的方块
                                game.current_blocks.remove(block_idx);
                                
                                // 如果没有方块了，生成新的
                                if game.current_blocks.is_empty() {
                                    game.generate_blocks();
                                }
                            }
                        }
                    }
                    
                    // 重置拖拽状态
                    game.drag_block_idx = None;
                    game.drag_pos = None;
                }
            }
            
            // 检查游戏结束
            if game.check_game_over() {
                game.state = GameState::GameOver;
            }
        },
        GameState::GameOver => {
            if is_mouse_button_pressed(MouseButton::Left) {
                game.state = GameState::Menu;
            }
        }
    }
}

#[macroquad::main("方块消除游戏")]
async fn main() {
    run_game().await
}

// 主要游戏逻辑函数
async fn run_game() {
    // 设置竖屏窗口
    request_new_screen_size(400.0, 600.0); // 修改为竖屏比例
    
    let mut game = Game::new();
    
    loop {
        update_game(&mut game);
        draw_game(&game);
        
        next_frame().await
    }
}

