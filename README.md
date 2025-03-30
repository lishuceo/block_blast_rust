# block_blast_rust
This is a cross-platform mini-game inspired by Block Blast, built with Cursor and Rust.
# 方块消除游戏 - 纯净WASM版本

这是方块消除游戏的纯净WebAssembly版本，完全不依赖wasm-bindgen。

## 特点

- 纯净构建，不使用wasm-bindgen
- 自定义随机数生成器，替代rand和getrandom库
- 更小的WASM文件大小
- 更简单的构建过程
- 直接通过URL加载JavaScript依赖

## 构建方法

### 一键构建并运行

```bash
# 最简单的方法 - 一步完成构建和启动服务器
run_game.bat
```

### 分步构建

```bash
# 仅构建WASM文件
build_wasm_clean.bat

# 启动服务器
python serve.py
```

## 自定义随机数生成器

本项目使用了自定义的随机数生成器（`src/random.rs`），完全不依赖外部库，适合WebAssembly环境使用。

### 使用方法

```rust
// 导入随机数模块
use crate::random;

// 生成范围内的随机整数
let random_number = random::gen_range(0, 10); // 生成0到9之间的随机数

// 生成范围内的随机浮点数
let random_float = random::gen_range_f32(0.0, 1.0);

// 从切片中随机选择元素
let random_item = random::choose(&my_array);

// 或者实例化自己的随机数生成器
let mut rng = random::SimpleRandom::new_from_time();
let value = rng.gen_range(1, 100);
```

## 技术细节

- 使用Xorshift算法生成随机数
- 随机数种子来源于macroquad的时间函数
- 支持整数、浮点数范围随机和随机选择功能
- 无需任何外部依赖

## 可能的问题和解决方法

1. **无法加载JavaScript文件**: 
   如果macroquad的JavaScript依赖无法加载，请尝试使用本地版本替代CDN版本。

2. **构建错误**: 
   确保您已经删除了Cargo.toml中的rand和getrandom依赖。

3. **浏览器缓存问题**:
   使用Ctrl+F5强制刷新浏览器缓存。

4. **"找不到模块"错误**:
   确保lib.rs中导出了random模块。

## 目录结构

- `src/random.rs` - 自定义随机数生成器
- `build_wasm_clean.bat` - 纯净构建脚本
- `run_game.bat` - 一键构建并运行脚本
- `serve.py` - 简单的HTTP服务器脚本
- `web/` - 生成的Web文件目录 
