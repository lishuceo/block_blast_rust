# Block Blast Rust 项目技术文档

## 项目概述

Block Blast Rust 是一个基于 Rust 和 macroquad 引擎开发的方块消除游戏。游戏的主要目标是将不同形状的方块放置到 8x8 的网格中，当填满整行或整列时消除方块获得分数。游戏设计为跨平台的，支持桌面和 WebAssembly (WASM) 环境。

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
│   ├── save.rs             # 游戏存档和加载
│   ├── effects.rs          # 粒子效果和视觉特效
│   ├── cloud.rs            # 云服务和排行榜实现
│   ├── random.rs           # 随机数生成工具
│   ├── log.rs              # 跨平台日志系统
│   └── minimal.rs          # 最小化示例代码
├── resources/              # 资源文件目录
│   └── fonts/              # 字体文件
├── web/                    # WebAssembly 构建目录
│   └── node_modules/       # 前端依赖库
│       └── sce-game-sdk/   # SCE游戏SDK
├── Cargo.toml              # 项目依赖配置
├── Cargo.lock              # 依赖锁定文件
├── build_wasm_clean.bat    # WASM 构建脚本 (Windows)
├── serve.py                # 简易 HTTP 服务器
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

3. **界面渲染函数**:
   - `draw_main_menu` 函数负责渲染主菜单界面
   - `draw_menu` 函数负责渲染传统菜单界面
   - `draw_game` 函数负责渲染游戏界面
   - `draw_game_over` 函数负责渲染游戏结束界面
   - `draw_leaderboard` 函数负责渲染排行榜界面

4. **游戏逻辑更新**:
   - `update_game` 函数处理游戏逻辑更新
   - `update_main_menu_animations` 处理主菜单动画更新
   - 处理用户输入、方块移动、消除检测等

5. **中文文本支持**:
   - 实现了 `draw_chinese_text` 函数支持中文显示
   - 使用嵌入式字体资源

6. **3D 效果渲染**:
   - `draw_cube_block` 函数实现立体感方块绘制

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

2. **方块类型**:
   - 简单形状 (1-2 个方块)
   - 标准形状 (类俄罗斯方块形状)
   - 复杂形状 (更多单元格或不规则形状)

3. **方块生成**:
   - `random` 方法随机生成方块
   - `random_with_chances` 根据概率生成不同复杂度的方块

### grid.rs

管理游戏网格和方块放置:

1. **网格结构**:
   ```rust
   pub struct Grid {
       pub cells: [[Option<Color>; 8]; 8],
   }
   ```

2. **网格操作**:
   - `can_place_block` 检查方块是否可放置
   - `place_block` 在网格上放置方块
   - `check_and_clear` 检查并消除已填满的行和列
   - `draw` 渲染网格和已放置的方块

3. **辅助功能**:
   - `can_place_block_with_tolerance` 提供放置容错，改善用户体验

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

## 关键技术实现

### 跨平台适配

1. **条件编译**:
   - 使用 `#[cfg(target_arch = "wasm32")]` 条件编译，区分 WASM 和原生平台
   - 根据平台提供不同实现

2. **DPI 缩放**:
   - 通过 `get_dpi_scale()` 函数动态检测和适配不同显示设备的 DPI

3. **输入适配**:
   - 同时支持鼠标和触摸输入
   - 在移动设备上提供更大的触摸判定区域

### 中文文本渲染

1. **字体加载**:
   - 使用 `include_bytes!` 嵌入字体资源
   - 通过 `once_cell` 实现字体的懒加载

2. **文本绘制**:
   - `draw_chinese_text` 函数处理中文显示
   - 支持居中对齐和基线调整

### 3D 效果渲染

使用 2D 渲染技术模拟 3D 效果:

1. **立体方块**:
   - 使用多个三角形绘制立体感方块
   - 计算亮面和暗面颜色，产生 3D 效果

2. **粒子效果**:
   - 使用二维粒子系统创建爆炸、消除等特效

### WebAssembly 集成

1. **纯净 WASM 构建**:
   - 不依赖 wasm-bindgen，通过 macroquad 直接生成 WASM

2. **JavaScript 交互**:
   - 通过 `macroquad::miniquad::invoke_js_with_result` 实现 WASM 与 JavaScript 的交互
   - 实现云服务和排行榜功能

3. **SCE SDK 集成**:
   - 使用 JavaScript 桥接函数连接 Rust 代码和 SCE SDK
   - 提供登录、排行榜、分数上传等功能
   - 支持离线和在线两种模式

4. **WASM 日志系统**:
   - 通过 JavaScript 桥接将 Rust 日志输出到浏览器控制台
   - 支持不同级别的日志（调试、信息、警告、错误）
   - 提供便捷的日志宏接口，统一不同平台的日志体验

## SCE SDK 集成详解

### 简介

SCE SDK (星火对战平台SDK) 是一个用于在星火对战平台中开发小游戏的工具包。本项目通过 JavaScript 桥接的方式将 SCE SDK 与 Rust 游戏代码集成，实现在线排行榜和用户系统功能。

### 集成方式

1. **前端集成**:
   - 在 `index_template.html` 中引入 SCE SDK
   - 创建 JavaScript 桥接函数，提供 Rust 代码调用的接口
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
   pub async fn login() -> Result<(), String>
   ```
   调用 JavaScript 函数 `sce_login` 进行用户登录，获取用户ID和名称。

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
   - 简单模式：增加简单方块的生成概率
   - 普通模式：标准方块概率更高
   - 可调整方块数量和生成概率

3. **分数系统**:
   - 基础分数：每次消除的行列数
   - 连击加成：连续消除增加连击数和分数
   - 最高分记录

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
   - 添加挑战模式
   - 关卡系统

4. **多语言支持**:
   - 当前已支持中文
   - 可扩展为更完整的多语言系统

5. **移动端优化**:
   - 进一步优化触摸控制
   - 添加手机适配的 UI 布局

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