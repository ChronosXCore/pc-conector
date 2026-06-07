# ============================================================
# build.ps1  -  PC Conector - Script de Compilacion Completo
# Uso: .\build.ps1          -> compila el instalador
#      .\build.ps1 -Dev     -> inicia en modo desarrollo
# ============================================================
param([switch]$Dev)

# Configurar rutas de Rust/Cargo (GNU toolchain - no requiere Visual Studio)
$env:RUSTUP_HOME = "F:\Programas Desarrollados\Pc conector\rustup-home"
$env:CARGO_HOME  = "F:\Programas Desarrollados\Pc conector\cargo-cache"
$env:PATH = "F:\Programas Desarrollados\Pc conector\cargo-cache\bin;" +
            "F:\Programas Desarrollados\Pc conector\rustup-home\toolchains\stable-x86_64-pc-windows-gnu\bin;" +
            "F:\msys64\usr\bin;" +
            "F:\msys64\mingw64\bin;" +
            $env:PATH

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   PC Conector - Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Cargo: $(cargo --version)" -ForegroundColor Green
Write-Host ""

Set-Location "F:\Programas Desarrollados\Pc conector\pc-conector"

if ($Dev) {
    Write-Host "Iniciando en modo DESARROLLO..." -ForegroundColor Yellow
    npx tauri dev
} else {
    Write-Host "Compilando instalador (esto puede tardar 5-15 minutos)..." -ForegroundColor Yellow
    npx tauri build
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "BUILD EXITOSO!" -ForegroundColor Green
        $installer = Get-ChildItem "src-tauri\target\release\bundle" -Recurse -Filter "*.msi" | Select-Object -First 1
        if ($installer) {
            Write-Host "Instalador MSI: $($installer.FullName)" -ForegroundColor Cyan
        }
        $exe = Get-ChildItem "src-tauri\target\release\bundle" -Recurse -Filter "*.exe" | Where-Object { $_.Name -like "*setup*" -or $_.Name -like "*install*" } | Select-Object -First 1
        if ($exe) {
            Write-Host "Instalador EXE: $($exe.FullName)" -ForegroundColor Cyan
        }
    } else {
        Write-Host "BUILD FALLIDO. Revisa los errores de arriba." -ForegroundColor Red
    }
}
