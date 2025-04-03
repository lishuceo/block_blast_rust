# block_blast_rust
This is a cross-platform mini-game inspired by Block Blast, built with Cursor and Rust.

Demo : https://minigame.spark.xd.com/block_blast_rust/

# 方块消除游戏 - 纯净WASM版本

这是方块消除游戏的纯净WebAssembly版本，完全不依赖wasm-bindgen。

## 特点

- 纯净构建，不使用wasm-bindgen
- 更小的WASM文件大小
- 更简单的构建过程
- 直接通过URL加载JavaScript依赖

## 原生构建与运行

如果您想在本地运行原生版本（非WebAssembly），请按照以下步骤操作：

### 1. 构建与运行（一步完成）

```bash
# 直接构建并运行游戏
cargo run --release
```

### 2. 分步构建

```bash
# 只构建不运行
cargo build --release

# 运行已构建的游戏
./target/release/block_blast_bin  # Linux/macOS
# 或
.\target\release\block_blast_bin.exe  # Windows
```

## WebAssembly构建步骤

### 1. 安装WebAssembly目标

首先确保安装了WebAssembly编译目标：

```bash
rustup target add wasm32-unknown-unknown
```

### 2. 编译WebAssembly文件

```bash
cargo build --release --target wasm32-unknown-unknown
```

### 3. 准备web目录

```bash
# 创建web目录
mkdir -p web/resources

# 复制WASM文件
cp target/wasm32-unknown-unknown/release/block_blast_bin.wasm web/
# 或者可能是：
# cp target/wasm32-unknown-unknown/release/block_blast.wasm web/block_blast_bin.wasm

# 复制JavaScript支持文件
cp mq_js_bundle.js web/

# 复制HTML模板（如果存在）
cp index_template.html web/index.html
# 如果没有index_template.html，需要创建一个基本的HTML文件
```

### 4. 启动本地服务器

可以使用Python的简易HTTP服务器：

#### Python 3:
```bash
# 在web目录中启动服务器
cd web
python -m http.server 8000
```

#### 或者使用Node.js:
```bash
# 安装http-server（如果尚未安装）
npm install -g http-server

# 启动服务器
http-server web -p 8000
```

然后访问 http://localhost:8000/ 即可运行游戏。

## 技术细节

- 使用macroquad和miniquad框架开发
- 针对WebAssembly环境优化
- 无需任何额外的JavaScript依赖

## 可能的问题和解决方法

1. **无法加载JavaScript文件**: 
   如果macroquad的JavaScript依赖无法加载，请尝试使用本地版本替代CDN版本。

2. **浏览器缓存问题**:
   使用Ctrl+F5强制刷新浏览器缓存。

3. **编译错误 - 找不到core**:
   出现 `error[E0463]: can't find crate for 'core'` 错误时，确保已安装 wasm32-unknown-unknown 目标。

4. **HTTP服务器问题**:
   - 确保您的计算机已安装Python或Node.js
   - 如果需要从其他设备访问，可以使用 `python -m http.server 8000 --bind 0.0.0.0` 绑定到所有网络接口

5. **原生构建问题**:
   - 确保已安装Rust和Cargo
   - 如果缺少依赖库，请根据错误信息安装所需的系统库
   - 对于Windows用户，可能需要安装适当的Visual C++构建工具

## 目录结构

- `src/` - 游戏源代码
- `web/` - 生成的Web文件目录
- `resources/` - 资源文件（如图像、音效等） 
