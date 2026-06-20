@echo off
:: NetBridge - Script de inicio
echo ====================================
echo   NetBridge - Iniciando...
echo ====================================

:: Configurar entorno de compilacion con msys64 + cmake wrapper
set CARGO_HOME=F:\Programas Desarrollados\Pc conector\cargo-cache
set RUSTUP_HOME=F:\Programas Desarrollados\Pc conector\rustup-home
set CC=F:\msys64\mingw64\bin\gcc.exe
set CXX=F:\msys64\mingw64\bin\g++.exe
set AR=F:\msys64\mingw64\bin\ar.exe
set CC_x86_64_pc_windows_gnu=F:\msys64\mingw64\bin\gcc.exe

:: cmake-bin contiene un wrapper que inyecta -DCMAKE_POLICY_VERSION_MINIMUM=3.5
:: para compatibilidad de la libreria Opus con CMake 4.x
set PATH=F:\Programas Desarrollados\Pc conector\cmake-bin;F:\msys64\mingw64\bin;F:\Programas Desarrollados\Pc conector\cargo-cache\bin;F:\Programas Desarrollados\Pc conector\rustup-home\toolchains\stable-x86_64-pc-windows-gnu\bin;%PATH%

echo ====================================
echo   Iniciando servidor de desarrollo...
echo ====================================
cd /d "F:\Programas Desarrollados\Pc conector\pc-conector"
call npm run tauri:dev

echo ====================================
echo   NetBridge detenido.
echo ====================================
pause
