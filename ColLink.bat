@echo off
title ColLink
cd /d "%~dp0"

if not exist "%~dp0backend\.venv\Scripts\pythonw.exe" (
    echo [ColLink] Python 虚拟环境未在 backend\.venv 找到，请先配置环境。
    pause
    exit /b 1
)

start "" "%~dp0backend\.venv\Scripts\pythonw.exe" "%~dp0desktop.py"
exit
