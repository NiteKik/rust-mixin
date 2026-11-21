@echo off
setlocal
chcp 65001 >nul
where pwsh >nul 2>&1
if %ERRORLEVEL%==0 (
    set "PS=pwsh"
) else (
    set "PS=powershell"
)
pushd "%~dp0" || exit /b 1
%PS% -NoProfile -ExecutionPolicy Bypass -File "scripts\video-demo.ps1" %*
set "DEMO_EXIT=%ERRORLEVEL%"
popd
endlocal & exit /b %DEMO_EXIT%
