@echo off
title ColLink (Debug Mode)
cd /d "%~dp0"

if not exist "%~dp0backend\.venv\Scripts\python.exe" (
    echo [ColLink] Python venv not found in backend\.venv
    pause
    exit /b 1
)

echo ============================================================
echo   ColLink - Desktop Studio (Debug Mode)
echo ============================================================
echo.
"%~dp0backend\.venv\Scripts\python.exe" "%~dp0desktop.py"
pause
