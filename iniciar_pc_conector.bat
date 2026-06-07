@echo off
:: PC Conector - Script de inicio
:: Configura el entorno y ejecuta la aplicación en modo desarrollo

echo ====================================
echo   PC Conector - Iniciando...
echo ====================================

:: Configurar rutas
set CARGO_HOME=F:\pc-conector-cargo
set RUSTUP_HOME=F:\pc-conector-rustup
set CC=F:\winlibs\mingw64\bin\gcc.exe
set CXX=F:\winlibs\mingw64\bin\g++.exe
set AR=F:\winlibs\mingw64\bin\ar.exe
set CC_x86_64_pc_windows_gnu=F:\winlibs\mingw64\bin\gcc.exe
set PATH=F:\pc-conector-rustup\toolchains\stable-x86_64-pc-windows-gnu\bin;F:\winlibs\mingw64\bin;%PATH%
set TEMP=C:\Temp
set TMP=C:\Temp

:: Ir al directorio del proyecto
cd /d "F:\Programas Desarrollados\Pc conector\pc-conector"

echo Iniciando servidor de desarrollo...
npm run tauri:dev

pause
