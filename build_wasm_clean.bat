@echo off
REM 设置UTF-8编码
chcp 65001 > nul

echo =========================================
echo === 方块消除游戏 - 纯净WASM构建脚本 ===
echo =========================================
echo.

REM 1. 清理旧文件和目录
echo 1. 清理旧文件...
if exist web (
  echo   - 备份web目录...
  if exist web.bak rmdir /s /q web.bak
  rename web web.bak
)
if exist target\wasm32-unknown-unknown (
  echo   - 清理旧构建目录...
  rmdir /s /q target\wasm32-unknown-unknown
)

REM 2. 创建新目录
echo 2. 创建目录结构...
mkdir web

REM 2.1 检查并复制WASM文件
echo 2.1 复制mq_js_bundle.js文件到web目录...
if exist mq_js_bundle.js (
  copy /y mq_js_bundle.js web\mq_js_bundle.js > nul
  echo   - mq_js_bundle.js文件复制成功
) else (
  echo   - 错误：未找到mq_js_bundle.js文件！
  exit /b 1
)

REM 2.2 检查并复制resource文件夹
echo test1111
echo resource文件夹到web目录...
if exist resource (
  echo   - 复制resource文件夹内容...
  xcopy /E /I /Y resource web\resource > nul
  echo   - resource文件夹复制成功
) else (
  echo   - 错误：未找到resource文件夹！
  exit /b 1
)

REM 3. 检查wasm32目标是否已安装
echo 3. 检查wasm32-unknown-unknown目标...
rustup target list | findstr wasm32-unknown-unknown >nul
if %errorlevel% neq 0 (
  echo   - 安装wasm32-unknown-unknown目标...
  rustup target add wasm32-unknown-unknown
  if %errorlevel% neq 0 (
    echo   - 错误：安装wasm32-unknown-unknown失败！
    exit /b 1
  )
) else (
  echo   - wasm32-unknown-unknown目标已安装
)

REM 4. 构建WASM文件
echo 4. 构建WASM文件...
cargo build --release --target wasm32-unknown-unknown
if %errorlevel% neq 0 (
  echo   - 错误：构建WASM文件失败！
  exit /b 1
)

REM 5. 检查并复制WASM文件
echo 5. 复制WASM文件到web目录...
if exist target\wasm32-unknown-unknown\release\block_blast_bin.wasm (
  copy /y target\wasm32-unknown-unknown\release\block_blast_bin.wasm web\block_blast_bin.wasm > nul
  echo   - WASM文件复制成功
) else (
  echo   - 错误：未找到WASM文件！
  exit /b 1
)


REM 6. 创建或复制HTML文件
echo 6. 准备HTML文件...
if exist index_template.html (
  echo   - 从模板复制HTML文件
  copy /y index_template.html web\index.html > nul
  echo   - HTML文件复制成功
) else (
  echo   - 创建新的HTML文件...
  (
  echo ^<!DOCTYPE html^>
  echo ^<html lang="zh"^>
  echo ^<head^>
  echo     ^<meta charset="utf-8"^>
  echo     ^<title^>方块消除游戏^</title^>
  echo     ^<style^>
  echo         html, body, canvas {
  echo             margin: 0;
  echo             padding: 0;
  echo             width: 100%%;
  echo             height: 100%%;
  echo             overflow: hidden;
  echo             position: absolute;
  echo             background: black;
  echo             z-index: 0;
  echo         }
  echo     ^</style^>
  echo ^</head^>
  echo ^<body^>
  echo     ^<canvas id="glcanvas" tabindex='1'^>^</canvas^>
  echo     ^<!-- 加载macroquad的JavaScript捆绑包 --^>
  echo     ^<script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"^>^</script^>
  echo     ^<script^>
  echo         // 加载WASM文件
  echo         load("block_blast.wasm");
  echo     ^</script^>
  echo ^</body^>
  echo ^</html^>
  ) > web\index.html
  echo   - HTML文件创建成功
)

REM 7. 创建简单的Python服务器脚本
echo 7. 创建服务器脚本...
(
echo import http.server, socketserver
echo import functools
echo.
echo PORT = 8000
echo Handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory='web'^)
echo.
echo print(f"启动服务器在端口 {PORT}..."^)
echo print(f"请访问 http://localhost:{PORT}/"^)
echo print("按Ctrl+C退出服务器"^)
echo.
echo with socketserver.TCPServer(("", PORT^), Handler^) as httpd:
echo     try:
echo         httpd.serve_forever(^)
echo     except KeyboardInterrupt:
echo         print("\n服务器已停止"^)
) > serve.py

echo.
echo ========== 构建完成! ==========
echo.
echo WASM文件: web\block_blast_bin.wasm
echo HTML文件: web\index.html
echo.
echo 运行以下命令启动服务器:
echo   python serve.py
echo.
echo 在浏览器中访问 http://localhost:8000/
echo 如果出现问题，请尝试使用Ctrl+F5强制刷新
echo.

pause 