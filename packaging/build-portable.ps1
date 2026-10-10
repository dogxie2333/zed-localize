param([string]$Toolchain='1.99.0-x86_64-pc-windows-gnu')
$ErrorActionPreference='Stop'
$zedRoot=Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $zedRoot
$env:RUSTC_BOOTSTRAP='dll_syringe'
$env:CARGO_BUILD_JOBS='1'
$zedCache=Join-Path (Split-Path -Parent $zedRoot) '.build-cache/zed-zh-addon/cache'
New-Item -ItemType Directory -Path (Join-Path $zedCache 'temp') -Force|Out-Null
$env:CARGO_HOME=Join-Path $zedCache 'cargo-home'
$env:npm_config_cache=Join-Path $zedCache 'npm'
$env:TEMP=Join-Path $zedCache 'temp'
$env:TMP=$env:TEMP
python tools/download_gadget.py
if($LASTEXITCODE -ne 0){throw 'Pinned Gadget download failed'}
Push-Location -LiteralPath (Join-Path $zedRoot 'web')
try {npm ci --no-audit --no-fund;if($LASTEXITCODE -ne 0){throw 'Frontend dependencies failed'};npm run build;if($LASTEXITCODE -ne 0){throw 'Frontend build failed'}} finally {Pop-Location}
cargo "+$Toolchain" build --release --locked --features ai,ui,portable --bins
if($LASTEXITCODE -ne 0){throw 'Rust component build failed'}
python packaging/make_payload.py
if($LASTEXITCODE -ne 0){throw 'Native payload packaging failed'}
python packaging/append_payload.py
if($LASTEXITCODE -ne 0){throw 'Single-file assembly failed'}
Get-FileHash -LiteralPath dist/Zed-Localize.exe
