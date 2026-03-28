# Leuwi Panjang Terminal — Windows Installer (PowerShell)
Write-Host "Leuwi Panjang Terminal — Windows Installer" -ForegroundColor Green

# Check Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rust..."
    Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile "$env:TEMP\rustup-init.exe"
    Start-Process "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

$InstallDir = "$env:LOCALAPPDATA\leuwi-panjang"

if (Test-Path $InstallDir) {
    Set-Location $InstallDir
    git pull
} else {
    git clone "https://github.com/situkangsayur/leuwi-panjang.git" $InstallDir
    Set-Location $InstallDir
}

Write-Host "Building..."
cargo build --release

$BinDir = "$env:LOCALAPPDATA\leuwi-panjang\target\release"
Write-Host "Installed at: $BinDir\leuwi-panjang.exe"
Write-Host "Add to PATH: $BinDir"
