# ColLink 一键启动脚本 (PowerShell)

Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "  启动 ColLink " -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

$CurrentDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $CurrentDir) { $CurrentDir = Get-Location }

# 1. 启动后端服务
Write-Host "`n[1/2] 正在启动 FastAPI 后端服务 (端口: 8000)..." -ForegroundColor Yellow
$BackendProcess = Start-Process -FilePath "$CurrentDir\backend\.venv\Scripts\python.exe" `
    -ArgumentList "-m", "uvicorn", "app.main:app", "--host", "127.0.0.1", "--port", "8000", "--reload" `
    -WorkingDirectory "$CurrentDir\backend" `
    -PassThru

# 2. 启动前端服务
Write-Host "[2/2] 正在启动 Vite 前端服务 (端口: 5173)..." -ForegroundColor Yellow
$FrontendProcess = Start-Process -FilePath "cmd.exe" `
    -ArgumentList "/c", "npm", "run", "dev" `
    -WorkingDirectory "$CurrentDir\frontend" `
    -PassThru

Write-Host "`nColLink 服务启动完成！" -ForegroundColor Green
Write-Host "前端界面访问地址: http://localhost:5173" -ForegroundColor Green
Write-Host "后端 API 文档地址: http://127.0.0.1:8000/docs" -ForegroundColor Green
Write-Host "`n按 Ctrl+C 或关闭本窗口以停止服务..." -ForegroundColor Gray

try {
    Wait-Process -Id $FrontendProcess.Id
} finally {
    Write-Host "`n正在停止后台服务..." -ForegroundColor Red
    if ($BackendProcess -and -not $BackendProcess.HasExited) { Stop-Process -Id $BackendProcess.Id -Force }
    if ($FrontendProcess -and -not $FrontendProcess.HasExited) { Stop-Process -Id $FrontendProcess.Id -Force }
}