use macroquad::prelude::*;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use block_blast::GameMode; 
use block_blast::wave::{WaveManager, WavePhase, ChallengeType}; 
use std::collections::HashMap;
use crate::effects::Effects; // <--- 添加对 Effects 类型的导入

// 确保以下导入存在
use macroquad::rand as mq_rand; 
use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::{COLOR_PRIMARY, COLOR_PRIMARY_DARK, COLOR_PRIMARY_OVERLAY, COLOR_BORDER, COLOR_TITLE, GOLD, ORANGE}; 
// use crate::utils; // <--- 确保这行被移除或注释掉

pub mod block;
pub mod grid;
pub mod save;
pub mod effects;
pub mod cloud;
pub mod log;
pub mod drawing; 
mod constants;
mod utils;    // <--- 确保这行存在，将 utils.rs 声明为 main.rs 的子模块

// 使用宏导入
#[macro_use]
mod log_macro_import {
    // 空模块，仅用于导入宏
    pub use crate::log::*;
}


// 将字体数据直接嵌入到可执行文件中
const CHINESE_FONT_DATA: &[u8] = include_bytes!("../resources/fonts/SourceHanSansCN-Medium.ttf");

// 全局中文字体变量 - 使用Font类型
static CHINESE_FONT: Lazy<Mutex<Option<Font>>> = Lazy::new(|| Mutex::new(None));

// 获取设备DPI缩放比例的辅助函数
fn get_dpi_scale() -> f32 {
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

// WASM初始化代码
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn wasm_init() {
    // 在WASM环境下设置更友好的panic处理
    std::panic::set_hook(Box::new(|panic_info| {
        let message = format!("游戏发生错误: {:?}", panic_info);
        log_error!("{}", message);
        // 这里可以添加显示错误信息的UI代码
    }));
}

// 绘制中文文本的辅助函数(支持真正的中文渲染)
fn draw_chinese_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    // 安全获取字体锁，处理可能的错误
    let chinese_font_result = CHINESE_FONT.lock();
    
    // 如果无法获取锁或字体未加载，回退到默认字体
    if chinese_font_result.is_err() || chinese_font_result.as_ref().unwrap().is_none() {
        // 回退到默认英文显示
        let english_text = match text {
            "逆向俄罗斯方块" => "Block Blast",
            "点击开始游戏" => "Click to Start",
            "最高分" => "High Score",
            "简单模式" => "Easy Mode",
            "普通模式" => "Normal Mode",
            "按空格键切换游戏难度" => "Press Space to Change Difficulty",
            "游戏结束" => "Game Over",
            "最终得分" => "Final Score",
            "点击重新开始" => "Click to Restart",
            "分数" => "Score",
            "连击" => "Combo",
            "将方块拖放到网格中，填满行或列即可消除" => "Drag blocks to fill rows or columns",
            _ if text.contains("简单方块概率") => "Block chances",
            _ if text.contains("可拖拽方块数量") => "Draggable blocks",
            _ if text.contains("消除") => "Cleared",
            _ => text,
        };
        
        // 使用传入的字体大小（已在外部应用了DPI缩放）
        draw_text(
            english_text, 
            x - measure_text(english_text, None, font_size as u16, 1.0).width / 2.0, 
            y, 
            font_size, 
            color
        );
        return;
    }
    
    // 安全地获取字体引用
    let chinese_font = chinese_font_result.unwrap();
    let font = if let Some(font) = &*chinese_font {
        font
    } else {
        // 这个分支不应该被执行，因为我们已经在前面检查了字体是否为None
        // 但为了代码安全，我们仍然提供一个回退方案
        let english_text = match text {
            "逆向俄罗斯方块" => "Block Blast",
            _ => text,
        };
        
        draw_text(
            english_text, 
            x - measure_text(english_text, None, font_size as u16, 1.0).width / 2.0, 
            y, 
            font_size, 
            color
        );
        return;
    };
    
    // 使用macroquad的measure_text函数精确测量文本宽度
    let text_dims = measure_text(
        text,
        Some(font),
        font_size as u16,
        1.0  // 字间距
    );
    
    // 确保文本居中显示
    let text_x = x - text_dims.width / 2.0;
    
    // 基线调整 - 解决垂直对齐问题
    let baseline_adjustment = font_size * 0.2; // 基准值
    
    // 根据文本内容进行微调
    let y_offset = match text {
        "逆向俄罗斯方块" => -font_size * 0.1,  // 标题上移
        "点击开始游戏" | "点击重新开始" => font_size * 0.05,  // 轻微下移
        _ if text.contains("分数") || text.contains("最高分") => font_size * 0.05,
        _ if text.contains("连击") => font_size * 0.05,
        _ if text.chars().count() >= 10 => font_size * 0.08,  // 长文本稍微下移
        _ => 0.0  // 默认不偏移
    };
    
    // 绘制文本
    draw_text_ex(
        text,
        text_x,
        y + baseline_adjustment + y_offset,
        TextParams {
            font_size: font_size as u16,
            font: Some(font),
            color,
            ..Default::default()
        },
    );
}

// 游戏状态枚举
#[derive(PartialEq, Copy, Clone)]
enum GameState {
    MainMenu,   // 新增主菜单状态，用于显示Figma设计的首页
    Menu,
    Playing,
    GameOver,
    Leaderboard, // 新增排行榜状态
}

// 新增：动画块数据
#[derive(Clone)] // 需要 Clone，因为 Option<T> 要求 T: Clone
struct AnimatingBlockData {
    block_idx: usize,
    start_pos: Vec2,
    target_pos: Vec2,
    current_pos: Vec2,
    timer: f32,      // 0.0 to 1.0
    duration: f32,   // in seconds
}

// 游戏数据
struct Game {
    state: GameState,
    grid: grid::Grid,
    current_blocks: Vec<block::BlockShape>,  // 当前可选方块
    score: u32,
    combo: u32,
    drag_block_idx: Option<usize>,    // 当前拖拽的方块索引
    drag_pos: Option<Vec2>,           // 拖拽位置 (左上角单元格中心)
    drag_offset: Vec2,                // 新增：拖动偏移量，记录手指与方块的初始偏移
    drag_original_pos: Option<Vec2>,  // 新增：拖拽开始时，方块在底部区域的原始中心位置
    initial_mouse_drag_start_pos: Option<Vec2>, // 新增：记录拖动开始时的鼠标位置
    save_data: save::SaveData,        // 保留 SaveData 结构，但最高分不再使用它
    cloud_high_score: Option<u32>,    // 新增：从云端获取的最高分
    needs_score_upload: bool,         // 新增：标记是否需要在主循环中上传分数
    game_mode: GameMode,              // 新增：游戏模式枚举
    effects: effects::Effects,         // 特效系统
    wave_manager: WaveManager, // <--- 新增字段
    
    // 排行榜相关字段
    leaderboard_data: Vec<cloud::PlayerRank>, // 排行榜数据
    player_rank: Option<cloud::PlayerRank>,   // 玩家自己的排名
    is_leaderboard_loading: bool,             // 排行榜是否加载中
    leaderboard_error: Option<String>,        // 排行榜加载错误信息
    show_leaderboard_button: bool,            // 是否显示排行榜按钮
    score_uploaded: bool,                     // 分数是否已上传
    
    // 主菜单动画相关变量
    t_block_rotation: f32,            // T形方块旋转角度
    title_bounce: f32,                // 标题弹跳动画值
    animation_time: f32,              // 动画计时器
    title_texture: Option<Texture2D>, // 新增：用于存储标题图像
    
    // 页面过渡动画相关变量
    transition_alpha: f32,            // 过渡透明度 (0.0-2.0: 0.0-1.0为淡出阶段, 1.0-2.0为淡入阶段)
    transition_active: bool,          // 过渡动画是否激活
    transition_timer: f32,            // 过渡动画计时器
    transition_from: GameState,       // 过渡起始状态
    transition_to: GameState,         // 过渡目标状态
    transition_phase: bool,           // 过渡阶段 (false=淡出, true=淡入)
    
    // 新增：用于方块返回动画
    animating_block: Option<AnimatingBlockData>,

    // 新增：用于累积统计方块形状出现次数
    accumulated_shape_counts: HashMap<String, usize>,
}

impl Game {
    fn new() -> Self {
        Game {
            state: GameState::MainMenu, // 默认进入主菜单状态，而不是旧的Menu
            grid: grid::Grid::new(),
            current_blocks: Vec::new(),
            score: 0,
            combo: 0,
            drag_block_idx: None,
            drag_pos: None,
            drag_offset: Vec2::new(0.0, 0.0), // 初始化为零偏移, 会在start_drag中设置
            drag_original_pos: None, // 初始化为 None
            initial_mouse_drag_start_pos: None, // 初始化为 None
            save_data: save::SaveData::load(), // 仍然加载，但不再用于最高分判断
            cloud_high_score: None,    // 初始化为 None
            needs_score_upload: false, // 初始化为 false
            game_mode: GameMode::Normal, // 默认设为Normal模式
            effects: effects::Effects::new(), // 初始化特效系统
            wave_manager: WaveManager::new(), // <--- 初始化 WaveManager
            
            // 排行榜相关字段
            leaderboard_data: Vec::new(), // 排行榜数据
            player_rank: None,   // 玩家自己的排名
            is_leaderboard_loading: false,             // 排行榜是否加载中
            leaderboard_error: None,        // 排行榜加载错误信息
            show_leaderboard_button: false,            // 是否显示排行榜按钮
            score_uploaded: false,                     // 分数是否已上传
            
            // 主菜单动画相关变量
            t_block_rotation: 0.0,
            title_bounce: 0.0,
            animation_time: 0.0,
            title_texture: None, // 初始化标题图像为 None
            
            // 页面过渡动画相关变量
            transition_alpha: 1.0,
            transition_active: false,
            transition_timer: 0.0,
            transition_from: GameState::MainMenu,
            transition_to: GameState::MainMenu,
            transition_phase: false,
            
            animating_block: None, // 初始化为 None
            accumulated_shape_counts: HashMap::new(), // 初始化累积计数器
        }
    }
    
    // 生成随机方块
    fn generate_blocks(&mut self) {
        self.current_blocks.clear();
        
        // 使用新的困难度分析替代旧的填充率
        let difficulty_score = self.grid.get_difficulty_score();
        let offer_help = self.wave_manager.should_offer_helpful_block_v2(difficulty_score);
        let mut helpful_block_generated = false;

        // 添加更详细的日志，包含连通区域分析
        let regions = self.grid.analyze_connected_empty_regions();
        log_info!("generate_blocks: difficulty_score: {:.3}, offer_help: {}, regions_count: {}", 
                  difficulty_score, offer_help, regions.len());
        
        // 打印前3个最大的区域的详细信息
        for (i, region) in regions.iter().take(3).enumerate() {
            log_info!("  Region {}: {} cells, shape_score: {:.2}, size: {}x{}, bounds: ({},{}) to ({},{})", 
                      i + 1, region.cell_count, region.shape_score, region.width, region.height,
                      region.min_x, region.min_y, region.max_x, region.max_y);
        }
        
        // 计算并显示各项评分细节
        let total_empty_cells: usize = regions.iter().map(|r| r.cell_count).sum();
        let fragmentation = if total_empty_cells > 0 { 
            regions.len() as f32 / total_empty_cells as f32 
        } else { 
            0.0 
        };
        log_info!("  Total empty: {}, Fragmentation: {:.3}, Has 4x4 space: {}", 
                  total_empty_cells, fragmentation, 
                  regions.iter().any(|r| r.can_fit_4x4_block()));

        if offer_help {
            // 尝试查找可以放置的1至5格的小方块
            let candidate_helpful_shapes = self.grid.find_placeable_shapes_for_empty_spots(
                5, // 最多填充5个格子
                &[
                    &block::SHAPE_DOT, 
                    &block::SHAPE_H2, 
                    &block::SHAPE_H3,
                    &block::SHAPE_O,
                    &block::SHAPE_L,
                    &block::SHAPE_T,
                    &block::SHAPE_I, // 4格直线
                    // 可以考虑未来加入 SHAPE_Z 等更复杂的形状，但要注意它们可能更难精确匹配空位
                ]
            );

            if !candidate_helpful_shapes.is_empty() {
                // 从候选列表中随机选择一个
                let selected_idx = macroquad::rand::gen_range(0, candidate_helpful_shapes.len() as i32) as usize;
                let helpful_block = candidate_helpful_shapes[selected_idx].clone(); 
                
                self.current_blocks.push(helpful_block);
                helpful_block_generated = true;
                log_info!("Helpful block generated: {:?}. Candidates found: {}", self.current_blocks.last().unwrap().cells, candidate_helpful_shapes.len());
            } else {
                log_info!("Offer help was true, but no placeable helpful shapes found with max_cells=5.");
            }
        }

        let num_blocks_to_generate_normally = 
            if helpful_block_generated {
                if self.wave_manager.blocks_per_generation > 0 {
                    self.wave_manager.blocks_per_generation.saturating_sub(1) // 确保不会小于0
                } else {
                    0
                }
            } else {
                self.wave_manager.blocks_per_generation
            };

        if num_blocks_to_generate_normally > 0 {
            log_info!(
                "Generating {} normal blocks for mode {:?}. Offer help: {}, Helpful generated: {}", 
                num_blocks_to_generate_normally, self.game_mode, offer_help, helpful_block_generated
            );
        }
        
        for i in 0..num_blocks_to_generate_normally {
            log_debug!("Generating normal block {}/{} for mode {:?}", i + 1, num_blocks_to_generate_normally, self.game_mode);
            let block = block::BlockShape::generate_for_mode(self.game_mode);
            self.current_blocks.push(block);
        }

        if self.current_blocks.is_empty() && self.wave_manager.blocks_per_generation > 0 {
            log_warn!("Block generation resulted in empty current_blocks (expected >0). Generating a fallback block.");
            self.current_blocks.push(block::BlockShape::new_dot()); 
        } else if self.current_blocks.is_empty() && self.wave_manager.blocks_per_generation == 0 {
            log_info!("Block generation resulted in empty current_blocks as per blocks_per_generation = 0 (expected).");
        }

        // --- 新增：统计并打印生成的方块类型 --- 
        // 移除临时的 shape_counts，我们将使用 game.accumulated_shape_counts
        // use std::collections::HashMap; 
        // let mut shape_counts: HashMap<String, usize> = HashMap::new();

        // 辅助函数 identify_shape_approx 不再需要，因为我们将使用 BlockShape.base_shape_name
        // fn identify_shape_approx(cells: &[(i32, i32)]) -> String { ... }

        for block_shape in &self.current_blocks {
            // 使用 BlockShape 中存储的 base_shape_name 进行精确统计
            let shape_name_str = block_shape.base_shape_name.to_string();
            *self.accumulated_shape_counts.entry(shape_name_str).or_insert(0) += 1;
        }

        log_info!("Accumulated generated block types distribution (GameMode: {:?}):
", self.game_mode);
        // 为了更好的可读性，对结果进行排序（可选）
        let mut sorted_counts: Vec<_> = self.accumulated_shape_counts.iter().collect();
        sorted_counts.sort_by_key(|k| k.0.clone()); // 按形状名称排序

        for (shape_name, count) in sorted_counts {
            log_info!("  - {}: {}", shape_name, count);
        }
        // --- 统计结束 ---
    }
    
    // 处理拖拽开始
    fn start_drag(&mut self, mouse_pos: Vec2) {
        // 检查是否点击了某个可选方块 - 竖屏模式下的布局
        let grid_size = screen_width() * 0.9;
        let cell_size = grid_size / 8.0;
        
        // 计算可拖拽方块区域的位置 - 使用宽高比
        let aspect_ratio = screen_width() / screen_height();
        
        // 基于宽高比判断屏幕类型
        let is_wide_screen = aspect_ratio > 0.8;   // 宽屏 (接近正方形)
        let is_tall_screen = aspect_ratio < 0.5;   // 高屏 (典型手机竖屏)
        let is_small_screen = screen_height() < 600.0;
        
        // 使用与其他位置相同的顶部偏移计算
        let grid_offset_y = if is_tall_screen {
            screen_height() * 0.22  // 从 0.18 增加到 0.22，与其他函数保持一致
        } else if is_wide_screen {
            screen_height() * 0.12  // 从 0.07 增加到 0.12，与其他函数保持一致
        } else {
            screen_height() * 0.15  // 从 0.10 增加到 0.15，与其他函数保持一致
        };
        
        // 更新分隔线和间距计算 - 使用宽高比
        let spacing = if is_tall_screen {
            60.0 // 高屏幕上使用更大间距
        } else if is_wide_screen {
            20.0 // 宽屏上使用较小间距
        } else {
            40.0 // 标准屏幕上使用中等间距
        };
        
        let separator_y = grid_offset_y + grid_size + 15.0 + spacing;
        let _bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
        
        // 确保底部区域最小高度 - 使用宽高比
        let min_bottom_height = if is_tall_screen {
            screen_height() * 0.25 // 在高屏上提供更大的底部区域
        } else if is_wide_screen {
            screen_height() * 0.15 // 在宽屏上使用较小的底部区域
        } else {
            screen_height() * 0.20 // 标准底部区域大小
        };
        let bottom_area_height = (screen_height() - separator_y).max(min_bottom_height);
        // let blocks_y = separator_y + bottom_area_height / 2.0; // 垂直居中
        // 将方块位置往上调整，与 draw_game 中的计算保持一致
        let blocks_y = separator_y + bottom_area_height * 0.4; // 从0.5 (中心) 减少到0.4，使方块向上移动
        
        // 计算方块布局 - 根据最大方块数量(blocks_per_generation)确定尺寸
        let max_block_size = cell_size * 4.0; // 最大方块尺寸
        let block_size = if self.wave_manager.blocks_per_generation <= 2 {
            max_block_size // 对于1-2个最大方块，使用最大尺寸
        } else {
            // 对于更多方块，减小尺寸以适应屏幕
            // 考虑屏幕大小，在小屏幕上进一步减小尺寸
            let width_factor = if is_small_screen { 0.80 } else { 0.85 };
            (screen_width() * width_factor) / (self.wave_manager.blocks_per_generation as f32 * 1.2)
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
                self.initial_mouse_drag_start_pos = Some(mouse_pos); // 记录拖动开始时的鼠标位置
                
                // --- 记录原始位置 --- 
                self.drag_original_pos = Some(Vec2::new(block_pos_x, block_pos_y));
                // ---------------------
                
                // 计算向上的偏移量 - 使方块在手指上方显示，但保持中心点对齐
                let touch_offset_y = -cell_size * 2.0; // 向上偏移2个格子的距离
                self.drag_offset = Vec2::new(
                    0.0,  // 不再需要水平偏移，因为我们使用中心点
                    touch_offset_y // 只保留垂直方向的额外偏移
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

    // 初始化云服务
    async fn init_cloud(&mut self) {
        // 初始化云服务SDK，使用实际的云服务而不是模拟数据
        match cloud::initialize_sdk().await {
            Ok(_) => {
                self.show_leaderboard_button = cloud::is_cloud_initialized();
                log_info!("云服务初始化成功");
            },
            Err(e) => {
                log_error!("云服务初始化失败: {}", e);
                self.show_leaderboard_button = false;
            }
        }
    }

    // 加载排行榜数据
    async fn load_leaderboard(&mut self) {
        self.is_leaderboard_loading = true;
        self.leaderboard_error = None;

        // 获取排行榜数据
        match cloud::get_leaderboard(50).await {
            Ok(_) => {
                // 获取玩家排名
                let _ = cloud::get_player_rank().await;
                
                // 从全局状态获取数据
                let (is_loading, error, leaderboard, player_rank) = cloud::get_leaderboard_data();
                
                // --- 添加调试打印 ---
                log_debug!("从 cloud::get_leaderboard_data 获取到的排行榜数据: {:?}", leaderboard);
                log_debug!("从 cloud::get_leaderboard_data 获取到的玩家排名: {:?}", player_rank);
                // --- 调试打印结束 ---
                
                self.is_leaderboard_loading = is_loading;
                self.leaderboard_error = error;
                self.leaderboard_data = leaderboard;
                self.player_rank = player_rank;
            },
            Err(e) => {
                self.is_leaderboard_loading = false;
                self.leaderboard_error = Some(e);
            }
        }
    }

    // 上传分数
    async fn upload_score(&mut self) {
        if self.score_uploaded || !self.show_leaderboard_button {
            return;
        }

        match cloud::upload_score(self.score).await {
            Ok(_) => {
                self.score_uploaded = true;
                // 上传成功后刷新排行榜数据
                self.load_leaderboard().await;
            },
            Err(e) => {
                log_error!("上传分数失败: {}", e);
            }
        }
    }

    // 更新主菜单动画
    fn update_main_menu_animations(&mut self, dt: f32) {
        // 更新总动画时间
        self.animation_time += dt;
        
        // 更新T形方块旋转 (缓慢旋转)
        self.t_block_rotation += dt * 0.2;
        if self.t_block_rotation > std::f32::consts::PI * 2.0 {
            self.t_block_rotation -= std::f32::consts::PI * 2.0;
        }
        
        // 计算标题弹跳动画 (使用正弦函数)
        self.title_bounce = (self.animation_time * 2.0).sin() * 5.0;
    }
    
    // 开始页面过渡动画
    fn start_transition(&mut self, to_state: GameState) {
        self.transition_active = true;
        self.transition_timer = 0.0;
        self.transition_alpha = 0.0; // 从透明开始
        self.transition_from = self.state;
        self.transition_to = to_state;
        self.transition_phase = false; // 开始于淡出阶段
        
        // 如果目标是排行榜，提前开始加载数据
        if to_state == GameState::Leaderboard {
            // 预加载标记，但延迟实际加载直到淡出阶段结束
            self.is_leaderboard_loading = true;
        }
    }

    // 平滑过渡辅助函数
    fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = f32::max(0.0, f32::min(1.0, (x - edge0) / (edge1 - edge0)));
        t * t * (3.0 - 2.0 * t) // 平滑的三次函数
    }
    
    // 更新页面过渡动画
    fn update_transition(&mut self, dt: f32) -> bool {
        if !self.transition_active {
            return false;
        }
        
        // 更新过渡计时器
        self.transition_timer += dt * 3.5; // 控制过渡速度，稍微放慢一点
        
        if !self.transition_phase {
            // 淡出阶段 (0.0-1.0)
            // 使用平滑的easeInOut函数
            let t = f32::min(self.transition_timer, 1.0);
            self.transition_alpha = Self::smooth_step(0.0, 1.0, t);
            
            // 淡出阶段完成后切换到淡入阶段
            if self.transition_timer >= 1.0 {
                self.state = self.transition_to; // 切换状态
                self.transition_phase = true;    // 切换到淡入阶段
                self.transition_timer = 0.0;     // 重置计时器
            }
        } else {
            // 淡入阶段 (1.0-0.0)
            // 使用平滑的easeInOut函数
            let t = f32::min(self.transition_timer, 1.0);
            self.transition_alpha = 1.0 - Self::smooth_step(0.0, 1.0, t);
            
            // 淡入阶段完成后结束过渡
            if self.transition_timer >= 1.0 {
                self.transition_active = false;
                self.transition_alpha = 0.0;
                return true; // 过渡完成
            }
        }
        
        false // 过渡仍在进行
    }
}

// 绘制排行榜界面
fn draw_leaderboard(game: &mut Game) {
    let width = screen_width();
    let height = screen_height();
    
    let top_color = Color::new(40.0/255.0, 72.0/255.0, 160.0/255.0, 0.95);
    let bottom_color = Color::new(20.0/255.0, 36.0/255.0, 80.0/255.0, 0.95);
    drawing::draw_vertical_gradient(0.0, 0.0, width, height, top_color, bottom_color);

    let title_font_size = 48.0;
    let week_range_font_size = 18.0;
    let header_font_size = 18.0;
    let entry_font_size = 20.0;
    let row_height = 45.0;
    let padding_general = 10.0;
    let padding_section = 20.0;

    let title_y_center = height * 0.12;
    draw_chinese_text("每周排行榜", width / 2.0, title_y_center, title_font_size, WHITE);

    let week_range_y_center = title_y_center + (title_font_size / 2.0) + (week_range_font_size / 2.0) + padding_general;
    let week_range_text = utils::get_current_week_display_text(); // <--- 使用新的函数
    draw_chinese_text(&week_range_text, width / 2.0, week_range_y_center, week_range_font_size, LIGHTGRAY);
    
    let back_button_size = 30.0;
    let back_button_padding = 15.0;
    let back_button_x = width - back_button_size - back_button_padding;
    let back_button_y = back_button_padding;
    let back_button_rect = Rect::new(back_button_x, back_button_y, back_button_size, back_button_size);
    let back_icon_text = "返回";
    let back_icon_font_size = 20.0;
    draw_chinese_text(back_icon_text, back_button_rect.center().x, back_button_rect.center().y + back_icon_font_size * 0.1, back_icon_font_size, LIGHTGRAY);

    if is_mouse_button_released(MouseButton::Left) && back_button_rect.contains(mouse_position().into()) {
        game.start_transition(GameState::MainMenu);
    }

    if !game.show_leaderboard_button || game.is_leaderboard_loading {
        let status_text_y = height * 0.5; 
        let status_font_size = 22.0;
        let loading_details_font_size = 16.0;

        if game.is_leaderboard_loading {
            draw_chinese_text("正在加载排行榜...", width / 2.0, status_text_y - 20.0, status_font_size, SKYBLUE);
            let dot_radius = 5.0;
            let dot_spacing = 20.0;
            let animation_speed = 2.0;
            let current_time = get_time() as f32;
            for i in 0..3 {
                let delay = i as f32 * 0.3;
                let scale = ((current_time * animation_speed + delay).sin() * 0.5 + 0.5).max(0.3);
                let alpha = scale;
                draw_circle(
                    width / 2.0 - dot_spacing + (i as f32 * dot_spacing),
                    status_text_y + 20.0, 
                    dot_radius * scale,
                    Color::new(SKYBLUE.r, SKYBLUE.g, SKYBLUE.b, alpha)
                );
            }
            draw_chinese_text("请稍候", width / 2.0, status_text_y + 50.0, loading_details_font_size, GRAY);
        } else {
            draw_chinese_text("无法连接排行榜服务", width / 2.0, status_text_y - 15.0, status_font_size, ORANGE);
            draw_chinese_text("请检查网络连接或稍后再试", width / 2.0, status_text_y + 15.0, loading_details_font_size, GRAY);
        }
    } else {
        let header_text_y_center = week_range_y_center + (week_range_font_size / 2.0) + (header_font_size / 2.0) + padding_section;
        let line_below_header_y = header_text_y_center + (header_font_size / 2.0) + padding_general;
        let entries_area_top_y = line_below_header_y + padding_general;

        let rank_center_x = width * 0.18;
        let name_left_x = width * 0.30;
        let score_right_x = width * 0.85;

        let leaderboard = &game.leaderboard_data;

        if leaderboard.is_empty() && !game.is_leaderboard_loading {
            draw_chinese_text("排行榜暂无数据", width / 2.0, height / 2.0, 24.0, WHITE);
        } else {
            draw_chinese_text("排名", rank_center_x, header_text_y_center, header_font_size, GRAY);
            let player_header_text = "玩家";
            // Pass None for font to measure_text to avoid locking CHINESE_FONT here
            let player_header_dims = measure_text(player_header_text, None, header_font_size as u16, 1.0);
            draw_chinese_text(player_header_text, name_left_x + player_header_dims.width / 2.0, header_text_y_center, header_font_size, GRAY);
            let score_header_text = "分数";
            // Pass None for font to measure_text
            let score_header_dims = measure_text(score_header_text, None, header_font_size as u16, 1.0);
            draw_chinese_text(score_header_text, score_right_x - score_header_dims.width / 2.0, header_text_y_center, header_font_size, GRAY);
            
            draw_line(width * 0.05, line_below_header_y, width * 0.95, line_below_header_y, 1.5, Color::new(0.5,0.5,0.5,0.7));

            for (i, rank_entry) in leaderboard.iter().take(10).enumerate() {
                let current_row_top_y = entries_area_top_y + (i as f32 * row_height);
                let current_row_center_y_for_text = current_row_top_y + (row_height / 2.0);
                
                let is_player_self = game.player_rank.as_ref().map_or(false, |pr| pr.name == rank_entry.name && pr.score == rank_entry.score);
                let row_bg_color = if is_player_self { Color::new(0.1, 0.2, 0.3, 0.7) } else { Color::new(0.0,0.0,0.0,0.0) };
                if is_player_self {
                    draw_rectangle(width * 0.05, current_row_top_y, width * 0.9, row_height, row_bg_color);
                }

                draw_chinese_text(&format!("{}", i + 1), rank_center_x, current_row_center_y_for_text, entry_font_size, WHITE);
                
                let mut player_name_display = rank_entry.name.clone();
                let max_name_render_width = score_right_x - name_left_x - (width*0.05);
                let approx_char_width = entry_font_size * 0.7; 
                let max_chars = (max_name_render_width / approx_char_width).floor() as usize;
                if player_name_display.chars().count() > max_chars && max_chars > 3 {
                    player_name_display = player_name_display.chars().take(max_chars.saturating_sub(1)).collect::<String>() + "...";
                }
                // Pass None for font to measure_text for width calculation
                let player_name_width = measure_text(&player_name_display, None, entry_font_size as u16, 1.0).width;
                draw_chinese_text(&player_name_display, name_left_x + player_name_width/2.0 , current_row_center_y_for_text, entry_font_size, WHITE);
                
                let score_display = format!("{}", rank_entry.score);
                // Pass None for font to measure_text for width calculation
                let score_display_dims = measure_text(&score_display, None, entry_font_size as u16, 1.0);
                draw_chinese_text(&score_display, score_right_x - score_display_dims.width / 2.0, current_row_center_y_for_text, entry_font_size, GOLD);
            }

            if let Some(player_rank) = &game.player_rank {
                 let player_is_in_top_10 = leaderboard.iter().any(|entry| entry.name == player_rank.name && entry.score == player_rank.score);
                 if !player_is_in_top_10 && player_rank.rank > 0 {
                    let num_displayed_entries = leaderboard.len().min(10);
                    let player_rank_area_top_y = entries_area_top_y + (num_displayed_entries as f32 * row_height) + padding_general; 
                    let player_rank_text_y_center = player_rank_area_top_y + (18.0 / 2.0) + 5.0;

                    draw_line(width*0.05, player_rank_area_top_y - (padding_general / 2.0) , width*0.95, player_rank_area_top_y - (padding_general / 2.0), 1.0, Color::new(0.5,0.5,0.5,0.3));
                    
                    let your_rank_text = format!("您的排名: {}", player_rank.rank);
                    // Pass None for font to measure_text
                    let your_rank_dims = measure_text(&your_rank_text, None, 18.0 as u16, 1.0);
                    draw_chinese_text(&your_rank_text, name_left_x + your_rank_dims.width / 2.0, player_rank_text_y_center, 18.0, LIGHTGRAY);
                    
                    let your_score_text = format!("分数: {}", player_rank.score);
                    // Pass None for font to measure_text
                    let your_score_dims = measure_text(&your_score_text, None, 18.0 as u16, 1.0);
                    draw_chinese_text(&your_score_text, score_right_x - your_score_dims.width / 2.0, player_rank_text_y_center, 18.0, LIGHTGRAY);
                 }
            }
        }
    }
}

// 辅助函数，检测鼠标是否在某个矩形区域内
fn is_mouse_in_rect(x: f32, y: f32, width: f32, height: f32) -> bool {
    let mouse_pos = mouse_position();
    mouse_pos.0 >= x && mouse_pos.0 <= x + width && mouse_pos.1 >= y && mouse_pos.1 <= y + height
}

// 绘制函数
fn draw_game(game: &mut Game) {
    // 获取DPI缩放比例
    // let dpi_scale = get_dpi_scale(); // (Removed)
    
    // 清屏为深蓝色背景 #3C569E - 匹配全局设计
    clear_background(COLOR_PRIMARY);
    
    // 绘制游戏内容
    // 计算网格尺寸和位置，考虑DPI缩放
    let grid_size = screen_width() * 0.9;
    let cell_size = grid_size / 8.0;
    let grid_offset_x = (screen_width() - grid_size) / 2.0;
    
    // 计算游戏布局 - 使用宽高比而不是绝对高度
    let aspect_ratio = screen_width() / screen_height();
    
    // 基于宽高比判断屏幕类型
    let is_wide_screen = aspect_ratio > 0.8;   // 宽屏 (接近正方形)
    let is_tall_screen = aspect_ratio < 0.5;   // 高屏 (典型手机竖屏)
    // 是否是小屏幕保留作为辅助判断
    let is_small_screen = screen_height() < 600.0;
    
    // 更好地调整顶部偏移 - 根据宽高比调整
    let grid_offset_y = if is_tall_screen {
        screen_height() * 0.22  // 从 0.18 增加到 0.22，与 draw_game 保持一致
    } else if is_wide_screen {
        screen_height() * 0.12  // 从 0.07 增加到 0.12，与 draw_game 保持一致
    } else {
        screen_height() * 0.15  // 从 0.10 增加到 0.15，与 draw_game 保持一致
    };
    
    // 绘制当前分数，字体大小根据DPI缩放
    let score_text = format!("{}", game.score);
    let text_x = screen_width() / 2.0; // 水平居中
    let text_y = grid_offset_y * 0.7;  // 使用原标题的垂直位置
    let font_size = 30.0; // 使用原标题的字体大小 (Removed dpi_scale)
    let gold_color = Color::new(1.0, 0.843, 0.0, 1.0); // 金色 (#FFD700)
    let shadow_color = Color::new(0.0, 0.0, 0.0, 0.5); // 半透明黑色阴影
    let shadow_offset = 2.0; // 阴影偏移量

    // 绘制阴影 (投影)
    draw_chinese_text(&score_text,
             text_x + shadow_offset,
             text_y + shadow_offset,
             font_size,
             shadow_color);

    // 绘制主要文本 (金色)
    draw_chinese_text(&score_text,
             text_x,
             text_y,
             font_size,
             WHITE);
    
    // 在左上角绘制最高分
    let high_score_text = format!("  {}", game.cloud_high_score.unwrap_or(0));
    let high_score_x = 20.0; // 左侧边距
    let high_score_y = grid_offset_y * 0.25; // 顶部边距
    let high_score_font_size = 20.0; // 字体大小
    
    // 绘制最高分阴影
    draw_chinese_text(&high_score_text,
             high_score_x + shadow_offset,
             high_score_y + shadow_offset,
             high_score_font_size,
             shadow_color);
             
    // 绘制最高分文本
    draw_chinese_text(&high_score_text,
             high_score_x,
             high_score_y,
             high_score_font_size,
             gold_color); 
    
    // --- 绘制 WaveManager 状态信息 ---
    let wave_phase = game.wave_manager.get_current_phase();
    let phase_text = match wave_phase {
        WavePhase::Accumulation => "阶段 - 积累",
        WavePhase::ChallengeActive(_challenge_type) => "挑战 - 方块潮", // 挑战类型现在固定为BlockFlood
        WavePhase::Relief => "阶段 - 缓和",
    };
    let phase_text_font_size = 18.0; // 字体大小
    let phase_text_x = screen_width() - 20.0; // 右侧边距
    let phase_text_y = grid_offset_y * 0.25;    // 与最高分对齐或略下方
    let phase_text_width = measure_text(phase_text, None, phase_text_font_size as u16, 1.0).width;
    // 右对齐绘制
    draw_chinese_text(
        phase_text, 
        phase_text_x - phase_text_width, // x 坐标调整为右对齐
        phase_text_y, 
        phase_text_font_size, 
        WHITE
    );
    // --- 结束 WaveManager 状态信息绘制 ---

    // 绘制游戏网格背景
    draw_rectangle(
        grid_offset_x - 5.0,
        grid_offset_y - 5.0,
        grid_size + 10.0,
        grid_size + 10.0,
        COLOR_PRIMARY_DARK // 使用深色主色
    );
    
    // 添加细边框 - 在高DPI设备上更清晰
    let border_width = 2.0; // * dpi_scale; (Removed)
    draw_rectangle_lines(
        grid_offset_x - 5.0,
        grid_offset_y - 5.0,
        grid_size + 10.0,
        grid_size + 10.0,
        border_width,
        COLOR_BORDER
    );
    
    // 绘制游戏网格 - 需要传递 WaveManager 的目标行/列信息
    // game.grid.draw(grid_offset_x, grid_offset_y, cell_size); // 旧的调用
    // let active_targets = if let WavePhase::ChallengeActive(ChallengeType::TargetRows(_)) | WavePhase::ChallengeActive(ChallengeType::TargetCols(_)) = wave_phase {
    //     Some(game.wave_manager.get_active_target_lines())
    // } else {
    //     None
    // };
    // let is_target_rows = matches!(wave_phase, WavePhase::ChallengeActive(ChallengeType::TargetRows(_)));
    
    // 由于移除了目标行/列挑战，不再需要高亮逻辑
    game.grid.draw(grid_offset_x, grid_offset_y, cell_size); 
    
    // 更新粒子效果系统
    game.effects.draw();
    
    // 绘制分隔线 - 根据屏幕宽高比调整分隔线的位置
    // 调整间距 - 基于宽高比
    let spacing = if is_tall_screen {
        60.0 // 高屏幕上使用更大间距
    } else if is_wide_screen {
        20.0 // 宽屏上使用较小间距
    } else {
        40.0 // 标准屏幕上使用中等间距
    };
    
    let separator_y = grid_offset_y + grid_size + 15.0 + spacing;
    let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
    
    // 绘制下方区域的背景
    // 确保底部区域至少有屏幕高度的一定比例
    let min_bottom_height = if is_tall_screen {
        screen_height() * 0.25 // 在高屏上提供更大的底部区域
    } else if is_wide_screen {
        screen_height() * 0.15 // 在宽屏上使用较小的底部区域
    } else {
        screen_height() * 0.20 // 标准底部区域大小
    };
    let bottom_area_height = (screen_height() - bottom_area_top).max(min_bottom_height);
    
    // 绘制当前可选方块 - 在竖屏模式下水平排列
    // 计算垂直位置，使方块位于底部区域的中间
    // let blocks_y = bottom_area_top + bottom_area_height / 2.0; // 可拖拽方块位于底部区域的垂直中心
    // 将方块位置向上移动 - 不再位于正中间，而是位于底部区域的上半部分
    let blocks_y = bottom_area_top + bottom_area_height * 0.2; // 从0.5 (中心) 减少到0.4，与 draw_game 保持一致
    
    // 计算方块布局 - 根据最大方块数量(blocks_per_generation)确定尺寸，而非当前方块数量
    // 这样即使放置了方块，剩余方块的大小也不会突然变化
    let max_block_size = cell_size * 4.0; // 最大方块尺寸
    let block_size = if game.wave_manager.blocks_per_generation <= 2 {
        max_block_size // 对于1-2个最大方块，使用最大尺寸
    } else {
        // 对于更多方块，减小尺寸以适应屏幕
        // 考虑屏幕大小，在小屏幕上进一步减小尺寸
        let width_factor = if is_small_screen { 0.80 } else { 0.85 };
        (screen_width() * width_factor) / (game.wave_manager.blocks_per_generation as f32 * 1.2)
    };
    
    let block_margin = block_size * 0.2; // 方块之间的间距根据方块大小缩放
    let total_width = block_size * game.current_blocks.len() as f32 + block_margin * (game.current_blocks.len() as f32 - 1.0);
    let start_x = (screen_width() - total_width) / 2.0;
    
    // --- 绘制底部可选方块 --- 
    for (idx, block) in game.current_blocks.iter().enumerate() { // <--- 将 _block 改为 block
        let block_pos_x = start_x + block_size/2.0 + idx as f32 * (block_size + block_margin);
        let block_pos_y = blocks_y;
        
        // 检查是否是正在返回动画的块，如果是则跳过
        if let Some(anim_data) = &game.animating_block {
            if anim_data.block_idx == idx { 
                continue; 
            }
        }
        
        // 检查是否是正在拖拽的块，如果是则绘制透明占位符
        let draw_color = if game.drag_block_idx == Some(idx) {
            Color::new(block.color.r/2.0, block.color.g/2.0, block.color.b/2.0, 0.3) // 使用 block.color
        } else {
            block.color // 使用 block.color
        };

        // 绘制方块 - 根据方块尺寸调整单元格大小
        let cell_scale = block_size / (cell_size * 5.0); // 调整单元格大小与方块大小的比例
        for &(dx, dy) in &block.cells { // 使用 block.cells
            let x = block_pos_x + dx as f32 * cell_size * cell_scale;
            let y = block_pos_y + dy as f32 * cell_size * cell_scale;
            drawing::draw_cube_block(x - cell_size * cell_scale / 2.0, y - cell_size * cell_scale / 2.0, 
                          cell_size * cell_scale, draw_color);
        }
    }
    
    // --- 绘制返回动画中的方块 --- 
    if let Some(anim_data) = &game.animating_block {
        if anim_data.block_idx < game.current_blocks.len() {
            let block = &game.current_blocks[anim_data.block_idx]; // 这里已经是 block
            // ... (rest of animation drawing logic uses block correctly) ...
            let mut min_dx = i32::MAX;
            let mut min_dy = i32::MAX;
            for &(dx, dy) in &block.cells {
                if dx < min_dx { min_dx = dx; }
                if dy < min_dy { min_dy = dy; }
            }
            for &(dx, dy) in &block.cells {
                let rel_dx = dx - min_dx;
                let rel_dy = dy - min_dy;
                let x = anim_data.current_pos.x + rel_dx as f32 * cell_size;
                let y = anim_data.current_pos.y + rel_dy as f32 * cell_size;
                drawing::draw_cube_block(x - cell_size/2.0, y - cell_size/2.0, cell_size, block.color);
            }
        }
    }
    
    // 绘制拖拽中的方块
    if let (Some(block_idx), Some(pos)) = (game.drag_block_idx, game.drag_pos) {
        if block_idx < game.current_blocks.len() {
            let block = &game.current_blocks[block_idx]; 
            
            // 找到最左上角的cell（最小x和y坐标的cell）
            let mut min_dx = i32::MAX;
            let mut min_dy = i32::MAX;
            for &(dx_cell, dy_cell) in &block.cells { // Renamed to avoid conflict with loop vars later
                if dx_cell < min_dx {
                    min_dx = dx_cell;
                }
                if dy_cell < min_dy {
                    min_dy = dy_cell;
                }
            }
            
            // --- 恢复 can_place, corrected_x, corrected_y 的计算 --- 
            // 计算左上角cell在网格中的坐标 (pos是拖拽方块左上角cell的中心)
            let grid_top_left_x = ((pos.x - grid_offset_x) / cell_size).floor();
            let grid_top_left_y = ((pos.y - grid_offset_y) / cell_size).floor();
            
            // 计算方块整体在网格中的基准坐标 (以方块自身的0,0为基准的那个cell对应到网格的哪个格子)
            let base_grid_x = grid_top_left_x as i32 - min_dx;
            let base_grid_y = grid_top_left_y as i32 - min_dy;
            
            // 判断是否在有效网格范围内进行预览检查 (可以稍微放宽，因为只是预览)
            let is_valid_for_preview_check = 
                base_grid_x > - (block.cells.iter().map(|(cx,_)|cx.abs()).max().unwrap_or(0) + 2) && 
                base_grid_x < (8 + block.cells.iter().map(|(cx,_)|cx.abs()).max().unwrap_or(0) + 2) &&
                base_grid_y > - (block.cells.iter().map(|(_,cy)|cy.abs()).max().unwrap_or(0) + 2) &&
                base_grid_y < (8 + block.cells.iter().map(|(_,cy)|cy.abs()).max().unwrap_or(0) + 2);

            let (can_place, corrected_x, corrected_y) = if is_valid_for_preview_check {
                game.grid.can_place_block_with_tolerance(block, base_grid_x, base_grid_y, 1) 
            } else {
                (false, base_grid_x, base_grid_y) // 如果完全在网格外，不尝试容错
            };
            // --- 计算结束 ---
            
            // 为所有单元格绘制预览
            for &(dx, dy) in &block.cells {
                let preview_x = grid_offset_x + (corrected_x + dx) as f32 * cell_size;
                let preview_y = grid_offset_y + (corrected_y + dy) as f32 * cell_size;
                
                if (corrected_x + dx) >= 0 && (corrected_x + dx) < 8 && (corrected_y + dy) >= 0 && (corrected_y + dy) < 8 {
                    if can_place {
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.2, 0.8, 0.2, 0.7));
                    } else {
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.8, 0.2, 0.2, 0.7));
                    }
                }
            }
            
            // 在网格上拖动时绘制方块 (实际方块，不是预览)
            // pos 是拖拽方块的左上角cell的中心
            for &(dx, dy) in &block.cells {
                let current_cell_x = pos.x + (dx - min_dx) as f32 * cell_size;
                let current_cell_y = pos.y + (dy - min_dy) as f32 * cell_size;                
                drawing::draw_cube_block(current_cell_x - cell_size/2.0, current_cell_y - cell_size/2.0, cell_size, block.color);
            }
        }
    }
    
    // 绘制菜单/游戏结束界面
    match game.state {
        GameState::MainMenu => {
            // 主菜单界面在draw_main_menu函数中单独处理
        },
        GameState::Menu => {
            // let dpi_scale = get_dpi_scale(); // (Removed)
            let width = screen_width();
            let height = screen_height();

            // 绘制半透明背景 - 使用深蓝色主色的深色版本
            draw_rectangle(0.0, 0.0, width, height, COLOR_PRIMARY_OVERLAY);
            
            // 绘制大标题
            let title_y = height * 0.25;
            draw_chinese_text("逆向俄罗斯方块", 
                     width / 2.0, 
                     title_y, 
                     40.0, // * dpi_scale, (Removed)
                     WHITE);

            // 按钮尺寸和间距
            let btn_width = 220.0; // * dpi_scale; (Removed)
            let btn_height = 55.0; // * dpi_scale; (Removed)
            let btn_spacing = 20.0; // * dpi_scale; (Removed)
            let btn_start_y = title_y + 100.0; // * dpi_scale; // 在标题下方留出更多空间 (Removed)

            // 开始游戏按钮位置
            let start_btn_x = width / 2.0 - btn_width / 2.0;
            let start_btn_y = btn_start_y;
            let start_btn_rect = Rect::new(start_btn_x, start_btn_y, btn_width, btn_height);

            // 排行榜按钮位置
            let leaderboard_btn_x = start_btn_x;
            let leaderboard_btn_y = start_btn_y + btn_height + btn_spacing;
            let leaderboard_btn_rect = Rect::new(leaderboard_btn_x, leaderboard_btn_y, btn_width, btn_height);

            // --- 绘制按钮公共函数 ---
            let draw_menu_button = |rect: Rect, text: &str, font_size: f32| {
                let mouse_pos = mouse_position().into();
                let is_hover = rect.contains(mouse_pos);
                let is_down = is_hover && is_mouse_button_down(MouseButton::Left);
                
                let base_color = Color::from_rgba(70, 70, 90, 255);
                let hover_color = Color::from_rgba(100, 100, 120, 255);
                let down_color = Color::from_rgba(50, 50, 70, 255);
                
                let btn_color = if is_down { down_color } else if is_hover { hover_color } else { base_color };
                
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, btn_color);
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, // * dpi_scale, (Removed)
                                     LIGHTGRAY);
                
                draw_chinese_text(text, 
                                  rect.x + rect.w / 2.0, 
                                  rect.y + rect.h / 2.0, // 调整Y使文本垂直居中
                                  font_size, 
                                  WHITE);
            };
            // --- 绘制按钮公共函数结束 ---

            // 绘制开始游戏按钮
            draw_menu_button(start_btn_rect, "开始游戏", 25.0); // * dpi_scale); (Removed)

            // 绘制排行榜按钮
            draw_menu_button(leaderboard_btn_rect, "排行榜", 25.0); // * dpi_scale); (Removed)
            
            // 绘制最高分
            let high_score_y = leaderboard_btn_y + btn_height + 60.0; // * dpi_scale; (Removed)
            draw_chinese_text(&format!("最高分: {}", game.cloud_high_score.unwrap_or(0)), 
                     width / 2.0, 
                     high_score_y, 
                     22.0, // * dpi_scale, (Removed)
                     Color::new(0.6, 0.8, 1.0, 1.0)); // 稍亮的蓝色
        },
        GameState::GameOver => {
            // 绘制半透明背景
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), COLOR_PRIMARY_OVERLAY);
            
            // 绘制游戏结束文本
            draw_chinese_text("游戏结束", 
                     screen_width() / 2.0, 
                     screen_height() / 3.0, 
                     30.0, // * dpi_scale, (Removed)
                     WHITE);
            
            // 绘制最终得分
            draw_chinese_text(&format!("最终得分: {}", game.score), 
                     screen_width() / 2.0, 
                     screen_height() / 2.0, 
                     25.0, // * dpi_scale, (Removed)
                     Color::new(1.0, 0.8, 0.2, 1.0));
            
            // 绘制最高分
            let new_record = game.score > game.cloud_high_score.unwrap_or(0); // 与云端比较
            let high_score_text = if new_record {
                format!("新纪录! {}", game.score)
            } else {
                // format!("最高分: {}", game.save_data.high_score)
                format!("最高分: {}", game.cloud_high_score.unwrap_or(0)) // 使用云端最高分
            };
            
            draw_chinese_text(&high_score_text, 
                     screen_width() / 2.0, 
                     screen_height() / 2.0 + 40.0, 
                     22.0, // * dpi_scale, (Removed)
                     if new_record { Color::new(1.0, 0.5, 0.0, 1.0) } else { Color::new(0.2, 0.8, 1.0, 1.0) });
            
            // 绘制重新开始提示
            draw_chinese_text("点击重新开始", 
                     screen_width() / 2.0, 
                     screen_height() / 2.0 + 100.0, 
                     25.0, // * dpi_scale, (Removed)
                     WHITE);

            // GameOver状态下显示排行榜按钮
            if game.show_leaderboard_button {
                let btn_width = 200.0;
                let btn_height = 40.0;
                let btn_x = screen_width() / 2.0 - btn_width / 2.0;
                let btn_y = screen_height() * 0.7; // 从底部向上30%的位置
                let btn_rect = Rect::new(btn_x, btn_y, btn_width, btn_height);
                
                // 绘制按钮
                draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, DARKGRAY);
                draw_rectangle_lines(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, 2.0, WHITE);
                
                // 按钮文字
                draw_chinese_text(
                    "查看排行榜", 
                    btn_rect.x + btn_rect.w / 2.0, 
                    btn_rect.y + btn_rect.h / 2.0, 
                    18.0,
                    WHITE
                );
                
                // 如果分数已上传，显示一个提示
                if game.score_uploaded {
                    draw_chinese_text(
                        "分数已上传", 
                        screen_width() / 2.0, 
                        btn_y + btn_height + 20.0,
                        14.0,
                        GREEN
                    );
                }
            }
        },
        GameState::Leaderboard => {
            // 排行榜状态下，不需要任何更新逻辑
            // 按钮处理放在run_game中
        },
        GameState::Playing => { 
            // Playing state content is drawn before this match statement.
            // This arm is to make the match exhaustive.
            // No additional overlays for Playing state here usually.
        } 
        // Consider if a catch-all like `_ => {}` was intended if other states might exist
        // but given the error, `Playing` is the specific one missing.
    }
}

// 更新游戏状态
fn update_game(game: &mut Game) {
    let dt = get_frame_time(); // 获取时间增量
    
    // 更新粒子效果
    game.effects.update(dt);
    
    // 更新返回动画
    if let Some(anim_data) = game.animating_block.as_mut() {
        anim_data.timer += dt / anim_data.duration;
        if anim_data.timer >= 1.0 {
            // 动画结束
            game.animating_block = None;
        } else {
            // 线性插值计算当前位置
            let t = anim_data.timer;
            anim_data.current_pos = anim_data.start_pos.lerp(anim_data.target_pos, t);
            // 可以添加缓动函数，例如 ease-out: t * (2.0 - t)
            // let t = t * (2.0 - t); 
            // anim_data.current_pos = anim_data.start_pos.lerp(anim_data.target_pos, t);
        }
        // 如果动画正在进行，则不处理其他游戏逻辑（可选，但可以防止动画期间的干扰）
        // return; 
    }
    
    // 检测按空格键切换难度模式
    if is_key_pressed(KeyCode::Space) {
        // 循环切换模式
        game.game_mode = match game.game_mode {
            GameMode::Happy => GameMode::Easy,
            GameMode::Easy => GameMode::Normal,
            GameMode::Normal => GameMode::Happy,
        };
        log_info!("模式切换为: {:?}", game.game_mode);
    }
    
    // 移除调整概率和生成数量的代码，因为模式现在由池控制
    /*
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
    */
    
    match game.state {
        GameState::MainMenu => {
            // 主菜单状态下不需要游戏更新逻辑
            return;
        },
        GameState::Menu => {
            if is_mouse_button_pressed(MouseButton::Left) {
                let width = screen_width();
                let height = screen_height();
                let mouse_pos: Vec2 = mouse_position().into();

                // 按钮尺寸和位置计算（与draw_game中保持一致）
                let title_y = height * 0.25;
                let btn_width = 220.0; // * dpi_scale; (Removed)
                let btn_height = 55.0; // * dpi_scale; (Removed)
                let btn_spacing = 20.0; // * dpi_scale; (Removed)
                let btn_start_y = title_y + 100.0; // * dpi_scale; // 在标题下方留出更多空间 (Removed)

                let start_btn_x = width / 2.0 - btn_width / 2.0;
                let start_btn_y = btn_start_y;
                let start_btn_rect = Rect::new(start_btn_x, start_btn_y, btn_width, btn_height);

                let leaderboard_btn_x = start_btn_x;
                let leaderboard_btn_y = start_btn_y + btn_height + btn_spacing;
                let leaderboard_btn_rect = Rect::new(leaderboard_btn_x, leaderboard_btn_y, btn_width, btn_height);

                // 检查是否点击了开始游戏按钮
                if start_btn_rect.contains(mouse_pos) {
                    game.start_transition(GameState::Playing);
                    game.grid = grid::Grid::new(); // 重置网格
                    game.score = 0;
                    game.combo = 0;
                    game.score_uploaded = false; // 重置上传状态
                    game.accumulated_shape_counts.clear(); // 清空累积统计
                    game.generate_blocks();
                    // 清空拖拽状态，以防万一
                    game.drag_block_idx = None;
                    game.drag_pos = None;
                }
                // 检查是否点击了排行榜按钮
                else if leaderboard_btn_rect.contains(mouse_pos) {
                    log_info!("排行榜按钮被点击"); 
                    // 使用过渡动画进入排行榜页面
                    game.start_transition(GameState::Leaderboard);
                }
            }
        },
        GameState::Playing => {
            // 获取鼠标位置
            let mouse_pos: Vec2 = mouse_position().into();
            
            // 计算网格位置和大小
            let grid_size = screen_width() * 0.9;
            let cell_size = grid_size / 8.0;
            let grid_offset_x = (screen_width() - grid_size) / 2.0;
            
            // 更新计算网格位置 - 使用宽高比
            let aspect_ratio = screen_width() / screen_height();
            
            // 基于宽高比判断屏幕类型
            let is_wide_screen = aspect_ratio > 0.8;   // 宽屏 (接近正方形)
            let is_tall_screen = aspect_ratio < 0.5;   // 高屏 (典型手机竖屏)
            let is_small_screen = screen_height() < 600.0;
            
            // 使用与 draw_game 相同的顶部偏移计算
            let grid_offset_y = if is_tall_screen {
                screen_height() * 0.22  // 从 0.18 增加到 0.22，与 draw_game 保持一致
            } else if is_wide_screen {
                screen_height() * 0.12  // 从 0.07 增加到 0.12，与 draw_game 保持一致
            } else {
                screen_height() * 0.15  // 从 0.10 增加到 0.15，与 draw_game 保持一致
            };
            
            // 更新分隔线和间距计算 - 使用宽高比
            let spacing = if is_tall_screen {
                60.0 // 高屏幕上使用更大间距
            } else if is_wide_screen {
                20.0 // 宽屏上使用较小间距
            } else {
                40.0 // 标准屏幕上使用中等间距
            };
            
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
                        let block = &game.current_blocks[block_idx];
                        
                        // 找到最左上角的cell（最小x和y坐标的cell）
                        let mut min_dx = i32::MAX;
                        let mut min_dy = i32::MAX;
                        for &(dx, dy) in &block.cells {
                            if dx < min_dx { min_dx = dx; }
                            if dy < min_dy { min_dy = dy; }
                        }
                        
                        // 核心改动：应用偏移量使方块位于手指上方
                        // let adjusted_pos = Vec2::new(
                        //     mouse_pos.x + game.drag_offset.x,
                        //     mouse_pos.y + game.drag_offset.y
                        // ); // 旧的 adjusted_pos 计算方式
                        
                        // 计算方块几何中心 和 从几何中心到左上角cell的偏移 (scaled by cell_size)
                        let min_dx_all = block.cells.iter().map(|(dx, _)| *dx).min().unwrap_or(0);
                        let max_dx_all = block.cells.iter().map(|(dx, _)| *dx).max().unwrap_or(0);
                        let min_dy_all = block.cells.iter().map(|(_, dy)| *dy).min().unwrap_or(0);
                        let max_dy_all = block.cells.iter().map(|(_, dy)| *dy).max().unwrap_or(0);
                        let _center_x = (min_dx_all + max_dx_all) as f32 / 2.0; // Relative to block's own 0,0
                        let _center_y = (min_dy_all + max_dy_all) as f32 / 2.0; // Relative to block's own 0,0
                        
                        let offset_to_top_left_x = (min_dx as f32 - _center_x) * cell_size;
                        let offset_to_top_left_y = (min_dy as f32 - _center_y) * cell_size;
                        let block_geometry_to_top_left_offset = Vec2::new(offset_to_top_left_x, offset_to_top_left_y);

                        let fixed_upward_offset_vector = game.drag_offset; // This is (0, -cell_size * 2.0)

                        // --- 计算基础视觉位置 (不含动态补偿) ---
                        // The anchor for the block (e.g. its center) is mouse_pos + fixed_upward_offset_vector
                        // The top-left cell of the block is then offset from this anchor by block_geometry_to_top_left_offset
                        let base_visual_anchor = mouse_pos + fixed_upward_offset_vector;
                        let base_top_left_cell_center = base_visual_anchor + block_geometry_to_top_left_offset;
                        
                        // --- 计算动态拖动补偿 --- 
                        let mut compensation_vector = Vec2::ZERO;
                        if let Some(initial_drag_mouse_pos) = game.initial_mouse_drag_start_pos {
                            let drag_delta_vector = mouse_pos - initial_drag_mouse_pos;
                            let drag_distance = drag_delta_vector.length();

                            if drag_distance > 1.0 { // 避免小距离抖动或除零
                                let distance_for_max_compensation_effect = (screen_width().min(screen_height())) * 0.35; // 40% of smaller screen dim
                                let max_compensation_magnitude = cell_size * 2.0; // 最大额外偏移2.0个格子

                                let effective_drag_distance = drag_distance.min(distance_for_max_compensation_effect);
                                
                                let compensation_factor = (effective_drag_distance / distance_for_max_compensation_effect).powf(1.2); // 轻微加速
                                let current_compensation_magnitude = compensation_factor * max_compensation_magnitude;
                                
                                compensation_vector = drag_delta_vector.normalize_or_zero() * current_compensation_magnitude;
                            }
                        }
                        // --- 动态拖动补偿计算结束 ---

                        // 设置最终的拖拽位置 (包含固定偏移和动态补偿)
                        game.drag_pos = Some(base_top_left_cell_center + compensation_vector);

                    } else {
                        // 索引无效，重置拖拽状态
                        game.drag_block_idx = None;
                        game.drag_pos = None;
                        game.initial_mouse_drag_start_pos = None; // 清理状态
                    }
                } else {
                    // 这个分支不应该发生，但以防万一
                    game.drag_pos = None;
                    game.initial_mouse_drag_start_pos = None; // 清理状态
                }
            }
            
            // 在鼠标释放时处理方块放置
            if is_mouse_button_released(MouseButton::Left) && game.drag_block_idx.is_some() {
                if let Some(block_idx) = game.drag_block_idx {
                    if let Some(pos) = game.drag_pos { // pos is the top-left cell's center
                        // Ensure block index is still valid after potential removals
                        if block_idx < game.current_blocks.len() {
                            let block = &game.current_blocks[block_idx];
                            
                            // --- 网格计算 (与 draw_game 一致) ---
                            let grid_size = screen_width() * 0.9;
                            let cell_size = grid_size / 8.0;
                            let grid_offset_x = (screen_width() - grid_size) / 2.0;
                            
                            // 修正：使用与其他地方相同的网格偏移计算
                            let aspect_ratio = screen_width() / screen_height();
                            let is_wide_screen = aspect_ratio > 0.8;   // 宽屏 (接近正方形)
                            let is_tall_screen = aspect_ratio < 0.5;   // 高屏 (典型手机竖屏)
                            
                            // 使用与其他地方相同的网格偏移计算
                            let grid_offset_y = if is_tall_screen {
                                screen_height() * 0.22
                            } else if is_wide_screen {
                                screen_height() * 0.12
                            } else {
                                screen_height() * 0.15
                            };
                            // --- 网格计算结束 ---
                            
                            // --- 原始方块位置计算 (与 draw_game 绘制底部方块一致) ---
                            let is_small_screen = screen_height() < 600.0;
                            let spacing = if is_small_screen { 20.0 } else { 30.0 };
                            let separator_y = grid_offset_y + grid_size + 15.0 + spacing;
                            let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
                            let min_bottom_height = screen_height() * 0.2;
                            let bottom_area_height = (screen_height() - bottom_area_top).max(min_bottom_height);
                            let blocks_y = bottom_area_top + bottom_area_height / 2.0;
                            let max_block_size_calc = cell_size * 4.0;
                            let block_size_calc = if game.wave_manager.blocks_per_generation <= 2 {
                                max_block_size_calc
                            } else {
                                let width_factor = if is_small_screen { 0.80 } else { 0.85 };
                                (screen_width() * width_factor) / (game.wave_manager.blocks_per_generation as f32 * 1.2)
                            };
                            let block_margin_calc = block_size_calc * 0.2;
                            // 注意：这里需要计算所有可能位置的总宽度，即使某些块已被移除
                            let total_possible_width = block_size_calc * game.wave_manager.blocks_per_generation as f32 
                                                     + block_margin_calc * (game.wave_manager.blocks_per_generation as f32 - 1.0);
                            let start_x_calc = (screen_width() - total_possible_width) / 2.0;
                            // 计算当前拖动块的原始目标位置中心点 (需要假设它在 current_blocks 中的原始位置)
                            // TODO: This calculation might be complex if blocks are removed. A better way might be needed.
                            // Let's assume for now we calculate based on its *current* index if it were present
                            // Need a reliable way to find original position index if blocks_per_generation changes
                            // --- Simplified target pos calculation for now --- 
                            let target_pos_x = start_x_calc + block_size_calc / 2.0 + block_idx as f32 * (block_size_calc + block_margin_calc);
                            let target_pos_y = blocks_y;
                            let original_bottom_pos = Vec2::new(target_pos_x, target_pos_y);
                            // --- 原始方块位置计算结束 ---
                            
                            // 找到最左上角的cell (用于计算网格坐标)
                            let mut min_dx = i32::MAX;
                            let mut min_dy = i32::MAX;
                            for &(dx, dy) in &block.cells {
                                if dx < min_dx { min_dx = dx; }
                                if dy < min_dy { min_dy = dy; }
                            }
                            
                            // 计算目标网格坐标 (以左上角cell为基准)
                            let grid_top_left_x = ((pos.x - grid_offset_x) / cell_size).floor();
                            let grid_top_left_y = ((pos.y - grid_offset_y) / cell_size).floor();
                            let grid_x = grid_top_left_x as i32 - min_dx;
                            let grid_y = grid_top_left_y as i32 - min_dy;
                            
                            // 检查是否在网格或附近
                            let is_near_grid = pos.x >= grid_offset_x - cell_size 
                                             && pos.x <= grid_offset_x + grid_size + cell_size
                                             && pos.y >= grid_offset_y - cell_size
                                             && pos.y <= grid_offset_y + grid_size + cell_size;
                            
                            let mut placed_successfully = false;
                            if is_near_grid { // 只在靠近网格时尝试放置
                                let (can_place, corrected_x, corrected_y) = 
                                    game.grid.can_place_block_with_tolerance(block, grid_x, grid_y, 1);
                                
                                if can_place {
                                    placed_successfully = true;
                                    game.grid.place_block(block, corrected_x, corrected_y);
                                    // trigger_vibration_on_place(5); // <-- 移除旧的震动调用
                                    
                                    // --- WaveManager 回合推进和奖励获取 ---
                                    let score_bonus_from_wave = game.wave_manager.increment_turn();
                                    if score_bonus_from_wave > 0 {
                                        game.score += score_bonus_from_wave;
                                        log_info!("WaveManager bonus: +{} score! Total score: {}", score_bonus_from_wave, game.score);
                                        // TODO: 可以在此触发奖励特效/音效
                                    }
                                    // --- 结束 --- 

                                    if corrected_x != grid_x || corrected_y != grid_y {
                                        log_info!("位置已自动校正: 从({},{})到({},{})", grid_x, grid_y, corrected_x, corrected_y);
                                    }
                                    
                                    let (cleared_row_indices, cleared_col_indices) = game.grid.check_and_clear();
                                    let cleared_count = cleared_row_indices.len() + cleared_col_indices.len();

                                    // --- 新的震动和特效逻辑 ---
                                    if cleared_count > 1 {
                                        // 多行消除：强特效和长震动
                                        // game.effects.show_screen_flash(GOLD, 0.25); // <-- 移除全屏闪光
                                        trigger_vibration_on_place(20);
                                        
                                        let text_to_show = format!("X {}", cleared_count);
                                        // 计算网格中心位置作为特效出现点
                                        let grid_size = screen_width() * 0.9;
                                        let grid_offset_y = screen_height() * 0.15; // 使用一个简化的Y偏移估算
                                        let effect_pos = Vec2::new(screen_width() / 2.0, grid_offset_y + grid_size / 2.0);
                                        
                                        game.effects.show_floating_text(
                                            text_to_show,
                                            effect_pos,
                                            WHITE, // 使用白色，与闪光背景形成对比
                                            60.0,  // <-- 将字体大小从 48.0 增加到 60.0
                                            1.5    // 持续1.5秒
                                        );
                                        // --- 结束 ---

                                    } else if cleared_count == 1 {
                                        // 单行消除：中等震动
                                        trigger_vibration_on_place(10);
                                    } else {
                                        // 仅放置，无消除：短震动
                                        trigger_vibration_on_place(5);
                                    }
                                    
                                    if cleared_count > 0 {
                                        for &index in &cleared_row_indices {
                                             game.wave_manager.notify_line_cleared(index, true);
                                        }
                                        for &index in &cleared_col_indices {
                                            game.wave_manager.notify_line_cleared(index, false);
                                        }
                                        log_info!("Lines cleared ({}) - Notified WaveManager", cleared_count);
                                        
                                        // --- 新的、区分强度的局部特效 ---
                                        let effect_color = GOLD; // 统一使用金色
                                        
                                        let grid_size = screen_width() * 0.9;
                                        let cell_size = grid_size / 8.0;
                                        let grid_offset_x = (screen_width() - grid_size) / 2.0;
                                        // 确保这里的 grid_offset_y 与 draw_game 中的一致
                                        let aspect_ratio = screen_width() / screen_height();
                                        let is_tall_screen = aspect_ratio < 0.5;
                                        let is_wide_screen = aspect_ratio > 0.8;
                                        let grid_offset_y = if is_tall_screen {
                                            screen_height() * 0.22
                                        } else if is_wide_screen {
                                            screen_height() * 0.12
                                        } else {
                                            screen_height() * 0.15
                                        };

                                        // 根据消除行数决定使用哪个特效函数
                                        let effect_fn = if cleared_count > 1 {
                                            Effects::show_multi_clear_effect
                                        } else {
                                            Effects::show_clear_effect
                                        };

                                        for &y_idx in &cleared_row_indices {
                                            for x_idx in 0..8 {
                                                let effect_x = grid_offset_x + x_idx as f32 * cell_size + cell_size / 2.0;
                                                let effect_y = grid_offset_y + y_idx as f32 * cell_size + cell_size / 2.0;
                                                effect_fn(&mut game.effects, effect_x, effect_y, effect_color);
                                            }
                                        }
                                        for &x_idx in &cleared_col_indices {
                                            for y_idx in 0..8 {
                                                // 避免在行列交叉点重复产生特效
                                                if !cleared_row_indices.contains(&y_idx) {
                                                    let effect_x = grid_offset_x + x_idx as f32 * cell_size + cell_size / 2.0;
                                                    let effect_y = grid_offset_y + y_idx as f32 * cell_size + cell_size / 2.0;
                                                    effect_fn(&mut game.effects, effect_x, effect_y, effect_color);
                                                }
                                            }
                                        }
                                        
                                        if game.combo >= 2 { 
                                            let combo_x = screen_width() / 2.0; 
                                            let combo_y = grid_offset_y + grid_size / 2.0; 
                                            game.effects.show_combo_effect(game.combo, combo_x, combo_y); 
                                        }
                                        
                                        // 分数和连击
                                        game.combo += 1;
                                        // 新的计分方式...
                                        let base_score_for_turn = match cleared_count { // Use count here
                                            0 => 0, // 不应发生
                                            1 => 100,
                                            2 => 300,
                                            3 => 500,
                                            4 => 800,
                                            c => 800 + (c - 4) as u32 * 300, // Ensure calculation is u32
                                        };
                                        game.score += base_score_for_turn * game.combo;

                                    } else {
                                        game.combo = 0;
                                    }
                                    // --- 消除和得分逻辑结束 ---
                                    
                                    // 移除已使用的方块 - Important: do this *after* using block_idx
                                    game.current_blocks.remove(block_idx);
                                    
                                    // 如果没有方块了，生成新的
                                    if game.current_blocks.is_empty() {
                                        game.generate_blocks();
                                    }
                                    
                                    // Check for game over *after* potentially generating new blocks
                                    // if game.check_game_over() { // Moved game over check lower
                                    //    game.state = GameState::GameOver;
                                    // }

                                } // end if can_place
                            } // end if is_near_grid
                            
                            // 如果放置失败，启动返回动画
                            if !placed_successfully {
                                // ... (return animation logic) ...
                                log_info!("Block placement failed or cancelled.");
                                // Note: Turn is NOT incremented if placement fails
                            }
                        } else {
                             log_warn!("Skipping placement logic because block_idx {} is out of bounds for current_blocks len {}.", block_idx, game.current_blocks.len());
                        } // end if block_idx < game.current_blocks.len()
                    } // end if let Some(pos)
                    
                    // 重置拖拽状态 (无论成功与否)
                    game.drag_block_idx = None;
                    game.drag_pos = None;
                    game.drag_original_pos = None; // 清理状态
                    game.initial_mouse_drag_start_pos = None; // 清理拖动起始位置
                } // end if let Some(block_idx)
            } // end if is_mouse_button_released
            
            // --- !!! STEP 1 (Part 2): Handle Obstacle Generation Request !!! --- 
            // Check this every frame while playing, not just after placement
            // if let Some(count) = game.wave_manager.check_and_consume_obstacle_request() {
            //     log_info!("Obstacle generation requested: count = {}", count);
            //     let empty_cells = game.grid.get_random_empty_cells(count); 
            //     // ... (rest of old obstacle placement) ...
            // }

            // 检查游戏结束 - Check AFTER all placement logic and potential block generation
            if !game.current_blocks.is_empty() && game.check_game_over() {
                game.state = GameState::GameOver;
                log_info!("Game Over condition met.");
            }
        }, // End GameState::Playing
        GameState::GameOver => {
            if is_mouse_button_pressed(MouseButton::Left) {
                let width = screen_width();
                let height = screen_height();
                let mouse_pos: Vec2 = mouse_position().into();

                // 重新开始按钮的隐形区域（整个屏幕，除了排行榜按钮）
                let leaderboard_btn_width = 200.0; // * dpi_scale; // 应用DPI (Removed)
                let leaderboard_btn_height = 40.0; // * dpi_scale; // 应用DPI (Removed)
                let leaderboard_btn_x = width / 2.0 - leaderboard_btn_width / 2.0;
                // 注意：这里的 Y 坐标要和 draw_game 中绘制排行榜按钮的 Y 坐标匹配
                // 在 draw_game 的 GameOver 分支中，是 height / 2.0 + 80.0 (原始值) 
                // 但我们需要使用经过DPI缩放的值进行比较，或者重新计算draw_game中的值
                // 暂时假设draw_game中GameOver的按钮位置是相对于中心点固定的
                let leaderboard_btn_y = height / 2.0 + 100.0; // 调整了draw_game GameOver按钮位置，这里保持一致
                let leaderboard_btn_rect = Rect::new(leaderboard_btn_x, leaderboard_btn_y, leaderboard_btn_width, leaderboard_btn_height);
                
                // 检查是否点击了排行榜按钮
                if game.show_leaderboard_button && leaderboard_btn_rect.contains(mouse_pos) {
                    game.start_transition(GameState::Leaderboard);
                    // 加载操作由run_game处理
                } else {
                    // 点击其他区域则重新开始
                    game.start_transition(GameState::Menu); // 返回菜单而不是直接开始游戏
                    // 游戏状态的重置将在进入Menu或Playing状态时处理
                    // 清空累积统计也应该在这里，如果从 GameOver 直接到 Menu 再到 Playing
                    game.accumulated_shape_counts.clear(); 
                }
            }
        },
        GameState::Leaderboard => {
            // 排行榜状态下，不需要任何更新逻辑
            // 按钮处理放在run_game中
        }
    }
}

// macroquad窗口配置函数
fn window_conf() -> Conf {
    Conf {
        window_title: "方块消除游戏".to_string(),
        window_width: 400,  // 保留宽度用于计算初始纵横比，但高度由CSS控制
        // window_height: 600, // <-- 移除固定高度
        high_dpi: true,      // 保留高DPI支持
        sample_count: 1,      // 默认值
        window_resizable: false, // 保持不可调整大小
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // 设置随机数种子 - 这里可能就是调用 SystemTime::now() 的地方
    #[cfg(target_arch = "wasm32")]
    {
        // 对于WASM，我们需要一个不同的方式来获取种子，因为SystemTime::now()可能不可靠
        // 使用js Date.now() 作为种子的一部分
        let seed_from_js = unsafe {
            std::mem::transmute::<f64, u64>(miniquad::date::now()) 
        };
        mq_rand::srand(seed_from_js);
        // log_info!("WASM 随机数种子已使用 JS Date.now() 设置: {}", seed_from_js); // Temporarily comment out for debugging panic

        // 尝试初始化panic hook
        wasm_init(); // 调用我们之前定义的panic hook设置
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 使用当前时间戳作为随机数种子
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| {
                // 如果时间在 UNIX EPOCH 之前（不太可能，但作为健壮性处理）
                // 或者 unwrap_or_else 用于其他潜在错误
                log_warn!("SystemTime before UNIX EPOCH or other error! Using default seed. Error: {}", e);
                std::time::Duration::from_secs(0) // 回退到0，或某个固定值
            })
            .as_nanos() as u64; // 使用纳秒部分增加随机性

        mq_rand::srand(seed);
        log_info!("Random seed initialized with: {}", seed);
    }

    // 显示设备信息和DPI缩放
    let _dpi_scale = get_dpi_scale();
    log_info!("设备信息: 屏幕大小 {}x{}, DPI缩放: {}", screen_width(), screen_height(), _dpi_scale);
    
    // iOS设备相关日志
    #[cfg(target_os = "ios")]
    log_info!("在iOS设备上运行，使用3.0倍DPI缩放");
    
    // 使用嵌入的字体数据加载字体，而不是从文件系统加载
    match load_ttf_font_from_bytes(CHINESE_FONT_DATA) {
        Ok(font) => {
            *CHINESE_FONT.lock().unwrap() = Some(font);
            log_info!("成功加载中文字体");
        },
        Err(err) => {
            log_error!("无法加载中文字体: {:?}", err);
            // 在WASM环境中不触发panic
            #[cfg(target_arch = "wasm32")]
            log_info!("在WASM环境中继续运行，将使用默认字体");
        }
    }
    
    // 启动游戏
    run_game().await;
}

// 主要游戏逻辑函数
async fn run_game() {
    let mut game = Game::new();
    let mut previous_state = game.state; // 跟踪上一个状态
    
    // 异步加载标题图像
    match load_texture("resources/asset/image/title.png").await { // 修改路径
        Ok(texture) => {
            game.title_texture = Some(texture);
            log_info!("成功加载标题图像: resources/asset/image/title.png");
        }
        Err(e) => {
            log_error!("加载标题图像失败: {:?}. 将使用文本标题作为后备。", e);
            game.title_texture = None;
        }
    }

    // 初始化云服务
    game.init_cloud().await;
    
    // 尝试获取玩家初始最高分 (如果云服务初始化成功)
    if game.show_leaderboard_button {
        log_info!("尝试获取玩家初始云端最高分...");
        match cloud::get_player_rank().await {
            Ok(_) => {
                // 从全局状态获取数据
                let (_, _, _, player_rank) = cloud::get_leaderboard_data();
                if let Some(rank) = player_rank {
                    game.cloud_high_score = Some(rank.score);
                    log_info!("获取到初始云端最高分: {}", rank.score);
                } else {
                    log_info!("玩家尚无云端排名/分数记录。");
                    game.cloud_high_score = Some(0); // 认为是 0
                }
            },
            Err(e) => {
                log_error!("获取玩家初始排名/分数失败: {}", e);
                // 在获取失败时，可以暂时将其视为0，允许玩家设置第一个分数
                game.cloud_high_score = Some(0); 
            }
        }
    } else {
        log_warn!("云服务未初始化，无法获取初始最高分，将使用 0。");
        game.cloud_high_score = Some(0); // 云服务未连接，视为 0
    }

    // macroquad没有直接的set_target_fps函数
    // 可以改用适当的帧速率控制
    
    // 主游戏循环
    loop {
        // 更新过渡动画
        let transition_completed = game.update_transition(get_frame_time());
        
        // 状态转换逻辑: 在进入新状态时执行一次性操作
        if transition_completed || game.state != previous_state {
            match game.state {
                GameState::MainMenu => {
                    log_info!("进入主菜单状态");
                    // 重置一些状态或设置主菜单初始值
                },
                GameState::Playing => {
                    // 从菜单或游戏结束进入游戏状态时，重置一些数据
                    // （大部分重置已在update_game中处理）
                    log_info!("进入游戏状态");
                },
                GameState::Leaderboard => {
                    // 进入排行榜状态时，加载数据
                    log_info!("进入排行榜状态，加载数据...");
                    
                    // 先标记为加载中，防止在加载期间出现闪烁
                    game.is_leaderboard_loading = true;
                    
                    // 在过渡动画完成后再异步加载数据
                    if transition_completed {
                        game.load_leaderboard().await;
                        
                        // 如果云服务未初始化，尝试初始化
                        if !game.show_leaderboard_button {
                            log_info!("排行榜服务未初始化，尝试初始化...");
                            match cloud::initialize_sdk().await {
                                Ok(_) => {
                                    game.show_leaderboard_button = cloud::is_cloud_initialized();
                                    if game.show_leaderboard_button {
                                        // 初始化成功，加载排行榜数据
                                        log_info!("云服务初始化成功，加载排行榜数据");
                                        game.load_leaderboard().await;
                                    } else {
                                        log_warn!("云服务初始化仍未成功");
                                    }
                                },
                                Err(e) => {
                                    log_error!("云服务初始化失败: {}", e);
                                }
                            }
                        }
                    }
                },
                GameState::GameOver => {
                    // 进入游戏结束状态时，检查是否需要上传最终分数
                    log_info!("进入游戏结束状态");
                    // 检查是否是新高分 (与云端比较)
                    if game.score > game.cloud_high_score.unwrap_or(0) {
                         log_info!("游戏结束时检测到新高分! {} -> {}. 准备上传...", game.cloud_high_score.unwrap_or(0), game.score);
                         game.cloud_high_score = Some(game.score); // 更新本地缓存的最高分
                         // 移除本地保存调用: game.save_data.save();
                         
                         // 如果云服务可用且分数尚未上传，则上传
                         if game.show_leaderboard_button && !game.score_uploaded {
                             game.upload_score().await; // 等待上传完成
                         }
                    } else {
                        log_info!("最终分数 {} 未超过云端最高分 {}", game.score, game.cloud_high_score.unwrap_or(0));
                    }
                    // 确保即使不是新高分，如果之前上传失败了也再试一次 (或者移除这个逻辑?)
                    // if game.show_leaderboard_button && !game.score_uploaded {
                    //     game.upload_score().await;
                    // }
                },
                GameState::Menu => {
                    log_info!("返回菜单");
                    // 如果需要，可以在这里重置特定菜单状态
                }
            }
            previous_state = game.state; // 更新上一个状态
        }

        // 清理屏幕
        clear_background(COLOR_PRIMARY);

        // 更新和绘制当前状态
        if game.transition_active {
            if !game.transition_phase {
                // 淡出阶段：绘制旧页面并应用淡出效果
                match game.transition_from {
                    GameState::MainMenu => draw_main_menu(&mut game),
                    GameState::Menu => draw_menu(&mut game),
                    GameState::Playing => {
                        update_game(&mut game);
                        draw_game(&mut game);
                    },
                    GameState::GameOver => {
                        draw_game(&mut game);
                        update_game(&mut game);
                    },
                    GameState::Leaderboard => {
                        draw_leaderboard(&mut game);
                    }
                }
            } else {
                // 淡入阶段：绘制新页面并应用淡入效果
                match game.state {
                    GameState::MainMenu => {
                        game.update_main_menu_animations(get_frame_time());
                        draw_main_menu(&mut game);
                    },
                    GameState::Menu => draw_menu(&mut game),
                    GameState::Playing => {
                        update_game(&mut game);
                        draw_game(&mut game);
                    },
                    GameState::GameOver => {
                        draw_game(&mut game);
                        update_game(&mut game);
                    },
                    GameState::Leaderboard => {
                        draw_leaderboard(&mut game);
                    }
                }
            }
            
            // 绘制黑色遮罩实现淡入淡出效果
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), 
                           Color::new(0.0, 0.0, 0.0, game.transition_alpha));
        } else {
            // 正常绘制当前状态
            match game.state {
                GameState::MainMenu => {
                    // 更新主菜单动画
                    game.update_main_menu_animations(get_frame_time());
                    // 绘制主菜单
                    draw_main_menu(&mut game);
                },
                GameState::Menu => draw_menu(&mut game),
                GameState::Playing => {
                    update_game(&mut game);
                    draw_game(&mut game);
                },
                GameState::GameOver => {
                    // 绘制游戏结束界面
                    draw_game(&mut game); 
                    // 更新逻辑（处理重新开始或查看排行榜点击）
                    update_game(&mut game); // update_game 会根据点击改变状态
                },
                GameState::Leaderboard => {
                    // 绘制排行榜界面
                    draw_leaderboard(&mut game); // draw_leaderboard 处理返回按钮点击并改变状态
                    // 排行榜状态的更新逻辑（例如滚动）可以在这里添加，
                    // 但目前返回按钮在draw_leaderboard内部处理
                    update_game(&mut game); // 让update有机会处理Leaderboard状态（虽然目前为空）
                }
            }
        }
        
        // 在主循环中处理延迟的分数上传
        if game.needs_score_upload && game.show_leaderboard_button && !game.score_uploaded {
            log_info!("处理延迟的分数上传...");
            // 注意：upload_score 内部会设置 score_uploaded = true
            game.upload_score().await; 
            game.needs_score_upload = false; // 重置标志
        }
        
        // --- 将绘制代码移动到这里，就在 next_frame 之前 ---
        let jit_status_text = &get_wasm_jit_status();

        // 使用 draw_text (默认左对齐) 绘制
        // draw_text(jit_status_text, 10.0, 55.0, 18.0, Color::new(0.0, 1.0, 1.0, 1.0)); // 青色
        // --- 绘制代码结束 ---
        
        // 等待下一帧
        next_frame().await;
    }
}

// 新增绘制主菜单界面的函数
fn draw_main_menu(game: &mut Game) {
    // 清屏为深蓝色背景 #3C569E - 匹配全局设计
    clear_background(COLOR_PRIMARY);
    
    // 添加渐变背景效果
    let top_color = Color::new(40.0/255.0, 72.0/255.0, 160.0/255.0, 1.0);    //rgb(40, 72, 160) 较深的蓝色
    let bottom_color = Color::new(68.0/255.0, 118.0/255.0, 226.0/255.0, 1.0);  //rgb(68, 118, 226)  较亮的蓝色
    drawing::draw_vertical_gradient(0.0, 0.0, screen_width(), screen_height(), top_color, bottom_color);
    
    let width = screen_width();
    let height = screen_height();
    
    let center_x = width / 2.0;
    
    // 标题的基础Y轴位置 (用于布局，不含动画)
    let title_layout_y_base = height * 0.2;
    // 标题绘制时应用的Y轴位置 (包含动画)
    let title_draw_y = title_layout_y_base + game.title_bounce;
    
    let mut title_layout_bottom_y = title_layout_y_base; // 用于后续元素的布局基准

    if let Some(texture) = &game.title_texture {
        let original_aspect_ratio = texture.width() / texture.height();
        let display_width = width * 0.6; 
        let display_height = display_width / original_aspect_ratio;
        let display_x = center_x - display_width / 2.0;
        
        // 使用 title_draw_y 进行绘制
        draw_texture_ex(
            texture,
            display_x,
            title_draw_y - display_height / 2.0, // 调整Y轴使图像垂直居中于 title_draw_y
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(display_width, display_height)),
                ..Default::default()
            },
        );
        // 使用 title_layout_y_base 计算布局用的底部位置
        title_layout_bottom_y = title_layout_y_base + display_height / 2.0;
    } else {
        // 后备：如果图像加载失败，绘制文本标题
        let title_size = 48.0;
        // 使用 title_draw_y 进行绘制
        draw_chinese_text("方块爆破", center_x, title_draw_y, title_size, COLOR_TITLE);
        // 使用 title_layout_y_base 计算布局用的底部位置
        title_layout_bottom_y = title_layout_y_base + title_size / 2.0; 
    }
    
    // 副标题 - 定位在标题布局的下方 (移除)
    // let subtitle_y = title_layout_bottom_y + 30.0; 
    // let subtitle_size = 24.0;
    // draw_chinese_text("Rust 版本", center_x, subtitle_y, subtitle_size, Color::new(0.8, 0.8, 0.9, 1.0));
    
    // 计算按钮尺寸和位置
    let button_width = 200.0;
    let button_height = 60.0;
    let button_x = center_x - button_width / 2.0;
    
    // 按钮间距
    let button_spacing = 20.0;
    
    // 计算按钮区域的总高度
    let buttons_total_height = button_height * 2.0 + button_spacing;
    
    // 让按钮位于屏幕下半部分的中心位置
    // 首先计算屏幕下半部分的高度和中心
    let bottom_half_start = height * 0.5;
    let bottom_half_height = height - bottom_half_start;
    let bottom_half_center = bottom_half_start + bottom_half_height * 0.2;
    
    // 将按钮组居中放置在下半部分
    let buttons_start_y = bottom_half_center - buttons_total_height * 0.5;

    
    // 开始游戏按钮 - 现在相对于下半部分计算
    let start_button_y = buttons_start_y;
    let start_button_rect = Rect::new(button_x, start_button_y, button_width, button_height);
    
    // 绘制开始游戏按钮（蓝色）
    draw_rectangle(
        start_button_rect.x, 
        start_button_rect.y, 
        start_button_rect.w, 
        start_button_rect.h, 
        Color::new(0.2, 0.6, 1.0, 1.0)
    );
    
    // 添加按钮边框，模拟圆角效果
    draw_rectangle_lines(
        start_button_rect.x, 
        start_button_rect.y, 
        start_button_rect.w, 
        start_button_rect.h, 
        2.0, // * dpi_scale, (Removed)
        Color::new(0.3, 0.7, 1.0, 1.0)
    );
    
    // 开始游戏按钮文字
    let button_text_size = 24.0; // * dpi_scale; (Removed)
    draw_chinese_text(
        "开始游戏", 
        center_x, 
        start_button_rect.y + start_button_rect.h / 2.0 + button_text_size / 4.0, 
        button_text_size, 
        WHITE
    );
    
    // 排行榜按钮
    let leaderboard_button_y = start_button_y + button_height + button_spacing;
    let leaderboard_button_rect = Rect::new(button_x, leaderboard_button_y, button_width, button_height);
    
    // 绘制排行榜按钮（深色带边框）- 始终保持可点击状态
    draw_rectangle(
        leaderboard_button_rect.x, 
        leaderboard_button_rect.y, 
        leaderboard_button_rect.w, 
        leaderboard_button_rect.h, 
        Color::new(0.2, 0.2, 0.3, 1.0)
    );
    
    // 添加蓝色边框，模拟圆角效果
    draw_rectangle_lines(
        leaderboard_button_rect.x, 
        leaderboard_button_rect.y, 
        leaderboard_button_rect.w, 
        leaderboard_button_rect.h, 
        2.0, // * dpi_scale, (Removed)
        Color::new(0.3, 0.7, 1.0, 1.0)
    );
    
    // 排行榜按钮文字
    draw_chinese_text(
        "排行榜", 
        center_x, 
        leaderboard_button_rect.y + leaderboard_button_rect.h / 2.0 + button_text_size / 4.0, 
        button_text_size, 
        WHITE
    );
    
    // 绘制T形俄罗斯方块 - 使用简单方块绘制，不使用旋转动画
    let block_size = 40.0; // * dpi_scale; (Removed)
    let t_block_center_x = center_x;
    let t_block_center_y = height * 0.85;
    
    // 绘制T形方块（紫色）
    let block_color = Color::new(0.553, 0.369, 0.816, 1.0);
    
    // 上方方块 - 使用立体方块绘制
    drawing::draw_cube_block(
        t_block_center_x - block_size / 2.0, 
        t_block_center_y - block_size * 1.5, 
        block_size, 
        block_color
    );
    
    // 左方块
    drawing::draw_cube_block(
        t_block_center_x - block_size * 1.5, 
        t_block_center_y - block_size / 2.0, 
        block_size, 
        block_color
    );
    
    // 中间方块
    drawing::draw_cube_block(
        t_block_center_x - block_size / 2.0, 
        t_block_center_y - block_size / 2.0, 
        block_size, 
        block_color
    );
    
    // 右方块
    drawing::draw_cube_block(
        t_block_center_x + block_size / 2.0, 
        t_block_center_y - block_size / 2.0, 
        block_size, 
        block_color
    );
    
    // 页脚版权信息
    let footer_y = height - 30.0; // * dpi_scale; (Removed)
    let footer_size = 14.0; // * dpi_scale; (Removed)
    draw_chinese_text(
        "© 2023 方块爆破 | Rust 版本", 
        center_x, 
        footer_y, 
        footer_size, 
        Color::new(0.6, 0.6, 0.7, 1.0)
    );
    
    // 检测按钮点击
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        
        // 开始游戏按钮点击
        if start_button_rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1)) {
            // 使用过渡动画进入游戏状态
            game.start_transition(GameState::Playing);
            game.score = 0;
            game.combo = 0;
            game.grid = grid::Grid::new();
            game.generate_blocks();
            game.score_uploaded = false;
        }
        
        // 排行榜按钮点击
        if leaderboard_button_rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1)) {
            log_info!("排行榜按钮被点击"); 
            // 使用过渡动画进入排行榜页面
            game.start_transition(GameState::Leaderboard);
        }
    }
}

// 绘制传统菜单界面的函数
fn draw_menu(game: &mut Game) {
    // 移除：不再绘制游戏界面作为背景
    // draw_game(game);
    
    // 绘制半透明覆盖层使背景变暗 (直接绘制在屏幕上)
    let width = screen_width();
    let height = screen_height();
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.7));
    
    let dpi_scale = get_dpi_scale();
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    
    // 标题
    let title_size = 40.0; // * dpi_scale; (Removed)
    draw_chinese_text("方块爆破", center_x, center_y - 100.0, title_size, Color::new(1.0, 0.4, 0.2, 1.0));
    
    // 高分显示
    let high_score_size = 24.0; // * dpi_scale; (Removed)
    let high_score_text = format!("最高分: {}", game.cloud_high_score.unwrap_or(0)); // 使用云端最高分
    draw_chinese_text(&high_score_text, center_x, center_y - 30.0, high_score_size, WHITE);
    
    // 游戏模式选择
    let mode_size = 20.0; // * dpi_scale; (Removed)
    let mode_text = match game.game_mode {
        GameMode::Easy => "简单模式",
        GameMode::Normal => "普通模式",
        GameMode::Happy => "开心模式", // 添加 Happy 模式显示
    };
    draw_chinese_text(mode_text, center_x, center_y + 20.0, mode_size, WHITE);
    
    // 开始提示
    let hint_size = 28.0; // * dpi_scale; (Removed)
    draw_chinese_text("点击开始游戏", center_x, center_y + 70.0, hint_size, WHITE);
    
    // 切换难度提示
    let space_hint_size = 16.0; // * dpi_scale; (Removed)
    draw_chinese_text("按空格键切换游戏难度", center_x, center_y + 120.0, space_hint_size, Color::new(0.8, 0.8, 0.8, 1.0));
    
    // 主菜单按钮
    let back_size = 16.0; // * dpi_scale; (Removed)
    let back_text = "返回主菜单";
    let back_width = measure_text(back_text, None, back_size as u16, 1.0).width;
    let back_x = 20.0; // * dpi_scale; (Removed)
    let back_y = 20.0; // * dpi_scale; (Removed)
    let back_padding = 10.0; // * dpi_scale; (Removed)
    let back_rect = Rect::new(
        back_x - back_padding, 
        back_y - back_size, 
        back_width + 2.0 * back_padding, 
        back_size + 2.0 * back_padding
    );
    
    // 绘制返回按钮背景和文字
    draw_rectangle(
        back_rect.x, 
        back_rect.y, 
        back_rect.w, 
        back_rect.h, 
        Color::new(0.2, 0.2, 0.3, 0.8)
    );
    draw_chinese_text(back_text, back_x + back_width/2.0, back_y - back_size/2.0, back_size, WHITE);
    
    // 检测鼠标点击
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        let mouse_vec = Vec2::new(mouse_pos.0, mouse_pos.1);
        
        // 返回主菜单按钮点击
        if back_rect.contains(mouse_vec) {
            game.start_transition(GameState::MainMenu);
            return;
        }
        
        // 点击其他区域启动游戏
        game.score = 0;
        game.combo = 0;
        game.grid = grid::Grid::new();
        game.generate_blocks(); // 会使用当前的 game_mode
        game.score_uploaded = false;
        game.state = GameState::Playing;
    }
    
    // 空格键切换游戏难度 (逻辑移到 update_game 开头)
    /* if is_key_pressed(KeyCode::Space) {
        game.easy_mode = !game.easy_mode;
        
        if game.easy_mode {
            // 简单模式参数
            game.simple_block_chance = 60;
            game.standard_block_chance = 30;
            game.blocks_per_generation = 3;
        } else {
            // 普通模式参数
            game.simple_block_chance = 55;
            game.standard_block_chance = 25;
            game.blocks_per_generation = 3;
        }
    } */
}

// 添加对JS函数的FFI绑定
#[cfg(target_arch = "wasm32")]
extern "C" {
    // 不能直接调用 window.getWasmJitStatus，需要改为使用 js_invoke_string 类型的调用
    fn js_invoke_string(js_code_ptr: *const u8, js_code_len: usize) -> i32;
    fn js_get_result_ptr() -> *const u8;
    fn js_get_result_len() -> usize;
    fn js_trigger_vibration(duration_ms: i32); // 新增 FFI 绑定
}

// 安全包装函数获取JIT状态
#[cfg(target_arch = "wasm32")]
static JIT_STATUS_CACHE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

#[cfg(target_arch = "wasm32")]
fn get_wasm_jit_status() -> String {
    // 尝试获取缓存锁
    let mut cache_guard = JIT_STATUS_CACHE.lock().unwrap_or_else(|poisoned| {
        // 处理锁中毒情况，这里简单地重置缓存并继续
        log_warn!("JIT status cache lock poisoned, resetting.");
        poisoned.into_inner()
    });

    // 检查缓存中是否已有值
    if let Some(status) = cache_guard.as_ref() {
        return status.clone(); // 如果有，直接返回克隆值
    }

    // 如果缓存为空，则执行 FFI 调用获取状态
    // FFI 调用需要 unsafe 块
    let status = unsafe {
        // 构造 JS 调用代码
        let js_code = "window.getWasmJitStatus()";
        let js_code_bytes = js_code.as_bytes();
        
        // 调用JS并获取结果
        // 假设 js_invoke_string, js_get_result_ptr, js_get_result_len 已在外部定义
        js_invoke_string(js_code_bytes.as_ptr(), js_code_bytes.len());
        
        let result_ptr = js_get_result_ptr();
        let result_len = js_get_result_len();
        
        if result_ptr.is_null() || result_len == 0 {
            "未知".to_string() // JS 调用失败或未返回有效指针/长度
        } else {
            // 从结果指针和长度创建字节切片
            let result_bytes = std::slice::from_raw_parts(result_ptr, result_len);
            // 尝试将字节切片解码为 UTF-8 字符串
            match std::str::from_utf8(result_bytes) {
                Ok(s) => s.to_string(), // 解码成功，返回字符串
                Err(_) => "解码错误".to_string() // 解码失败
            }
        }
    };
    
    // 将获取到的状态存入缓存
    *cache_guard = Some(status.clone());
    
    // 返回新获取的状态
    status
    // 当 cache_guard 离开作用域时，锁会自动释放
}

#[cfg(not(target_arch = "wasm32"))]
fn get_wasm_jit_status() -> String {
    "非WASM环境 (Native)".to_string()
}

// 安全包装函数用于触发振动
#[cfg(target_arch = "wasm32")]
fn trigger_vibration_on_place(duration_ms: i32) {
    unsafe {
        js_trigger_vibration(duration_ms);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn trigger_vibration_on_place(_duration_ms: i32) {
    // 非WASM平台无操作
}




