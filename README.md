# block_blast_rust
This is a cross-platform mini-game inspired by Block Blast, built with Cursor and Rust.
# 方块消除游戏 - 纯净WASM版本

这是方块消除游戏的纯净WebAssembly版本，完全不依赖wasm-bindgen。

## 特点

- 纯净构建，不使用wasm-bindgen
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

## 技术细节

- 使用macroquad和miniquad框架开发
- 针对WebAssembly环境优化
- 无需任何额外的JavaScript依赖

## 可能的问题和解决方法

1. **无法加载JavaScript文件**: 
   如果macroquad的JavaScript依赖无法加载，请尝试使用本地版本替代CDN版本。

2. **浏览器缓存问题**:
   使用Ctrl+F5强制刷新浏览器缓存。

## 目录结构

- `build_wasm_clean.bat` - 纯净构建脚本
- `run_game.bat` - 一键构建并运行脚本
- `serve.py` - 简单的HTTP服务器脚本
- `web/` - 生成的Web文件目录 
