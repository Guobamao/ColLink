@echo off
title DataField Mapper Launcher
cls
echo ===================================================
echo   Starting DataField Mapper (Dev Server)
echo ===================================================

cd /d "%~dp0backend"
start "DataField-Backend" cmd /k "title DataField Backend (Port 8000) && .venv\Scripts\python.exe -m uvicorn app.main:app --host 127.0.0.1 --port 8000 --reload"

ping 127.0.0.1 -n 3 >nul

cd /d "%~dp0frontend"
start "DataField-Frontend" cmd /k "title DataField Frontend (Port 5173) && npm run dev"

echo.
echo ===================================================
echo   Services are running in separate windows:
echo   - Frontend: http://localhost:5173
echo   - Backend:  http://127.0.0.1:8000/docs
echo ===================================================
echo.
pause
