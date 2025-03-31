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
echo 2.2 检查resources文件夹...
if exist resources (
  echo   - 复制resources文件夹内容...
  xcopy /E /I /Y resources web\resources > nul
  echo   - resources文件夹复制成功
) else (
  echo   - 注意：未找到resources文件夹，继续执行...
  mkdir web\resources
  echo   - 已创建空的resources文件夹
)

REM 3. 检查wasm32目标是否已安装
echo 3.检查wasm32-unknown-unknown目标...
rustup target list | findstr wasm32-unknown-unknown >nul
if errorlevel 1 (
  echo   - 安装wasm32-unknown-unknown目标...
  rustup target add wasm32-unknown-unknown
  if errorlevel 1 (
    echo   - 错误：安装wasm32-unknown-unknown失败！
    exit /b 1
  )
) else (
  echo   - wasm32-unknown-unknown目标已安装
)

REM 4. 构建WASM文件
echo 4. 构建WASM文件...
cargo build --release --target wasm32-unknown-unknown
if errorlevel 1 (
  echo   - 错误：构建WASM文件失败！
  exit /b 1
)

REM 5. 检查并复制WASM文件
echo 5. 复制WASM文件到web目录...
set WASM_FILE=target\wasm32-unknown-unknown\release\block_blast_bin.wasm
if exist %WASM_FILE% (
  copy /y %WASM_FILE% web\block_blast_bin.wasm > nul
  echo   - WASM文件复制成功
) else (
  set WASM_FILE=target\wasm32-unknown-unknown\release\block_blast.wasm
  if exist %WASM_FILE% (
    copy /y %WASM_FILE% web\block_blast_bin.wasm > nul
    echo   - WASM文件复制成功（使用替代文件名）
  ) else (
    echo   - 错误：未找到WASM文件！
    echo   - 请检查以下文件是否存在：
    echo     - target\wasm32-unknown-unknown\release\block_blast_bin.wasm
    echo     - target\wasm32-unknown-unknown\release\block_blast.wasm
    exit /b 1
  )
)

REM 6. 创建或复制HTML文件
echo 6. 准备HTML文件...
if exist index_template.html (
  echo   - 从模板复制HTML文件
  copy /y index_template.html web\index.html > nul
  echo   - HTML文件复制成功
) 


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