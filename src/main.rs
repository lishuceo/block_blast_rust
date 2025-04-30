use macroquad::prelude::*;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use cloud::PlayerRank;
use block_blast::GameMode; // 使用库名导入

// 全局颜色常量 - 基于 #3C569E 的配色方案
const COLOR_PRIMARY: Color = Color { r: 0.235, g: 0.337, b: 0.62, a: 1.0 };         // 主色 #3C569E
const COLOR_PRIMARY_DARK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.5 };   // 主色75%亮度，用于网格区域
const COLOR_PRIMARY_OVERLAY: Color = Color { r: 0.118, g: 0.169, b: 0.31, a: 0.9 }; // 主色50%亮度，用于半透明覆盖层
const COLOR_BORDER: Color = Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 };               // 边框色
const COLOR_TITLE: Color = Color { r: 1.0, g: 0.4, b: 0.2, a: 1.0 };                // 标题色
const SKYBLUE: Color = Color { r: 0.5, g: 0.7, b: 1.0, a: 1.0 };

pub mod block;
pub mod grid;
pub mod save;
pub mod effects;
pub mod cloud;
pub mod log;
pub mod drawing; // 添加 drawing 模块声明
pub mod build_info; // <-- 添加这一行

// 使用宏导入
#[macro_use]
mod log_macro_import {
    // 空模块，仅用于导入宏
    pub use crate::log::*;
}

// 如果需要，显式导入TextAlign
// use macroquad::text::TextAlign;

// 移除不必要的导入
// use wasm_bindgen::prelude::*;

// 不需要导入自己定义的模块，因为它们已经在当前文件中声明
// use crate::block;
// use crate::grid;
// use crate::save;
// use crate::effects;

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
    save_data: save::SaveData,        // 保留 SaveData 结构，但最高分不再使用它
    cloud_high_score: Option<u32>,    // 新增：从云端获取的最高分
    needs_score_upload: bool,         // 新增：标记是否需要在主循环中上传分数
    game_mode: GameMode,              // 新增：游戏模式枚举
    simple_block_chance: i32,         // 保留，但可能不再直接使用
    standard_block_chance: i32,       // 保留，但可能不再直接使用
    blocks_per_generation: usize,     // 每次生成的方块数量 (1-5)
    effects: effects::Effects,         // 特效系统
    
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
    
    // 页面过渡动画相关变量
    transition_alpha: f32,            // 过渡透明度 (0.0-2.0: 0.0-1.0为淡出阶段, 1.0-2.0为淡入阶段)
    transition_active: bool,          // 过渡动画是否激活
    transition_timer: f32,            // 过渡动画计时器
    transition_from: GameState,       // 过渡起始状态
    transition_to: GameState,         // 过渡目标状态
    transition_phase: bool,           // 过渡阶段 (false=淡出, true=淡入)
    
    // 新增：用于方块返回动画
    animating_block: Option<AnimatingBlockData>,
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
            drag_offset: Vec2::new(0.0, 0.0), // 初始化为零偏移
            drag_original_pos: None, // 初始化为 None
            save_data: save::SaveData::load(), // 仍然加载，但不再用于最高分判断
            cloud_high_score: None,    // 初始化为 None
            needs_score_upload: false, // 初始化为 false
            game_mode: GameMode::Happy, // 默认设为简单模式
            simple_block_chance: 30,  // 保留默认值，但可能不再使用
            standard_block_chance: 60, // 保留默认值，但可能不再使用
            blocks_per_generation: 3, // 默认生成3个方块
            effects: effects::Effects::new(), // 初始化特效系统
            
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
            
            // 页面过渡动画相关变量
            transition_alpha: 1.0,
            transition_active: false,
            transition_timer: 0.0,
            transition_from: GameState::MainMenu,
            transition_to: GameState::MainMenu,
            transition_phase: false,
            
            animating_block: None, // 初始化为 None
        }
    }
    
    // 生成随机方块
    fn generate_blocks(&mut self) {
        self.current_blocks.clear();
        
        // 随机生成设定数量的方块
        for _ in 0..self.blocks_per_generation {
            // let block = block::BlockShape::generate_for_mode(self.easy_mode); // 使用旧的布尔值
            let block = block::BlockShape::generate_for_mode(self.game_mode); // 使用新的枚举值
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
        let _bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
        
        // 确保底部区域最小高度
        let min_bottom_height = screen_height() * 0.2;
        let bottom_area_height = (screen_height() - separator_y).max(min_bottom_height);
        let blocks_y = separator_y + bottom_area_height / 2.0; // 垂直居中
        
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
        match cloud::get_leaderboard(10).await {
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
    // let dpi_scale = get_dpi_scale(); // 获取DPI缩放 (Removed)
    
    // 绘制半透明背景
    draw_rectangle(0.0, 0.0, width, height, COLOR_PRIMARY_OVERLAY);
    
    // 绘制排行榜标题
    let title_text = "排行榜";
    let title_font_size = 40.0; // * dpi_scale; // 应用DPI缩放 (Removed)
    draw_chinese_text(title_text, width / 2.0, height * 0.15, title_font_size, WHITE);
    
    // 检查排行榜初始化状态
    if !game.show_leaderboard_button || game.is_leaderboard_loading {
        // 显示加载状态或未初始化状态
        let status_text = if game.is_leaderboard_loading {
            "正在加载排行榜数据..."
        } else {
            "排行榜服务未连接"
        };
        draw_chinese_text(status_text, width / 2.0, height * 0.35, 24.0, // * dpi_scale, (Removed)
                          if game.is_leaderboard_loading { SKYBLUE } else { Color::new(1.0, 0.5, 0.5, 1.0) });
        
        // 绘制加载动画
        if game.is_leaderboard_loading {
            // 绘制旋转的加载圈
            let center_x = width / 2.0;
            let center_y = height * 0.5;
            let radius = 20.0; // * dpi_scale; (Removed)
            let thickness = 4.0; // * dpi_scale; (Removed)
            let rotation_speed = 2.5; // 稍微降低旋转速度，减少视觉冲击
            
            // 计算当前旋转角度，添加平滑补间
            let current_time = get_time() as f32;
            let rotation = current_time * rotation_speed % (std::f32::consts::PI * 2.0);
            
            // 计算透明度脉动 - 在淡入动画期间始终保持高透明度
            let alpha_base = if game.transition_active && game.transition_phase {
                // 淡入阶段保持稳定透明度
                0.9
            } else {
                // 正常情况下有轻微脉动
                0.7 + (current_time * 1.5).sin() * 0.2
            };
            
            // 绘制背景圆圈
            draw_circle_lines(center_x, center_y, radius, thickness, Color::new(0.3, 0.3, 0.3, alpha_base * 0.7));
            
            // 绘制旋转的弧
            let segments = 32;
            let start_angle = rotation;
            let end_angle = start_angle + std::f32::consts::PI * 1.5;
            
            // 计算弧的点并绘制线段
            for i in 0..segments {
                let t1 = start_angle + (end_angle - start_angle) * i as f32 / segments as f32;
                let t2 = start_angle + (end_angle - start_angle) * (i + 1) as f32 / segments as f32;
                
                let x1 = center_x + radius * t1.cos();
                let y1 = center_y + radius * t1.sin();
                let x2 = center_x + radius * t2.cos();
                let y2 = center_y + radius * t2.sin();
                
                // 渐变色彩 - 从蓝色渐变到白色，添加稳定系数
                let alpha = i as f32 / segments as f32;
                let color = Color::new(
                    0.5 + 0.5 * alpha, 
                    0.7 + 0.3 * alpha, 
                    1.0, 
                    (0.7 + 0.3 * alpha) * alpha_base
                );
                
                draw_line(x1, y1, x2, y2, thickness, color);
            }
            
            // 在加载动画下方添加提示
            draw_chinese_text("正在连接服务器...", center_x, center_y + radius + 30.0, // * dpi_scale, (Removed)
                            16.0, // * dpi_scale, (Removed)
                            Color::new(0.5, 0.7, 1.0, alpha_base));
        } else {
            // 如果未初始化，显示重试提示
            draw_chinese_text("点击右上角按钮返回主菜单",
                             width / 2.0, height * 0.45, 20.0, // * dpi_scale, (Removed)
                             WHITE);
        }
    } else {
        // 绘制排行榜数据
        let font_size = 24.0; // * dpi_scale; // 应用DPI缩放 (Removed)
        let line_height = font_size * 1.5;
        let start_y = height * 0.25;
        
        let leaderboard = &game.leaderboard_data;
        if leaderboard.is_empty() {
            let text = "暂无数据";
            draw_chinese_text(text, width / 2.0, start_y + line_height, font_size, WHITE);
        } else {
            // 绘制表头
            let rank_text = "排名";
            let name_text = "玩家名称";
            let score_text = "分数";
            
            let column_width = width / 3.0;
            // 使用draw_chinese_text绘制表头
            draw_chinese_text(rank_text, width * 0.15, start_y, font_size, WHITE);
            draw_chinese_text(name_text, width * 0.35, start_y, font_size, WHITE);
            draw_chinese_text(score_text, width * 0.65, start_y, font_size, WHITE);
            
            // 绘制分割线
            draw_line(width * 0.1, start_y + font_size * 0.5, width * 0.9, start_y + font_size * 0.5, 2.0, // * dpi_scale, (Removed)
                      GRAY); // 应用DPI缩放
            
            // 绘制排行榜数据
            for (i, rank) in leaderboard.iter().enumerate() {
                let row_y = start_y + line_height * (i as f32 + 1.0);
                
                // 绘制排名
                let rank_display = format!("{}", i + 1);
                // 使用draw_chinese_text绘制排名（虽然是数字，但保持一致性）
                draw_chinese_text(&rank_display, width * 0.15, row_y, font_size, WHITE);
                
                // 绘制玩家名称
                // 使用draw_chinese_text绘制玩家名称
                draw_chinese_text(&rank.name, width * 0.35, row_y, font_size, WHITE);
                
                // 绘制分数
                let score_display = format!("{}", rank.score);
                // 使用draw_chinese_text绘制分数
                draw_chinese_text(&score_display, width * 0.65, row_y, font_size, WHITE);
            }
        }
    }
    
    // 绘制返回按钮
    let button_width = 200.0; // * dpi_scale; // 应用DPI缩放 (Removed)
    let button_height = 50.0; // * dpi_scale; // 应用DPI缩放 (Removed)
    let button_x = width / 2.0 - button_width / 2.0;
    let button_y = height * 0.8;
    
    let button_color = if is_mouse_in_rect(button_x, button_y, button_width, button_height) {
        if is_mouse_button_down(MouseButton::Left) {
            DARKGRAY
        } else {
            LIGHTGRAY
        }
    } else {
        GRAY
    };
    
    draw_rectangle(button_x, button_y, button_width, button_height, button_color);
    
    let font_size = 24.0; // * dpi_scale; // 应用DPI缩放 (Removed)
    let button_text = "返回主菜单";
    // 使用draw_chinese_text绘制按钮文本
    draw_chinese_text(button_text, 
                      button_x + button_width / 2.0, 
                      button_y + button_height / 2.0, // 调整Y坐标以垂直居中
                      font_size, 
                      BLACK);
    
    // 检测按钮点击
    if is_mouse_button_released(MouseButton::Left) && is_mouse_in_rect(button_x, button_y, button_width, button_height) {
        game.start_transition(GameState::MainMenu);
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
    
    // 根据屏幕大小动态计算顶部偏移
    let grid_offset_y = screen_height() * 0.07;

    // 绘制游戏标题，字体大小根据DPI缩放
    draw_chinese_text("逆向俄罗斯方块", 
             screen_width() / 2.0,
             grid_offset_y / 2.0, 
             20.0, // * dpi_scale, // 字体大小乘以DPI缩放 (Removed)
             WHITE);
    
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
    
    // 绘制游戏网格
    game.grid.draw(grid_offset_x, grid_offset_y, cell_size);
    
    // 更新粒子效果系统
    game.effects.draw();
    
    // 显示游戏分数
    let score_y = grid_offset_y + grid_size + 23.0;
    draw_chinese_text(
        &format!("分数: {}", game.score), 
        40.0, // 向右调整，更美观
        score_y, 
        15.0, // * dpi_scale, (Removed)
        WHITE
    );
    
    // 显示最高分
    draw_chinese_text(
        // &format!("最高分: {}", game.save_data.high_score), 
        &format!("最高分: {}", game.cloud_high_score.unwrap_or(0)), // 使用云端最高分
        screen_width() - 100.0, // 向右调整，更美观
        score_y, 
        15.0, // * dpi_scale, (Removed)
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
        2.0, // * dpi_scale, // 线宽度也随DPI缩放 (Removed)
        Color::new(0.3, 0.3, 0.3, 1.0)
    );
    
    // 绘制下方区域的背景
    let bottom_area_top = separator_y + (if is_small_screen { 2.0 } else { 5.0 });
    // 确保底部区域至少有屏幕高度的一定比例
    let min_bottom_height = screen_height() * 0.2; // 至少占屏幕高度的20%
    let bottom_area_height = (screen_height() - bottom_area_top).max(min_bottom_height);
    
    // 绘制可选方块区域的标题
    draw_chinese_text(
        "可拖拽方块", 
        screen_width() / 2.0, // 居中显示
        bottom_area_top + (if is_small_screen { 15.0 } else { 25.0 }), 
        20.0, // * dpi_scale, // 字体大小乘以DPI缩放 (Removed)
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
    
    // --- 绘制底部可选方块 --- 
    for (idx, block) in game.current_blocks.iter().enumerate() {
        let block_pos_x = start_x + block_size/2.0 + idx as f32 * (block_size + block_margin);
        let block_pos_y = blocks_y;
        
        // 检查是否是正在返回动画的块，如果是则跳过
        if let Some(anim_data) = &game.animating_block {
            if anim_data.block_idx == idx {
                continue; // 跳过绘制，因为它由动画逻辑绘制
            }
        }
        
        // 检查是否是正在拖拽的块，如果是则绘制透明占位符
        let draw_color = if game.drag_block_idx == Some(idx) {
            Color::new(block.color.r/2.0, block.color.g/2.0, block.color.b/2.0, 0.3) // 手动创建透明颜色
        } else {
            block.color // 正常颜色
        };

        // 绘制方块 - 根据方块尺寸调整单元格大小
        let cell_scale = block_size / (cell_size * 5.0); // 调整单元格大小与方块大小的比例
        for &(dx, dy) in &block.cells {
            let x = block_pos_x + dx as f32 * cell_size * cell_scale;
            let y = block_pos_y + dy as f32 * cell_size * cell_scale;
            drawing::draw_cube_block(x - cell_size * cell_scale / 2.0, y - cell_size * cell_scale / 2.0, 
                          cell_size * cell_scale, draw_color);
        }
    }
    
    // --- 绘制返回动画中的方块 --- 
    if let Some(anim_data) = &game.animating_block {
        // 确保索引有效
        if anim_data.block_idx < game.current_blocks.len() {
            let block = &game.current_blocks[anim_data.block_idx];
            
            // 使用动画的 current_pos 绘制
            // 注意：动画位置是中心点，需要调整单元格绘制
            // 找到最左上角的cell
            let mut min_dx = i32::MAX;
            let mut min_dy = i32::MAX;
            for &(dx, dy) in &block.cells {
                if dx < min_dx { min_dx = dx; }
                if dy < min_dy { min_dy = dy; }
            }
            
            // 绘制时使用网格单元格大小 (cell_size)，因为它是移动到网格或从网格返回
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
        // 确保索引有效
        if block_idx < game.current_blocks.len() {
            let block = &game.current_blocks[block_idx];
            
            // 找到最左上角的cell（最小x和y坐标的cell）
            let mut min_dx = i32::MAX;
            let mut min_dy = i32::MAX;
            for &(dx, dy) in &block.cells {
                if dx < min_dx {
                    min_dx = dx;
                }
                if dy < min_dy {
                    min_dy = dy;
                }
            }
            
            // 计算方块的几何中心
            let min_dx_all = block.cells.iter().map(|(dx, _)| *dx).min().unwrap_or(0);
            let max_dx_all = block.cells.iter().map(|(dx, _)| *dx).max().unwrap_or(0);
            let min_dy_all = block.cells.iter().map(|(_, dy)| *dy).min().unwrap_or(0);
            let max_dy_all = block.cells.iter().map(|(_, dy)| *dy).max().unwrap_or(0);

            let center_x = (min_dx_all + max_dx_all) as f32 / 2.0;
            let center_y = (min_dy_all + max_dy_all) as f32 / 2.0;
            
            // 计算左上角cell在网格中的坐标
            // pos现在是左上角cell的中心点
            let grid_top_left_x = ((pos.x - grid_offset_x) / cell_size).floor();
            let grid_top_left_y = ((pos.y - grid_offset_y) / cell_size).floor();
            
            // 计算网格坐标（以左上角cell为基准）
            let grid_x = grid_top_left_x as i32 - min_dx;
            let grid_y = grid_top_left_y as i32 - min_dy;
            
            // 判断是否在有效网格范围内
            let is_valid_pos = grid_x >= -1 && grid_x < 9 && grid_y >= -1 && grid_y < 9; // 扩大检测范围
            
            // 使用容错功能检查放置 - 仅用于预览
            let (can_place, corrected_x, corrected_y) = if is_valid_pos {
                game.grid.can_place_block_with_tolerance(block, grid_x, grid_y, 1) // 1格容错距离
            } else {
                (false, grid_x, grid_y)
            };
            
            // 为所有单元格绘制预览
            for &(dx, dy) in &block.cells {
                // 使用校正后的坐标绘制预览
                let preview_x = grid_offset_x + (corrected_x + dx) as f32 * cell_size;
                let preview_y = grid_offset_y + (corrected_y + dy) as f32 * cell_size;
                
                // 仅当预览位置在有效范围内时才绘制
                if (corrected_x + dx) >= 0 && (corrected_x + dx) < 8 && (corrected_y + dy) >= 0 && (corrected_y + dy) < 8 {
                    // 根据能否放置绘制不同颜色
                    if can_place {
                        // 半透明绿色
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.2, 0.8, 0.2, 0.7));
                        
                        // // 如果是校正后的位置，添加闪烁边框提示用户
                        // if corrected_x != grid_x || corrected_y != grid_y {
                        //     let pulse = (get_time() * 5.0).sin() * 0.5 + 0.5;
                        //     drawing::draw_cube_block(preview_x, preview_y, cell_size, 
                        //                Color::new(1.0, 1.0, 1.0, 0.5 + 0.3 * pulse as f32));
                        // }
                    } else {
                        // 半透明红色
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.8, 0.2, 0.2, 0.7));
                    }
                }
            }
            
            // 在网格上拖动时绘制方块
            for &(dx, dy) in &block.cells {
                // 使用实际鼠标位置(pos)来绘制拖动中的方块
                let rel_dx = dx - min_dx; // 相对于左上角的偏移
                let rel_dy = dy - min_dy;
                
                // 计算每个方块单元的实际位置
                let x = pos.x + rel_dx as f32 * cell_size;
                let y = pos.y + rel_dy as f32 * cell_size;
                
                // 绘制立体方块
                drawing::draw_cube_block(x - cell_size/2.0, y - cell_size/2.0, cell_size, block.color);
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
            let total_btn_height = btn_height * 2.0 + btn_spacing;
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
                // let dpi_scale = get_dpi_scale(); // (Removed)
                let btn_width = 200.0; // * dpi_scale; // 应用DPI (Removed)
                let btn_height = 40.0; // * dpi_scale; // 应用DPI (Removed)
                let btn_x = screen_width() / 2.0 - btn_width / 2.0;
                let btn_y = screen_height() / 2.0 + 100.0; // 统一按钮位置
                let btn_rect = Rect::new(btn_x, btn_y, btn_width, btn_height);
                
                // 绘制按钮
                draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, DARKGRAY);
                draw_rectangle_lines(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, 2.0, // * dpi_scale, (Removed)
                                     WHITE); // 应用DPI
                
                // 按钮文字
                draw_chinese_text(
                    "查看排行榜", 
                    btn_rect.x + btn_rect.w / 2.0, 
                    btn_rect.y + btn_rect.h / 2.0, 
                    18.0, // * dpi_scale, (Removed)
                    WHITE
                );
                
                // 如果分数已上传，显示一个提示
                if game.score_uploaded {
                    draw_chinese_text(
                        "分数已上传", 
                        screen_width() / 2.0, 
                        btn_y + btn_height + 20.0, // * dpi_scale, // 在按钮下方显示 (Removed)
                        14.0, // * dpi_scale, (Removed)
                        GREEN
                    );
                }
            }
        },
        GameState::Leaderboard => {
            // 绘制排行榜界面
            draw_leaderboard(game);
        },
        _ => {}
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
            GameMode::Easy => GameMode::Normal,
            GameMode::Normal => GameMode::Happy,
            GameMode::Happy => GameMode::Easy,
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
                        let block = &game.current_blocks[block_idx];
                        
                        // 找到最左上角的cell（最小x和y坐标的cell）
                        let mut min_dx = i32::MAX;
                        let mut min_dy = i32::MAX;
                        for &(dx, dy) in &block.cells {
                            if dx < min_dx {
                                min_dx = dx;
                            }
                            if dy < min_dy {
                                min_dy = dy;
                            }
                        }
                        
                        // 核心改动：应用偏移量使方块位于手指上方
                        let adjusted_pos = Vec2::new(
                            mouse_pos.x + game.drag_offset.x,
                            mouse_pos.y + game.drag_offset.y
                        );
                        
                        // 计算方块左上角cell的中心点
                        // 先计算方块几何中心
                        let min_dx_all = block.cells.iter().map(|(dx, _)| *dx).min().unwrap_or(0);
                        let max_dx_all = block.cells.iter().map(|(dx, _)| *dx).max().unwrap_or(0);
                        let min_dy_all = block.cells.iter().map(|(_, dy)| *dy).min().unwrap_or(0);
                        let max_dy_all = block.cells.iter().map(|(_, dy)| *dy).max().unwrap_or(0);
                        let _center_x = (min_dx_all + max_dx_all) as f32 / 2.0;
                        let _center_y = (min_dy_all + max_dy_all) as f32 / 2.0;
                        
                        // 从方块中心到左上角cell的偏移
                        let offset_to_top_left_x = (min_dx as f32 - _center_x) * cell_size;
                        let offset_to_top_left_y = (min_dy as f32 - _center_y) * cell_size;
                        
                        // 计算左上角cell的中心点坐标
                        let top_left_cell_center = Vec2::new(
                            adjusted_pos.x + offset_to_top_left_x,
                            adjusted_pos.y + offset_to_top_left_y
                        );
                        
                        game.drag_pos = Some(top_left_cell_center);
                    } else {
                        // 索引无效，重置拖拽状态
                        game.drag_block_idx = None;
                        game.drag_pos = None;
                    }
                } else {
                    // 这个分支不应该发生，但以防万一
                    game.drag_pos = None;
                }
            }
            
            // 在鼠标释放时处理方块放置
            if is_mouse_button_released(MouseButton::Left) && game.drag_block_idx.is_some() {
                if let Some(block_idx) = game.drag_block_idx {
                    if let Some(pos) = game.drag_pos { // pos is the top-left cell's center
                        let block = &game.current_blocks[block_idx];
                        
                        // --- 网格计算 (与 draw_game 一致) ---
                        let grid_size = screen_width() * 0.9;
                        let cell_size = grid_size / 8.0;
                        let grid_offset_x = (screen_width() - grid_size) / 2.0;
                        let grid_offset_y = screen_height() * 0.07;
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
                        let block_size_calc = if game.blocks_per_generation <= 2 {
                            max_block_size_calc
                        } else {
                            let width_factor = if is_small_screen { 0.80 } else { 0.85 };
                            (screen_width() * width_factor) / (game.blocks_per_generation as f32 * 1.2)
                        };
                        let block_margin_calc = block_size_calc * 0.2;
                        // 注意：这里需要计算所有可能位置的总宽度，即使某些块已被移除
                        let total_possible_width = block_size_calc * game.blocks_per_generation as f32 
                                                 + block_margin_calc * (game.blocks_per_generation as f32 - 1.0);
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
                                
                                if corrected_x != grid_x || corrected_y != grid_y {
                                    log_info!("位置已自动校正: 从({},{})到({},{})", grid_x, grid_y, corrected_x, corrected_y);
                                    // TODO: 添加声音或特效提示
                                }
                                
                                // --- 消除和得分逻辑 (保持不变) ---
                                let mut filled_rows = [false; 8];
                                let mut filled_cols = [false; 8];
                                for y in 0..8 { if (0..8).all(|x| game.grid.cells[y][x].is_some()) { filled_rows[y] = true; } }
                                for x in 0..8 { if (0..8).all(|y| game.grid.cells[y][x].is_some()) { filled_cols[x] = true; } }
                                
                                let (rows_cleared, cols_cleared) = game.grid.check_and_clear();
                                let cleared = rows_cleared + cols_cleared;
                                
                                if cleared > 0 {
                                    // 特效
                                    for y in 0..8 { if filled_rows[y] { for x in 0..8 { let effect_x = grid_offset_x + x as f32 * cell_size + cell_size/2.0; let effect_y = grid_offset_y + y as f32 * cell_size + cell_size/2.0; game.effects.show_clear_effect(effect_x, effect_y, block.color); } } } // 使用金色
                                    for x in 0..8 { if filled_cols[x] { for y in 0..8 { if !filled_rows[y] { let effect_x = grid_offset_x + x as f32 * cell_size + cell_size/2.0; let effect_y = grid_offset_y + y as f32 * cell_size + cell_size/2.0; game.effects.show_clear_effect(effect_x, effect_y, block.color); } } } } // 使用金色
                                    if game.combo >= 2 { let combo_x = screen_width() / 2.0; let combo_y = grid_offset_y + grid_size / 2.0; game.effects.show_combo_effect(game.combo, combo_x, combo_y); }
                                    
                                    // 分数和连击
                                    game.combo += 1;
                                    game.score += cleared * 100 * game.combo;
                                } else {
                                    game.combo = 0;
                                }
                                // --- 消除和得分逻辑结束 ---
                                
                                // 移除已使用的方块
                                game.current_blocks.remove(block_idx);
                                
                                // 如果没有方块了，生成新的
                                if game.current_blocks.is_empty() {
                                    game.generate_blocks();
                                }
                            }
                        } // end if is_near_grid
                        
                        // 如果放置失败，启动返回动画
                        if !placed_successfully {
                            log_info!("放置无效，启动返回动画 for block {}", block_idx);
                            let start_pos = pos; // 使用松手时的计算位置作为起点
                            // 获取存储的原始位置，如果不存在则使用重新计算的位置作为后备
                            let target_pos = game.drag_original_pos.unwrap_or(original_bottom_pos);
                            let animation_data = AnimatingBlockData {
                                block_idx,
                                start_pos,
                                target_pos: target_pos, // 使用存储的原始位置
                                current_pos: start_pos, // 初始位置
                                timer: 0.0,
                                duration: 0.15, // 动画持续时间 (秒)
                            };
                            game.animating_block = Some(animation_data);
                            // 不移除 game.current_blocks[block_idx]
                        }
                    }
                    
                    // 重置拖拽状态 (无论成功与否)
                    game.drag_block_idx = None;
                    game.drag_pos = None;
                    game.drag_original_pos = None; // 清理状态
                }
            }
            
            // 检查游戏结束
            if game.check_game_over() {
                game.state = GameState::GameOver;
            }
        },
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
        let build_time_text = &format!("build: {}", build_info::BUILD_TIMESTAMP);
        let jit_status_text = &format!("JIT status: {}", get_wasm_jit_status());
        // let debug_text = "调试: 文本渲染测试"; // 不再需要固定调试文本

        // 使用 draw_text (默认左对齐) 绘制
        draw_text(build_time_text, 10.0, 30.0, 18.0, Color::new(1.0, 1.0, 0.0, 1.0)); // 黄色
        draw_text(jit_status_text, 10.0, 55.0, 18.0, Color::new(0.0, 1.0, 1.0, 1.0)); // 青色
        // draw_chinese_text(debug_text, 10.0, 80.0, 18.0, Color::new(1.0, 0.5, 0.5, 1.0)); // 移除固定调试文本的绘制
        // --- 绘制代码结束 ---
        
        // 等待下一帧
        next_frame().await;
    }
}

// 新增绘制主菜单界面的函数
fn draw_main_menu(game: &mut Game) {
    // 清屏为深蓝色背景 #3C569E - 匹配全局设计
    clear_background(COLOR_PRIMARY);
    
    let width = screen_width();
    let height = screen_height();
    // let dpi_scale = get_dpi_scale(); // (Removed)
    
    // 计算居中位置
    let center_x = width / 2.0;
    
    // 标题文本 - 应用弹跳动画
    let title_y = height * 0.2 + game.title_bounce;
    let title_size = 48.0; // * dpi_scale; (Removed)
    draw_chinese_text("方块爆破", center_x, title_y, title_size, COLOR_TITLE);
    
    // 副标题
    let subtitle_y = title_y + 60.0; // * dpi_scale; (Removed)
    let subtitle_size = 24.0; // * dpi_scale; (Removed)
    draw_chinese_text("Rust 版本", center_x, subtitle_y, subtitle_size, Color::new(0.8, 0.8, 0.9, 1.0));
    
    // 计算按钮尺寸和位置
    let button_width = 200.0; // * dpi_scale; (Removed)
    let button_height = 60.0; // * dpi_scale; (Removed)
    let button_x = center_x - button_width / 2.0;
    
    // 开始游戏按钮
    let start_button_y = height * 0.45;
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
    let leaderboard_button_y = start_button_y + button_height + 20.0; // * dpi_scale; (Removed)
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
    // let high_score_text = format!("最高分: {}", game.save_data.high_score);
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
}

// 安全包装函数获取JIT状态
#[cfg(target_arch = "wasm32")]
// use std::sync::Mutex; <-- 移除这行重复导入
// use once_cell::sync::Lazy; // 需要确保导入 once_cell::sync::Lazy 和 std::sync::Mutex

// 定义静态变量来缓存 JIT 状态
// 注意: 理想情况下，这个静态定义应该在函数外部，位于模块级别。
static JIT_STATUS_CACHE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

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


