@echo off
REM 设置UTF-8编码
chcp 65001 > nul

echo =========================================
echo === 方块消除游戏 - 一键构建并运行 ===
echo =========================================
echo.

REM 先运行构建脚本
echo 正在构建WASM版本...
call build_wasm_clean.bat

REM 检查构建是否成功
if %errorlevel% neq 0 (
  echo.
  echo 构建失败，无法继续！
  echo 请检查错误信息。
  pause
  exit /b 1
)

REM 启动服务器
echo.
echo 构建成功，现在启动服务器...
echo.
echo 请在浏览器中访问 http://localhost:8000/
echo 按Ctrl+C可以停止服务器
echo.

REM 运行服务器
python serve.py 