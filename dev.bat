@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo [ColLink] 启动 Tauri v2 开发环境...
npm run tauri dev
