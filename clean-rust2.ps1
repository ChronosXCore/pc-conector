$target = "F:\Programas Desarrollados\Pc conector\rustup-home\toolchains"
if (Test-Path $target) {
    Remove-Item -Path $target -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "Toolchains folder removed"
} else {
    Write-Host "Toolchains folder not found"
}
$tmp = "F:\Programas Desarrollados\Pc conector\rustup-home\tmp"
if (Test-Path $tmp) {
    Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "Tmp folder removed"
}
$dl = "F:\Programas Desarrollados\Pc conector\rustup-home\downloads"
if (Test-Path $dl) {
    Remove-Item -Path $dl -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "Downloads folder removed"
}
Write-Host "DONE"
