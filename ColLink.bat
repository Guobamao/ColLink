@echo off
title ColLink
cd /d "%~dp0"

if not exist "%~dp0backend\.venv\Scripts\pythonw.exe" (
    echo [ColLink] Python venv not found in backend\.venv
    pause
    exit /b 1
)

start "" "%~dp0backend\.venv\Scripts\pythonw.exe" "%~dp0desktop.py"
exit
