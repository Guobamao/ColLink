@echo off
title ColLink (Debug Mode)
cd /d "%~dp0"

if not exist "%~dp0backend\.venv\Scripts\python.exe" (
    echo [ColLink] Python 虚拟环境未在 backend\.venv 找到，请先配置环境。
    pause
    exit /b 1
)

echo ============================================================
echo   ColLink - 数据字段血缘与可视化映射平台 (调试控制台模式)
echo ============================================================
echo.
"%~dp0backend\.venv\Scripts\python.exe" "%~dp0desktop.py"
pause
