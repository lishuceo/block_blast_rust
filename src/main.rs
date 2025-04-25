use macroquad::prelude::*;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use cloud::PlayerRank;

pub mod block;
pub mod grid;
pub mod save;
pub mod effects;
pub mod cloud;

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
        println!("{}", message);
        // 这里可以添加显示错误信息的UI代码
    }));
}

// 更高效的立体感方块绘制函数
pub fn draw_cube_block(x: f32, y: f32, size: f32, color: Color) {
    // 获取DPI缩放因子
    let dpi_scale = get_dpi_scale();
    
    // 亮色和暗色偏移量
    let light_factor = 0.4;
    let dark_factor = 0.4;
    
    // 边缘厚度 - 在高DPI屏幕上相应调整
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
    
    // 在高DPI设备上使用抗锯齿绘制，提高视觉质量
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
            save_data: save::SaveData::load(),
            easy_mode: true,          // 默认开启简单模式
            simple_block_chance: 30,  // 简单方块30%概率
            standard_block_chance: 60, // 标准方块60%概率
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
                println!("云服务初始化成功");
            },
            Err(e) => {
                println!("云服务初始化失败: {}", e);
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
                println!("从 cloud::get_leaderboard_data 获取到的排行榜数据: {:?}", leaderboard);
                println!("从 cloud::get_leaderboard_data 获取到的玩家排名: {:?}", player_rank);
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
                println!("上传分数失败: {}", e);
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
}

// 绘制排行榜界面
fn draw_leaderboard(game: &mut Game) {
    let width = screen_width();
    let height = screen_height();
    let dpi_scale = get_dpi_scale(); // 获取DPI缩放
    
    // 绘制半透明背景
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.7));
    
    // 绘制排行榜标题
    let title_text = "排行榜";
    let title_font_size = 40.0 * dpi_scale; // 应用DPI缩放
    draw_chinese_text(title_text, width / 2.0, height * 0.15, title_font_size, WHITE);
    
    // 绘制排行榜数据
    let font_size = 24.0 * dpi_scale; // 应用DPI缩放
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
        draw_line(width * 0.1, start_y + font_size * 0.5, width * 0.9, start_y + font_size * 0.5, 2.0 * dpi_scale, GRAY); // 应用DPI缩放
        
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
    
    // 绘制返回按钮
    let button_width = 200.0 * dpi_scale; // 应用DPI缩放
    let button_height = 50.0 * dpi_scale; // 应用DPI缩放
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
    
    let button_text = "返回主菜单";
    // 使用draw_chinese_text绘制按钮文本
    draw_chinese_text(button_text, 
                      button_x + button_width / 2.0, 
                      button_y + button_height / 2.0, // 调整Y坐标以垂直居中
                      font_size, 
                      BLACK);
    
    // 检测按钮点击
    if is_mouse_button_released(MouseButton::Left) && is_mouse_in_rect(button_x, button_y, button_width, button_height) {
        game.state = GameState::MainMenu; // 修改为返回MainMenu状态
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
    let dpi_scale = get_dpi_scale();
    
    // 修改窗口背景为深灰色
    clear_background(Color::new(0.2, 0.2, 0.22, 1.0));
    
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
             20.0 * dpi_scale, // 字体大小乘以DPI缩放
             WHITE);
    
    // 绘制游戏网格背景
    draw_rectangle(
        grid_offset_x - 5.0,
        grid_offset_y - 5.0,
        grid_size + 10.0,
        grid_size + 10.0,
        Color::new(0.1, 0.1, 0.12, 1.0)
    );
    
    // 添加细边框 - 在高DPI设备上更清晰
    let border_width = 2.0 * dpi_scale;
    draw_rectangle_lines(
        grid_offset_x - 5.0,
        grid_offset_y - 5.0,
        grid_size + 10.0,
        grid_size + 10.0,
        border_width,
        Color::new(0.3, 0.3, 0.3, 1.0)
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
        15.0 * dpi_scale, 
        WHITE
    );
    
    // 显示最高分
    draw_chinese_text(
        &format!("最高分: {}", game.save_data.high_score), 
        screen_width() - 100.0, // 向右调整，更美观
        score_y, 
        15.0 * dpi_scale, 
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
        2.0 * dpi_scale, // 线宽度也随DPI缩放
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
        screen_width() / 2.0, // 居中显示
        bottom_area_top + (if is_small_screen { 15.0 } else { 25.0 }), 
        20.0 * dpi_scale, // 字体大小乘以DPI缩放
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
                                       Color::new(0.2, 0.8, 0.2, 0.4));
                        
                        // 如果是校正后的位置，添加闪烁边框提示用户
                        if corrected_x != grid_x || corrected_y != grid_y {
                            let pulse = (get_time() * 5.0).sin() * 0.5 + 0.5;
                            draw_rectangle_lines(
                                preview_x, preview_y, cell_size, cell_size,
                                2.0 * dpi_scale, // 线宽考虑DPI缩放
                                Color::new(1.0, 1.0, 1.0, 0.5 + 0.3 * pulse as f32)
                            );
                        }
                    } else {
                        // 半透明红色
                        draw_rectangle(preview_x, preview_y, cell_size, cell_size, 
                                       Color::new(0.8, 0.2, 0.2, 0.4));
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
                draw_cube_block(x - cell_size/2.0, y - cell_size/2.0, cell_size, block.color);
            }
        }
    }
    
    // 绘制菜单/游戏结束界面
    match game.state {
        GameState::MainMenu => {
            // 主菜单界面在draw_main_menu函数中单独处理
        },
        GameState::Menu => {
            let dpi_scale = get_dpi_scale();
            let width = screen_width();
            let height = screen_height();

            // 绘制半透明背景
            draw_rectangle(0.0, 0.0, width, height, Color::new(0.1, 0.1, 0.12, 0.95)); // 使用稍暗的背景
            
            // 绘制大标题
            let title_y = height * 0.25;
            draw_chinese_text("逆向俄罗斯方块", 
                     width / 2.0, 
                     title_y, 
                     40.0 * dpi_scale, 
                     WHITE);

            // 按钮尺寸和间距
            let btn_width = 220.0 * dpi_scale;
            let btn_height = 55.0 * dpi_scale;
            let btn_spacing = 20.0 * dpi_scale;
            let total_btn_height = btn_height * 2.0 + btn_spacing;
            let btn_start_y = title_y + 100.0 * dpi_scale; // 在标题下方留出更多空间

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
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0 * dpi_scale, LIGHTGRAY);
                
                draw_chinese_text(text, 
                                  rect.x + rect.w / 2.0, 
                                  rect.y + rect.h / 2.0, // 调整Y使文本垂直居中
                                  font_size, 
                                  WHITE);
            };
            // --- 绘制按钮公共函数结束 ---

            // 绘制开始游戏按钮
            draw_menu_button(start_btn_rect, "开始游戏", 25.0 * dpi_scale);

            // 绘制排行榜按钮
            draw_menu_button(leaderboard_btn_rect, "排行榜", 25.0 * dpi_scale);
            
            // 绘制最高分
            let high_score_y = leaderboard_btn_y + btn_height + 60.0 * dpi_scale;
            draw_chinese_text(&format!("最高分: {}", game.save_data.high_score), 
                     width / 2.0, 
                     high_score_y, 
                     22.0 * dpi_scale, 
                     Color::new(0.6, 0.8, 1.0, 1.0)); // 稍亮的蓝色
        },
        GameState::GameOver => {
            // 绘制半透明背景
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.99));
            
            // 绘制游戏结束文本
            draw_chinese_text("游戏结束", 
                     screen_width() / 2.0, 
                     screen_height() / 3.0, 
                     30.0 * dpi_scale, 
                     WHITE);
            
            // 绘制最终得分
            draw_chinese_text(&format!("最终得分: {}", game.score), 
                     screen_width() / 2.0, 
                     screen_height() / 2.0, 
                     25.0 * dpi_scale, 
                     Color::new(1.0, 0.8, 0.2, 1.0));
            
            // 绘制最高分
            let new_record = game.score > game.save_data.high_score;
            let high_score_text = if new_record {
                format!("新纪录! {}", game.score)
            } else {
                format!("最高分: {}", game.save_data.high_score)
            };
            
            draw_chinese_text(&high_score_text, 
                     screen_width() / 2.0, 
                     screen_height() / 2.0 + 40.0, 
                     22.0 * dpi_scale, 
                     if new_record { Color::new(1.0, 0.5, 0.0, 1.0) } else { Color::new(0.2, 0.8, 1.0, 1.0) });
            
            // 绘制重新开始提示
            draw_chinese_text("点击重新开始", 
                     screen_width() / 2.0, 
                     screen_height() / 2.0 + 100.0, 
                     25.0 * dpi_scale, 
                     WHITE);

            // GameOver状态下显示排行榜按钮
            if game.show_leaderboard_button {
                let dpi_scale = get_dpi_scale();
                let btn_width = 200.0 * dpi_scale; // 应用DPI
                let btn_height = 40.0 * dpi_scale; // 应用DPI
                let btn_x = screen_width() / 2.0 - btn_width / 2.0;
                let btn_y = screen_height() / 2.0 + 100.0; // 统一按钮位置
                let btn_rect = Rect::new(btn_x, btn_y, btn_width, btn_height);
                
                // 绘制按钮
                draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, DARKGRAY);
                draw_rectangle_lines(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, 2.0 * dpi_scale, WHITE); // 应用DPI
                
                // 按钮文字
                draw_chinese_text(
                    "查看排行榜", 
                    btn_rect.x + btn_rect.w / 2.0, 
                    btn_rect.y + btn_rect.h / 2.0, 
                    18.0 * dpi_scale, 
                    WHITE
                );
                
                // 如果分数已上传，显示一个提示
                if game.score_uploaded {
                    draw_chinese_text(
                        "分数已上传", 
                        screen_width() / 2.0, 
                        btn_y + btn_height + 20.0 * dpi_scale, // 在按钮下方显示
                        14.0 * dpi_scale, 
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
        GameState::MainMenu => {
            // 主菜单状态下不需要游戏更新逻辑
            return;
        },
        GameState::Menu => {
            if is_mouse_button_pressed(MouseButton::Left) {
                let dpi_scale = get_dpi_scale();
                let width = screen_width();
                let height = screen_height();
                let mouse_pos: Vec2 = mouse_position().into();

                // 按钮尺寸和位置计算（与draw_game中保持一致）
                let title_y = height * 0.25;
                let btn_width = 220.0 * dpi_scale;
                let btn_height = 55.0 * dpi_scale;
                let btn_spacing = 20.0 * dpi_scale;
                let btn_start_y = title_y + 100.0 * dpi_scale;

                let start_btn_x = width / 2.0 - btn_width / 2.0;
                let start_btn_y = btn_start_y;
                let start_btn_rect = Rect::new(start_btn_x, start_btn_y, btn_width, btn_height);

                let leaderboard_btn_x = start_btn_x;
                let leaderboard_btn_y = start_btn_y + btn_height + btn_spacing;
                let leaderboard_btn_rect = Rect::new(leaderboard_btn_x, leaderboard_btn_y, btn_width, btn_height);

                // 检查是否点击了开始游戏按钮
                if start_btn_rect.contains(mouse_pos) {
                    game.state = GameState::Playing;
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
                    game.state = GameState::Leaderboard;
                    // 注意：排行榜数据的加载将在run_game循环中处理
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
                    if let Some(pos) = game.drag_pos {
                        let block = &game.current_blocks[block_idx];
                        
                        // 计算网格大小和位置
                        let grid_size = screen_width() * 0.9;
                        let cell_size = grid_size / 8.0;
                        let grid_offset_x = (screen_width() - grid_size) / 2.0;
                        let grid_offset_y = screen_height() * 0.07;
                        
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
                        
                        // 计算左上角cell在网格中的坐标
                        // pos现在是左上角cell的中心点
                        let grid_top_left_x = ((pos.x - grid_offset_x) / cell_size).floor();
                        let grid_top_left_y = ((pos.y - grid_offset_y) / cell_size).floor();
                        
                        // 计算网格坐标（以左上角cell为基准）
                        let grid_x = grid_top_left_x as i32 - min_dx;
                        let grid_y = grid_top_left_y as i32 - min_dy;
                        
                        // 检查并处理方块放置 - 使用容错版本
                        // 先判断是否在扩展的有效范围内
                        let is_near_valid = grid_x >= -1 && grid_x < 9 && grid_y >= -1 && grid_y < 9;
                        
                        if is_near_valid {
                            // 使用容错功能找到合适的放置位置
                            let (can_place, corrected_x, corrected_y) = 
                                game.grid.can_place_block_with_tolerance(block, grid_x, grid_y, 1);
                            
                            if can_place {
                                // 执行放置 - 使用校正后的位置
                                game.grid.place_block(block, corrected_x, corrected_y);
                                
                                // 如果位置被校正了，播放提示音效或视觉效果
                                if corrected_x != grid_x || corrected_y != grid_y {
                                    println!("位置已自动校正: 从({},{})到({},{})", 
                                             grid_x, grid_y, corrected_x, corrected_y);
                                    // TODO: 添加声音或特效提示
                                }
                                
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
                let dpi_scale = get_dpi_scale();
                let width = screen_width();
                let height = screen_height();
                let mouse_pos: Vec2 = mouse_position().into();

                // 重新开始按钮的隐形区域（整个屏幕，除了排行榜按钮）
                let leaderboard_btn_width = 200.0 * dpi_scale;
                let leaderboard_btn_height = 40.0 * dpi_scale;
                let leaderboard_btn_x = width / 2.0 - leaderboard_btn_width / 2.0;
                // 注意：这里的 Y 坐标要和 draw_game 中绘制排行榜按钮的 Y 坐标匹配
                // 在 draw_game 的 GameOver 分支中，是 height / 2.0 + 80.0 (原始值) 
                // 但我们需要使用经过DPI缩放的值进行比较，或者重新计算draw_game中的值
                // 暂时假设draw_game中的按钮绘制位置是固定的像素值，不依赖DPI（需要检查draw_game）
                // 假设draw_game中GameOver的按钮位置是相对于中心点固定的
                let leaderboard_btn_y = height / 2.0 + 100.0; // 调整了draw_game GameOver按钮位置，这里保持一致
                let leaderboard_btn_rect = Rect::new(leaderboard_btn_x, leaderboard_btn_y, leaderboard_btn_width, leaderboard_btn_height);
                
                // 检查是否点击了排行榜按钮
                if game.show_leaderboard_button && leaderboard_btn_rect.contains(mouse_pos) {
                    game.state = GameState::Leaderboard;
                    // 加载操作由run_game处理
                } else {
                    // 点击其他区域则重新开始
                    game.state = GameState::Menu; // 返回菜单而不是直接开始游戏
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
        window_width: 400,
        window_height: 600,
        high_dpi: true,  // 保留高DPI支持
        fullscreen: false,
        sample_count: 1,  // 移除抗锯齿，使用默认值1
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // 显示设备信息和DPI缩放
    let _dpi_scale = get_dpi_scale();
    println!("设备信息: 屏幕大小 {}x{}, DPI缩放: {}", screen_width(), screen_height(), _dpi_scale);
    
    // iOS设备相关日志
    #[cfg(target_os = "ios")]
    println!("在iOS设备上运行，使用3.0倍DPI缩放");
    
    // 使用嵌入的字体数据加载字体，而不是从文件系统加载
    match load_ttf_font_from_bytes(CHINESE_FONT_DATA) {
        Ok(font) => {
            *CHINESE_FONT.lock().unwrap() = Some(font);
            println!("成功加载中文字体");
        },
        Err(err) => {
            println!("无法加载中文字体: {:?}", err);
            // 在WASM环境中不触发panic
            #[cfg(target_arch = "wasm32")]
            println!("在WASM环境中继续运行，将使用默认字体");
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

    // macroquad没有直接的set_target_fps函数
    // 可以改用适当的帧速率控制
    
    // 主游戏循环
    loop {
        // 状态转换逻辑: 在进入新状态时执行一次性操作
        if game.state != previous_state {
            match game.state {
                GameState::MainMenu => {
                    println!("进入主菜单状态");
                    // 重置一些状态或设置主菜单初始值
                },
                GameState::Playing => {
                    // 从菜单或游戏结束进入游戏状态时，重置一些数据
                    // （大部分重置已在update_game中处理）
                    println!("进入游戏状态");
                },
                GameState::Leaderboard => {
                    // 进入排行榜状态时，加载数据
                    println!("进入排行榜状态，加载数据...");
                    game.load_leaderboard().await;
                },
                GameState::GameOver => {
                    // 进入游戏结束状态时，上传分数
                    println!("进入游戏结束状态");
                    if game.score > game.save_data.high_score {
                        game.save_data.high_score = game.score;
                        game.save_data.save();
                    }
                    if game.show_leaderboard_button && !game.score_uploaded {
                        game.upload_score().await;
                    }
                },
                GameState::Menu => {
                    println!("返回菜单");
                    // 如果需要，可以在这里重置特定菜单状态
                }
            }
            previous_state = game.state; // 更新上一个状态
        }

        // 清理屏幕
        clear_background(BLACK);
        
        // 更新和绘制当前状态
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
        
        // 等待下一帧
        next_frame().await;
    }
}

// 新增绘制主菜单界面的函数
fn draw_main_menu(game: &mut Game) {
    // 清屏为深色背景 - 匹配Figma设计
    clear_background(Color::new(0.11, 0.11, 0.16, 1.0));
    
    let width = screen_width();
    let height = screen_height();
    let dpi_scale = get_dpi_scale();
    
    // 计算居中位置
    let center_x = width / 2.0;
    
    // 标题文本 - 应用弹跳动画
    let title_y = height * 0.2 + game.title_bounce;
    let title_size = 48.0 * dpi_scale;
    draw_chinese_text("方块爆破", center_x, title_y, title_size, Color::new(1.0, 0.4, 0.2, 1.0));
    
    // 副标题
    let subtitle_y = title_y + 60.0 * dpi_scale;
    let subtitle_size = 24.0 * dpi_scale;
    draw_chinese_text("Rust 版本", center_x, subtitle_y, subtitle_size, Color::new(0.8, 0.8, 0.9, 1.0));
    
    // 计算按钮尺寸和位置
    let button_width = 200.0 * dpi_scale;
    let button_height = 60.0 * dpi_scale;
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
        2.0 * dpi_scale,
        Color::new(0.3, 0.7, 1.0, 1.0)
    );
    
    // 开始游戏按钮文字
    let button_text_size = 24.0 * dpi_scale;
    draw_chinese_text(
        "开始游戏", 
        center_x, 
        start_button_rect.y + start_button_rect.h / 2.0 + button_text_size / 4.0, 
        button_text_size, 
        WHITE
    );
    
    // 排行榜按钮
    let leaderboard_button_y = start_button_y + button_height + 20.0 * dpi_scale;
    let leaderboard_button_rect = Rect::new(button_x, leaderboard_button_y, button_width, button_height);
    
    // 绘制排行榜按钮（深色带边框）
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
        2.0 * dpi_scale,
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
    let block_size = 50.0 * dpi_scale;
    let t_block_center_x = center_x;
    let t_block_center_y = height * 0.75;
    
    // 绘制T形方块（紫色）
    let block_color = Color::new(0.8, 0.3, 0.9, 1.0);
    
    // 上方方块 - 使用立体方块绘制
    draw_cube_block(
        t_block_center_x - block_size / 2.0, 
        t_block_center_y - block_size * 1.5, 
        block_size, 
        block_color
    );
    
    // 左方块
    draw_cube_block(
        t_block_center_x - block_size * 1.5, 
        t_block_center_y - block_size / 2.0, 
        block_size, 
        block_color
    );
    
    // 中间方块
    draw_cube_block(
        t_block_center_x - block_size / 2.0, 
        t_block_center_y - block_size / 2.0, 
        block_size, 
        block_color
    );
    
    // 右方块
    draw_cube_block(
        t_block_center_x + block_size / 2.0, 
        t_block_center_y - block_size / 2.0, 
        block_size, 
        block_color
    );
    
    // 页脚版权信息
    let footer_y = height - 30.0 * dpi_scale;
    let footer_size = 14.0 * dpi_scale;
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
            // 直接进入游戏状态而不是菜单状态
            game.state = GameState::Playing;
            game.score = 0;
            game.combo = 0;
            game.grid = grid::Grid::new();
            game.generate_blocks();
            game.score_uploaded = false;
            // 可以添加一些按钮点击效果
        }
        
        // 排行榜按钮点击
        if leaderboard_button_rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1)) {
            if game.show_leaderboard_button {
                game.state = GameState::Leaderboard;
                // 可以在这里加载排行榜数据
            }
        }
    }
}

// 绘制传统菜单界面的函数
fn draw_menu(game: &mut Game) {
    // 绘制游戏界面作为背景
    draw_game(game);
    
    // 绘制半透明覆盖层使背景变暗
    let width = screen_width();
    let height = screen_height();
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.7));
    
    let dpi_scale = get_dpi_scale();
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    
    // 标题
    let title_size = 40.0 * dpi_scale;
    draw_chinese_text("方块爆破", center_x, center_y - 100.0 * dpi_scale, title_size, Color::new(1.0, 0.4, 0.2, 1.0));
    
    // 高分显示
    let high_score_size = 24.0 * dpi_scale;
    let high_score_text = format!("最高分: {}", game.save_data.high_score);
    draw_chinese_text(&high_score_text, center_x, center_y - 30.0 * dpi_scale, high_score_size, WHITE);
    
    // 游戏模式选择
    let mode_size = 20.0 * dpi_scale;
    let mode_text = if game.easy_mode {
        "简单模式"
    } else {
        "普通模式"
    };
    draw_chinese_text(mode_text, center_x, center_y + 20.0 * dpi_scale, mode_size, WHITE);
    
    // 开始提示
    let hint_size = 28.0 * dpi_scale;
    draw_chinese_text("点击开始游戏", center_x, center_y + 70.0 * dpi_scale, hint_size, WHITE);
    
    // 切换难度提示
    let space_hint_size = 16.0 * dpi_scale;
    draw_chinese_text("按空格键切换游戏难度", center_x, center_y + 120.0 * dpi_scale, space_hint_size, Color::new(0.8, 0.8, 0.8, 1.0));
    
    // 主菜单按钮
    let back_size = 16.0 * dpi_scale;
    let back_text = "返回主菜单";
    let back_width = measure_text(back_text, None, back_size as u16, 1.0).width;
    let back_x = 20.0 * dpi_scale;
    let back_y = 20.0 * dpi_scale;
    let back_padding = 10.0 * dpi_scale;
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
    draw_text(back_text, back_x, back_y, back_size, WHITE);
    
    // 检测鼠标点击
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        let mouse_vec = Vec2::new(mouse_pos.0, mouse_pos.1);
        
        // 返回主菜单按钮点击
        if back_rect.contains(mouse_vec) {
            game.state = GameState::MainMenu;
            return;
        }
        
        // 点击其他区域启动游戏
        game.score = 0;
        game.combo = 0;
        game.grid = grid::Grid::new();
        game.generate_blocks();
        game.score_uploaded = false;
        game.state = GameState::Playing;
    }
    
    // 空格键切换游戏难度
    if is_key_pressed(KeyCode::Space) {
        game.easy_mode = !game.easy_mode;
        
        if game.easy_mode {
            // 简单模式参数
            game.simple_block_chance = 30;
            game.standard_block_chance = 60;
            game.blocks_per_generation = 3;
        } else {
            // 普通模式参数
            game.simple_block_chance = 20;
            game.standard_block_chance = 50;
            game.blocks_per_generation = 2;
        }
    }
}

