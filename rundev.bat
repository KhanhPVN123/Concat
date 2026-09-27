@echo off
setlocal EnableExtensions
title Concat - Dev Studio

rem Always run from this script's directory (works when double-clicked)
cd /d "%~dp0"

echo.
echo ========================================================
echo   Concat Video ^& Demo Studio - Quick Startup
echo ========================================================
echo   Directory: %CD%
echo.

rem 1. Check Rust toolchain
where cargo >nul 2>&1
if errorlevel 1 (
  echo [ERROR] Cargo/Rust not found on PATH.
  echo         Please install Rust from https://rustup.rs/
  goto :fail
)

rem Ensure CMake is on PATH
if exist "C:\Program Files\CMake\bin" (
  set "PATH=C:\Program Files\CMake\bin;%PATH%"
)

for /f "tokens=*" %%v in ('cargo --version 2^>nul') do set "CARGO_VER=%%v"
for /f "tokens=*" %%v in ('rustc --version 2^>nul') do set "RUSTC_VER=%%v"
echo   Rustc: %RUSTC_VER%
echo   Cargo: %CARGO_VER%

rem 2. Check and configure FFMPEG_DIR
if "%FFMPEG_DIR%"=="" (
  if exist "C:\Users\Khanh Pham\AppData\Local\Microsoft\WinGet\Packages\Gyan.FFmpeg.Shared_Microsoft.Winget.Source_8wekyb3d8bbwe\ffmpeg-9.0.2-full_build-shared" (
    set "FFMPEG_DIR=C:\Users\Khanh Pham\AppData\Local\Microsoft\WinGet\Packages\Gyan.FFmpeg.Shared_Microsoft.Winget.Source_8wekyb3d8bbwe\ffmpeg-9.0.2-full_build-shared"
  )
)

if not "%FFMPEG_DIR%"=="" (
  echo   FFmpeg: %FFMPEG_DIR%
  set "PATH=%FFMPEG_DIR%\bin;%PATH%"
) else (
  echo [WARNING] FFMPEG_DIR environment variable is not set.
)

rem 3. Ensure we navigate to the Rust workspace root (src)
if not exist "src\Cargo.toml" (
  echo [ERROR] src\Cargo.toml missing. Wrong folder?
  goto :fail
)

cd /d "%~dp0src"

echo.
echo [INFO] Starting Concat in dev mode (profile: quick)...
echo        Command: cargo dev
echo ========================================================
echo   Press Ctrl+C in this CMD window to stop.
echo ========================================================
echo.

cargo dev
set "EXIT_CODE=%ERRORLEVEL%"

echo.
if not "%EXIT_CODE%"=="0" (
  echo [ERROR] Application exited with code %EXIT_CODE%.
  goto :fail
)

goto :end

:fail
echo.
echo Press any key to close this window...
pause >nul
exit /b 1

:end
exit /b 0
