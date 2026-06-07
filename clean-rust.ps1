Remove-Item -Path "F:\Programas Desarrollados\Pc conector\rustup-home\toolchains" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "F:\Programas Desarrollados\Pc conector\rustup-home\tmp" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "F:\Programas Desarrollados\Pc conector\rustup-home\downloads" -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "Limpieza completada"
