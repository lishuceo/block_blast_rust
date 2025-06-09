# Block Blast Rust 项目技术文档

## 项目概述

Block Blast Rust 是一个基于 Rust 和 macroquad 引擎开发的方块消除游戏。游戏的主要目标是将不同形状的方块放置到 8x8 的网格中，当填满整行或整列时消除方块获得分数。游戏设计为跨平台的，支持桌面和 WebAssembly (WASM) 环境。

**坐标系说明**: 项目中使用二维坐标系，其中 X 轴从左到右递增，Y 轴从上到下递增。网格左上角为 (0, 0)。

## 技术堆栈

- **编程语言**: Rust
- **游戏引擎**: macroquad 0.4
- **底层图形库**: miniquad 0.4
- **全局单例模式**: once_cell 1.8
- **云服务集成**: SCE Game SDK (星火对战平台)
- **构建目标**: 
  - 原生应用 (Windows, Linux, macOS)
  - WebAssembly (WASM32)

## 项目结构

```
block_blast_rust/
├── src/                    # 源代码目录
│   ├── main.rs             # 游戏主入口和核心逻辑
│   ├── lib.rs              # 库入口，导出所有模块
│   ├── block.rs            # 方块形状定义和生成逻辑
│   ├── grid.rs             # 游戏网格和放置逻辑
│   ├── wave.rs             # 新增：动态波次和难度管理器
│   ├── save.rs             # 游戏存档和加载
│   ├── effects.rs          # 粒子效果和视觉特效
│   ├── cloud.rs            # 云服务和排行榜实现
│   ├── random.rs           # 随机数生成工具
│   ├── log.rs              # 跨平台日志系统
│   ├── build_info.rs       # 构建时间信息
│   └── minimal.rs          # 最小化示例代码
├── resources/              # 资源文件目录
│   └── fonts/              # 字体文件
├── web/                    # WebAssembly 构建目录
│   └── node_modules/       # 前端依赖库
│       └── sce-game_sdk/   # SCE游戏SDK
├── Cargo.toml              # 项目依赖配置
├── Cargo.lock              # 依赖锁定文件
├── build.rs                # 构建脚本，生成构建时间信息
├── build_wasm_clean.bat    # WASM 构建脚本 (Windows)
├── serve.py                # 简易 HTTP 服务器
├── js_bridge.js            # JavaScript 桥接代码
└── index_template.html     # WASM 页面模板
```

## 核心模块说明

### main.rs

游戏的主要入口点，包含以下主要功能：

1. **游戏状态管理**:
   - 定义了 `GameState` 枚举 (MainMenu, Menu, Playing, GameOver, Leaderboard)
   - 管理游戏生命周期和状态转换

2. **游戏数据结构**:
   - `Game` 结构体保存游戏核心数据
   - 管理游戏网格、当前可用方块、分数等
   - 包含UI动画相关变量（旋转、弹跳等）
   - **新增**: 包含 `WaveManager` 实例，用于动态关卡管理。

3. **界面渲染函数**:
   - `draw_main_menu` 函数负责渲染主菜单界面
   - `draw_menu` 函数负责渲染传统菜单界面
   - `draw_game` 函数负责渲染游戏界面 (会在右上角显示当前波次阶段)
   - `draw_game_over` 函数负责渲染游戏结束界面
   - `draw_leaderboard` 函数负责渲染排行榜界面

4. **游戏逻辑更新**:
   - `update_game` 函数处理游戏逻辑更新
   - `update_main_menu_animations` 处理主菜单动画更新
   - 处理用户输入、方块移动、消除检测等
   - **新增**: 在成功放置方块后，调用 `wave_manager.increment_turn()` 推进回合，并处理来自 `WaveManager` 的奖励分数。
   - **新增**: 行/列消除后，调用 `wave_manager.notify_line_cleared()`。

5. **方块生成 (`Game::generate_blocks`)**:
   - **动态生成**: 现在从 `wave_manager` 获取方块生成的数量 (固定为3) 和当前的方块复杂度因子 (`block_complexity_factor`)。
   - **情景感知生成 (改进版)**:
     - 在生成方块前，调用 `grid.get_difficulty_score()` 获取基于连通区域分析的困难度分数
     - 调用 `wave_manager.should_offer_helpful_block_v2(difficulty_score)` 判断是否应该提供"帮助性"方块
     - 这种方法比原有的填充率判断更准确，能识别出空白区域被分割成孤立小块的困难情况
     - 如果需要帮助，则调用 `grid.find_placeable_shapes_for_empty_spots()` 尝试找到可以放置的小方块（支持1-5格的多种形状）
     - 如果找到候选的"有用方块"，会随机选择一个加入到待选列表中
     - 其余方块通过 `block::BlockShape::generate_with_complexity()` 结合当前的复杂度因子生成

6. **中文文本支持**:
   - 实现了 `draw_chinese_text` 函数支持中文显示
   - 使用嵌入式字体资源

7. **3D 效果渲染**:
   - `draw_cube_block` 函数实现立体感方块绘制

### drawing.rs

负责提供自定义的绘图函数，增强或优化 macroquad 的基本绘图能力:

1.  **立体感方块**:
    *   `draw_cube_block` 函数用于绘制具有简单 3D 效果的方块单元，增强视觉表现力。

2.  **圆角矩形**:
    *   新增了一套绘制圆角矩形的函数，用于 UI 元素（如按钮、面板）的绘制。
    *   实现方式：通过组合 macroquad 的基本形状（矩形、圆形扇区、线条）来模拟圆角效果。
    *   **优点**: 实现简单直观。
    *   **潜在缺点**: 对于大量绘制，性能可能不如自定义着色器或网格方案。
    *   **可用函数**:
        *   `draw_rounded_rectangle(x, y, width, height, radius, color)`: 绘制实心圆角矩形。
        *   `draw_rounded_rectangle_lines(x, y, width, height, radius, thickness, color)`: 绘制圆角矩形轮廓。
        *   `draw_rounded_rectangle_with_shadow(x, y, width, height, radius, color, shadow_offset, shadow_color)`: 绘制带阴影的圆角矩形。
        *   `draw_rounded_rectangle_with_border(x, y, width, height, radius, thickness, fill_color, border_color)`: 绘制带边框的圆角矩形。
        *   `draw_rounded_rectangle_3d(x, y, width, height, radius, color, depth)`: 绘制带有简单3D效果（高光/阴影）的圆角矩形。

### 新增UI系统

游戏新增了现代化UI界面系统，特别是全新的主菜单:

1. **主菜单界面**:
   - 基于Figma设计实现的美观UI
   - 包含游戏标题、副标题、功能按钮和装饰元素
   - 使用动画效果增强视觉体验

2. **动画效果**:
   - 标题文字弹跳动画
   - T形方块旋转动画
   - 缓动和过渡效果

3. **多层级菜单**:
   - 主菜单(MainMenu): 提供游戏主入口和排行榜入口
   - 传统菜单(Menu): 提供游戏设置和模式选择
   - 游戏界面(Playing): 显示实际游戏内容
   - 结束界面(GameOver): 显示得分和重新开始选项
   - 排行榜(Leaderboard): 显示全球排名

4. **现代UI元素**:
   - 圆角按钮
   - 配色方案一致的界面设计
   - 具有立体感的视觉元素
   - 响应式布局，适应不同设备

### block.rs

负责方块的定义和生成:

1. **方块结构**:
   ```rust
   pub struct BlockShape {
       pub cells: Vec<(i32, i32)>,
       pub color: Color,
   }
   ```

2. **方块类型与常量**:
   - 简单形状 (1-2 个方块)，标准形状 (类俄罗斯方块形状)，复杂形状。
   - **新增**: `pub const SHAPE_DOT: &[(i32, i32)]` 和 `pub const SHAPE_H2: &[(i32, i32)]` 被设为公共常量，方便其他模块引用。

3. **方块生成**:
   - `generate_with_complexity(complexity: f32)`: 根据复杂度因子混合不同的方块池 (EASY, NORMAL, HAPPY) 来决定生成的方块形状。复杂度因子越高，选中更复杂形状的概率越大。
   - **新增**: `pub fn new_dot() -> Self`: 直接创建一个1x1的点状方块。
   - **新增**: `pub fn get_random_block_color() -> Color`: 提供一个公共方法用于获取随机的方块颜色。

4. **辅助函数**:
   - **新增**: `pub fn rotate_90_clockwise(cells: &[(i32, i32)]) -> Vec<(i32, i32)>`: 将方块顺时针旋转90度，现在是公共函数。
   - **新增**: `pub fn normalize_cells(cells: Vec<(i32, i32)>) -> Vec<(i32, i32)>`: 标准化方块坐标，使其左上角尽量靠近(0,0)，现在是公共函数。

### grid.rs

管理游戏网格和方块放置:

1. **网格结构**:
   ```rust
   pub struct Grid {
       pub cells: [[Option<Color>; 8]; 8],
   }
   ```

2. **网格操作**:
   - `can_place_block` 检查方块是否可放置。
   - `place_block` 在网格上放置方块。
   - `check_and_clear` 检查并消除已填满的行和列，**返回被清除的行和列的索引 `(Vec<usize>, Vec<usize>)`**。
   - `draw` 渲染网格和已放置的方块。(之前是 `draw_with_highlights`，由于移除了目标行/列，不再需要高亮)

3. **辅助功能**:
   - `can_place_block_with_tolerance` 提供放置容错，改善用户体验。
   - **新增**: `pub fn get_almost_complete_lines(&self, cells_needed_to_complete: usize) -> (Vec<usize>, Vec<usize>)`:
     - 分析棋盘，返回一个元组，分别包含所有只差 `cells_needed_to_complete` 个格子即满的行和列的索引。
   - **新增**: `pub fn get_filled_ratio(&self) -> f32`:
     - 计算当前网格中已填充格子占总格子数的比例 (0.0 到 1.0)。
   - **新增**: `pub fn find_placeable_shapes_for_empty_spots(&self, max_cells_to_fill: usize, target_shapes: &[&'static [(i32, i32)]]) -> Vec<BlockShape>`:
     - 尝试将 `target_shapes` (例如1格或2格的小方块) 的所有旋转形式放置到棋盘上的所有空位。
     - 返回一个 `Vec<BlockShape>` 列表，包含所有成功找到的、格子数不超过 `max_cells_to_fill` 且可以放置的方块实例（已赋予随机颜色）。用于"情景感知方块生成"。

4. **连通区域分析**（新增）:
   - **新增**: `pub fn analyze_connected_empty_regions(&self) -> Vec<RegionInfo>`:
     - 使用洪水填充算法分析所有连通的空白区域
     - 返回按大小排序的 `RegionInfo` 结构体数组，包含每个区域的详细信息
   - **新增**: `pub fn get_difficulty_score(&self) -> f32`:
     - 基于连通区域分析计算棋盘的困难程度（0.0-1.0）
     - 考虑因素包括：最大连通区域的大小、形状分数、区域碎片化程度、是否能容纳大型方块
     - 替代原有的简单填充率判断，提供更准确的游戏困难度评估

5. **RegionInfo 结构**（新增）:
   ```rust
   pub struct RegionInfo {
       pub cell_count: usize,        // 区域包含的格子数
       pub min_x: usize,             // 区域的最小X坐标
       pub max_x: usize,             // 区域的最大X坐标
       pub min_y: usize,             // 区域的最小Y坐标
       pub max_y: usize,             // 区域的最大Y坐标
       pub width: usize,             // 区域宽度
       pub height: usize,            // 区域高度
       pub shape_score: f32,         // 形状分数（0.0-1.0），越接近1.0表示越方正
       cells: Vec<(usize, usize)>,   // 区域包含的所有格子坐标
   }
   ```
   - `can_fit_4x4_block()`: 检查该区域是否能容纳4x4的方块
   - `is_square_like()`: 检查该区域是否偏向方形且足够大

6. **困难度评分系统优化**（2024年更新）:
   - **问题**：原有系统阈值过严，导致帮助功能很少触发
   - **改进的评分因素**：
     - 最大连通区域大小：调整阈值为 <6/12/20/30 格子对应不同困难度
     - 形状质量：降低权重但保持对狭长形状的惩罚
     - 碎片化程度：提高权重至0.35，更重视空间分割问题
     - **新增**空白格子占比：当空白格子少于30%或50%时增加困难度
     - 4x4方块容纳能力：降低权重至0.15
   - **帮助触发优化**：
     - 降低各阶段的困难度阈值，使帮助更容易触发
     - 添加更多分级，实现更平滑的帮助概率曲线
     - 即使在低困难度下也保留小概率帮助机会

### wave.rs (动态波次与难度管理器)

新增的 `WaveManager` 模块负责管理游戏的动态节奏、难度调整和周期性挑战，以取代原有的固定难度模式。

1.  **核心概念**:
    *   **回合 (`turn_count`)**: 玩家每成功放置一个方块计为一个回合。
    *   **波次阶段 (`WavePhase`)**: 游戏在 `Accumulation` (积累)、`ChallengeActive` (挑战激活)、`Relief` (缓和) 三个主要阶段间周期性切换。
    *   **方块复杂度因子 (`block_complexity_factor`)**: 一个动态调整的浮点数，影响生成方块的复杂程度。会根据总回合数和当前波次阶段进行调整。
    *   **挑战类型 (`ChallengeType`)**: 目前仅实现 `BlockFlood` (方块潮，提高复杂度)。

2.  **主要功能**:
    *   `new()`: 初始化 `WaveManager`，设置各阶段的默认持续回合数、初始复杂度等。
    *   `increment_turn() -> u32`: 推进游戏回合，更新当前阶段，调整难度，并返回该回合可能产生的奖励分数。
    *   `update_phase()`: 根据当前阶段已持续的回合数和配置的阶段长度，自动切换到下一个阶段。
    *   `transition_to()`: 处理阶段切换时的状态重置。
    *   `update_difficulty()`: 根据当前所处阶段和总回合数，调整 `block_complexity_factor`。
    *   `select_next_challenge()`: 为接下来的 `ChallengeActive` 阶段选择一个挑战类型 (目前固定为 `BlockFlood`)。
    *   `start_challenge()` / `end_challenge()`: 处理挑战开始和结束时的逻辑，包括奖励计算。
    *   `notify_line_cleared()`: 在行或列被消除时调用，用于更新挑战进度和计算相关奖励 (当前 `BlockFlood` 挑战下无特定逻辑)。
    *   **新增**: `pub fn should_offer_helpful_block(&self, grid_filled_ratio: f32) -> bool`:
        - 根据当前波次阶段和棋盘的填充比例 (`grid_filled_ratio`)，以一定的概率决定是否应该为玩家提供"帮助性"的小方块。在"缓和"阶段或"积累"阶段棋盘较满时，提供帮助的概率会更高。
    *   **新增**: `pub fn should_offer_helpful_block_v2(&self, difficulty_score: f32) -> bool`:
        - 基于连通区域分析的困难度分数（而非简单的填充率）决定是否提供帮助
        - 相比原版本，能更准确地识别玩家需要帮助的情况（如空白区域被分割成多个孤立小块）
        - 不同阶段有不同的帮助概率曲线：
          - 缓和阶段：最容易获得帮助，困难度0.7时几乎必定帮助
          - 积累阶段：中等帮助概率，根据困难度动态调整
          - 挑战阶段：较少帮助，但极度困难时仍会提供支援

3.  **与游戏主循环的集成 (`main.rs`)**:
    *   `Game` 结构体包含一个 `WaveManager` 实例。
    *   在 `Game::update_game()` 中，成功放置方块后调用 `wave_manager.increment_turn()`。
    *   `Game::generate_blocks()` 会从 `wave_manager` 获取当前的 `blocks_per_generation` (固定为3) 和 `block_complexity_factor`，并调用 `wave_manager.should_offer_helpful_block()` 来辅助决定是否生成"有用方块"。
    *   消除行/列后，会调用 `wave_manager.notify_line_cleared()`。
    *   游戏界面会显示当前的 `WavePhase` (当前挑战仅为方块潮)。

### effects.rs

实现游戏特效系统:

1. **粒子系统**:
   ```rust
   pub struct Particle {
       position: Vec2,
       velocity: Vec2,
       color: Color,
       size: f32,
       lifetime: f32,
       max_lifetime: f32,
   }

   pub struct ParticleSystem {
       particles: Vec<Particle>,
   }
   ```

2. **特效类型**:
   - 消除特效 (`show_clear_effect`)
   - 连击特效 (`show_combo_effect`)

3. **粒子行为**:
   - 随时间淡出
   - 物理模拟 (简单重力)
   - 随机生成和运动

### log.rs

新增的跨平台日志系统:

1. **日志级别**:
   ```rust
   pub enum LogLevel {
       Debug = 0,
       Info = 1,
       Warning = 2,
       Error = 3,
   }
   ```

2. **平台适配**:
   - WASM 环境：通过 JavaScript 桥接将日志输出到浏览器控制台
   - 非 WASM 环境：使用标准的 println!/eprintln! 输出到控制台

3. **便捷宏**:
   - `log_debug!`: 输出调试信息
   - `log_info!`: 输出一般信息
   - `log_warn!`: 输出警告信息
   - `log_error!`: 输出错误信息

4. **JavaScript 集成**:
   - 使用 Rust 的 `extern "C"` 声明 JavaScript 端的日志函数
   - 通过 `js_bridge.js` 中注册的 `console_log` 函数实现浏览器控制台输出
   - 支持不同日志级别对应不同控制台方法 (debug, info, warn, error)

### save.rs

处理游戏数据的保存和加载:

```rust
pub struct SaveData {
    pub high_score: u32,
}
```

在当前实现中是简化版本，提供了接口但没有实际持久化存储。

### cloud.rs

处理云服务和排行榜功能:

1. **玩家排名**:
   ```rust
   pub struct PlayerRank {
       pub user_id: String,
       pub name: String,
       pub score: u32,
       pub rank: u32,
   }
   ```

2. **云服务功能**:
   - 初始化云服务 SDK (`initialize_sdk`)
   - 用户登录 (`login`)
   - 上传分数 (`upload_score`)
   - 获取排行榜数据 (`get_leaderboard`)
   - 获取当前玩家排名 (`get_player_rank`)

3. **平台适配**:
   - WASM 环境：通过 JavaScript 调用云服务 API
   - 非 WASM 环境：使用模拟数据

4. **SCE SDK 集成**:
   - 通过 JavaScript 桥接调用 SCE Game SDK 的 API
   - 适配星火对战平台的排行榜和用户系统

### 新增构建信息显示

游戏新增了构建时间显示功能，帮助开发者追踪不同版本:

1. **构建时间生成**:
   - 使用 `build.rs` 脚本在编译时生成时间戳
   - 通过 Cargo 的构建系统将时间戳写入 `build_info.rs` 文件
   - 格式化显示时间和目标平台信息

2. **实现细节**:
   ```rust
   // build.rs
   let build_timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
   let build_info = format!("{} ({})", build_timestamp, target);
   write!(f, "pub const BUILD_TIMESTAMP: &str = \"{}\";", build_info).unwrap();
   ```

3. **界面集成**:
   - 在游戏左上角显示构建时间和WebAssembly JIT状态
   - 使用半透明小字体避免干扰游戏体验
   - 通过JavaScript桥接获取WASM JIT状态

### 优化的响应式布局

为解决不同屏幕比例下的显示问题，实现了新的自适应布局系统：

1. **屏幕比例控制**:
   - 在 `index_template.html` 中设置最大宽高比为2:3
   - 使用弹性盒模型(flex)使内容居中
   - CSS样式确保最佳屏幕利用率

2. **动态布局计算**:
   - 根据屏幕宽高比而非绝对尺寸计算元素位置
   - 为宽屏、标准屏和高屏提供不同的布局参数
   ```rust
   // 基于宽高比判断屏幕类型
   let aspect_ratio = screen_width() / screen_height();
   let is_wide_screen = aspect_ratio > 0.8;   // 宽屏
   let is_tall_screen = aspect_ratio < 0.5;   // 高屏
   ```

3. **垂直空间优化**:
   - 高屏设备上将网格下移至屏幕22%处
   - 将底部可拖拽区域上移至底部高度的40%处
   - 动态调整元素间距，适应不同屏幕大小

4. **安全区适配**:
   - 考虑移动设备刘海屏、圆角等特殊区域
   - 使用CSS环境变量确保内容不被系统UI元素遮挡

### 增强的错误显示系统

全新错误日志系统提供了更强大的调试能力：

1. **多级日志显示**:
   - 支持显示不同级别的日志：调试(蓝色)、信息(绿色)、警告(橙色)、错误(红色)
   - 每种级别使用不同颜色和标签直观区分
   - 默认显示错误和警告，而信息和调试可通过API调用显示

2. **交互功能**:
   - 可拖动日志窗口到屏幕任意位置
   - 提供关闭按钮手动隐藏日志
   - 清空按钮快速清除所有日志记录

3. **JavaScript桥接**:
   ```javascript
   // 在js_bridge.js中实现
   window.showGameMessage = function(message, level) {
       level = level || 1; // 默认为info级别
       displayErrorOnScreen(message, level);
   }
   ```

4. **日志持久化**:
   - 保留最近5条日志消息
   - 每条消息带有时间戳
   - 最新消息使用明亮色彩，旧消息使用暗色
   - 错误信息显示30秒，其他级别显示15秒

5. **视觉效果**:
   - 使用CSS动画实现淡入淡出过渡
   - 拖动时提供视觉反馈
   - 支持触摸屏和鼠标操作

## WASM调试系统

新增了更强大的WebAssembly错误和日志显示系统：

1. **显示层级**:
   - `game_log_js` 函数将Rust日志转发到JavaScript
   - 所有错误信息显示在屏幕上，同时发送到浏览器控制台
   - 警告级别默认也显示在屏幕上

2. **用法示例**:
   ```rust
   // Rust代码中使用
   log_debug!("调试信息"); // 仅控制台显示
   log_info!("普通信息");  // 仅控制台显示
   log_warn!("警告信息");  // 控制台和屏幕都显示
   log_error!("错误信息"); // 控制台和屏幕都显示
   ```

3. **JavaScript端使用**:
   ```javascript
   // 在浏览器控制台直接调用
   window.showGameMessage("自定义消息", 0); // 调试(蓝色)
   window.showGameMessage("自定义消息", 1); // 信息(绿色)
   window.showGameMessage("自定义消息", 2); // 警告(橙色)
   window.showGameMessage("自定义消息", 3); // 错误(红色)
   ```

4. **跨平台兼容**:
   - 在WebAssembly环境使用JavaScript显示错误
   - 在原生平台使用标准输出显示错误
   - 两个平台保持API一致性

## SCE SDK 集成详解

### 简介

SCE SDK (星火对战平台SDK) 是一个用于在星火对战平台中开发小游戏的工具包。本项目通过 JavaScript 桥接的方式将 SCE SDK 与 Rust 游戏代码集成，实现在线排行榜和用户系统功能。

### 集成方式

1. **前端集成**:
   - 在 `index_template.html` 中引入 SCE SDK
   - 创建 JavaScript 桥接函数，提供 Rust 代码调用的接口 (`sce_init_sdk`, `sce_get_user_info_for_rust`, `sce_upload_score`, `sce_get_leaderboard`, `sce_get_user_rank`)
   - 配置 SDK 参数和初始化设置

2. **后端集成**:
   - 在 `cloud.rs` 中通过 `invoke_js_with_result` 调用 JavaScript 函数
   - 解析 JSON 返回结果并转换为 Rust 数据结构
   - 提供平台无关的 API 接口

### 关键函数

1. **初始化 SDK**:
   ```rust
   pub async fn initialize_sdk() -> Result<(), String>
   ```
   调用 JavaScript 函数 `sce_init_sdk` 初始化 SDK，并设置开发者令牌和游戏ID。

2. **用户登录**:
   ```rust
   // pub async fn login() -> Result<(), String> // 旧函数，实际行为已变更
   // 在SDK初始化成功后，将通过内部调用 window.sce_get_user_info_for_rust() 获取用户信息
   // Rust 端的 CloudState 将直接包含用户信息，不再有显式的 login 函数供外部调用。
   // 内部初始化流程会调用 JS 函数 sce_get_user_info_for_rust 获取用户ID和名称。
   ```
   在 SDK 初始化 (`initialize_sdk`) 成功后，系统会自动尝试获取用户信息。

3. **上传分数**:
   ```rust
   pub async fn upload_score(score: u32) -> Result<(), String>
   ```
   调用 JavaScript 函数 `sce_upload_score` 上传玩家得分到排行榜。

4. **获取排行榜**:
   ```rust
   pub async fn get_leaderboard(limit: u32) -> Result<(), String>
   ```
   调用 JavaScript 函数 `sce_get_leaderboard` 获取指定数量的排行榜数据。

5. **获取玩家排名**:
   ```rust
   pub async fn get_player_rank() -> Result<(), String>
   ```
   调用 JavaScript 函数 `sce_get_user_rank` 获取当前玩家在排行榜中的排名。

### 运行模式

系统支持两种运行模式：

1. **在线模式** (WASM 环境):
   - 通过 SCE SDK 连接到星火对战平台
   - 提供真实的用户登录和排行榜功能
   - 支持跨设备数据同步
   - **需要在index_template.html中设置有效的开发者令牌**
   - 通过macroquad的wasm::invoke_js实现与JavaScript的交互

2. **本地模式** (非 WASM 环境):
   - 提供模拟的用户和排行榜数据
   - 在开发和测试阶段使用
   - 数据不会被保存或同步

### 使用须知

1. **开发者令牌设置**:
   - 必须在 `index_template.html` 文件中设置有效的开发者令牌
   - 格式: `developer_token: 'YOUR_DEVELOPER_TOKEN'` 需替换为实际令牌
   - 若未设置有效令牌，排行榜功能将无法正常工作

2. **排行榜数据结构**:
   - 用户排名数据使用 `PlayerRank` 结构体封装
   - 全局状态通过 `CloudState` 枚举管理
   - 所有数据通过 `CLOUD_STATE` 全局变量存储和访问

## 游戏功能

1. **核心玩法**:
   - 将方块拖放到 8x8 网格中
   - 填满整行或整列时消除方块
   - 同时消除越多，得分越高

2. **难度系统**:
   - **动态关卡系统 (`WaveManager`)**: 取代旧的固定模式，通过回合数驱动的波次阶段（积累、挑战、缓和）和动态调整的方块复杂度因子来控制游戏节奏和难度。
   - **情景感知方块生成**: 在特定阶段（如缓和阶段或积累阶段棋盘较满时），系统会尝试生成能帮助玩家解围的小型方块（如1x1或1x2）。

3. **分数系统**:
   - 基础分数：每次消除的行列数
   - 连击加成：连续消除增加连击数和分数
   - 最高分记录 (与云端集成)
   - **波次奖励**: 完成特定挑战或在某些阶段可能会有额外的分数奖励。

4. **视觉反馈**:
   - 方块拖放动画
   - 消除特效
   - 连击特效

## 未来扩展方向

1. **音效系统**:
   - 当前已定义接口但未实现
   - 可以添加背景音乐和游戏音效

2. **存档系统完善**:
   - 实现真正的数据持久化
   - 添加更多存档内容（如游戏设置等）

3. **游戏模式扩展**:
   - 添加计时模式
   - 进一步丰富挑战模式中的挑战类型
   - 考虑引入更复杂的关卡目标或机制

4. **多语言支持**:
   - 当前已支持中文
   - 可扩展为更完整的多语言系统

5. **移动端优化**:
   - 进一步优化触摸控制
   - 添加手机适配的 UI 布局

6. **情景感知方块生成增强**:
    - 对整组生成的方块进行可放置性检查，避免死局。
    - 扩展"有用方块"的种类和匹配逻辑。

## 开发指南

### 构建原生应用

```bash
cargo run --release
```

### 构建 WebAssembly 版本

```bash
# 添加 WASM 目标
rustup target add wasm32-unknown-unknown

# 构建 WASM
cargo build --release --target wasm32-unknown-unknown

# 安装 SCE SDK
cd web
npm install sce-game-sdk --save

# 处理 WASM 文件
# 可以使用 build_wasm_clean.bat (Windows) 或自行复制文件
```

### 配置 SCE SDK

在 `index_template.html` 中设置您的开发者配置:

```javascript
window.SCE_CONFIG = {
    developer_token: 'YOUR_DEVELOPER_TOKEN', // 替换为你的开发者令牌
    game_id: 'your_game_id',                 // 替换为你的游戏ID
    env: 'pd'                                // 环境: pd(生产环境)、alpha、beta等
};
```

### 运行服务器

```bash
python serve.py
```

# JavaScript和WASM集成

我们实现了一个自定义的JavaScript与Rust WASM交互机制，遵循Macroquad WASM规范：

1. **JavaScript桥接文件**：
   - `js_bridge.js` 包含了JavaScript侧的实现
   - 通过miniquad插件系统注册函数，供Rust调用
   - 处理内存分配和字符串传递

2. **Rust端接口**：
   - 使用`extern "C"`声明外部JavaScript函数
   - 提供内存分配函数供JavaScript使用
   - 使用安全包装函数处理字符串转换

3. **SCE SDK集成**：
   - 通过`invoke_js_with_result`函数调用SCE SDK的JavaScript API
   - 使用JSON进行数据交换
   - 提供完整的排行榜和用户管理功能

4. **日志系统集成**：
   - 通过JavaScript桥接将Rust日志输出到浏览器控制台
   - 根据日志级别使用不同的控制台函数(`console.debug`, `console.info`, `console.warn`, `console.error`)
   - 在原生平台上则使用标准输出
   - 提供统一的日志宏接口(`log_debug!`, `log_info!`, `log_warn!`, `log_error!`)

使用这种方法，我们能够在WASM环境中正确调用JavaScript函数，同时保持类型安全和内存安全。 