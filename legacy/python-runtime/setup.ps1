param([string]$ZedExe='D:\Zed\Zed.exe',[switch]$WithSettings)
$ErrorActionPreference='Stop'
$zedRoot=Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $zedRoot
if(-not(Test-Path -LiteralPath $ZedExe)){throw 'Pass -ZedExe with the installed official GUI executable.'}
python -m venv .venv
if($LASTEXITCODE -ne 0){throw 'Python 3.12 or later is required.'}
& .\.venv\Scripts\python.exe -m pip install -r runtime/requirements-runtime.txt
if($LASTEXITCODE -ne 0){throw 'Runtime dependency installation failed.'}
if($WithSettings){
    & .\.venv\Scripts\python.exe -m pip install -r runtime/requirements-ui.txt 'tree-sitter>=0.25,<0.26' 'tree-sitter-rust>=0.24,<0.25'
    if($LASTEXITCODE -ne 0){throw 'Settings dependency installation failed.'}
}
New-Item -ItemType Directory -Path local/bin -Force|Out-Null
if(Test-Path -LiteralPath 'zed-zh-launcher.exe'){
    Copy-Item -LiteralPath 'zed-zh-launcher.exe' -Destination local/bin/zed-zh-launcher.exe -Force
}else{
    cargo build --release --locked --bin zed-zh-launcher
    if($LASTEXITCODE -ne 0){throw 'Launcher build failed.'}
    Copy-Item -LiteralPath target/release/zed-zh-launcher.exe -Destination local/bin/zed-zh-launcher.exe -Force
}
if(Test-Path -LiteralPath 'zed-zh-maintain.exe'){
    Copy-Item -LiteralPath 'zed-zh-maintain.exe' -Destination local/bin/zed-zh-maintain.exe -Force
}elseif($WithSettings){
    cargo build --release --locked --features ai --bin zed-zh-addon
    if($LASTEXITCODE -ne 0){throw 'Maintenance build failed.'}
    Copy-Item -LiteralPath target/release/zed-zh-addon.exe -Destination local/bin/zed-zh-maintain.exe -Force
}
$zedSettingsPath=Join-Path $zedRoot 'local/settings.json'
$zedSettings=@{}
if(Test-Path -LiteralPath $zedSettingsPath){
    $zedExisting=Get-Content -LiteralPath $zedSettingsPath -Raw|ConvertFrom-Json
    $zedExisting.PSObject.Properties|ForEach-Object{$zedSettings[$_.Name]=$_.Value}
}
$zedSettings['zed_exe']=$ZedExe
$zedSettings|ConvertTo-Json|Set-Content -LiteralPath $zedSettingsPath -Encoding utf8
& ./runtime/install-startup.ps1
Start-Process -FilePath ./local/bin/zed-zh-launcher.exe -ArgumentList '--watch'
Write-Output 'Installed. Save work and close Zed, then use the original desktop shortcut.'
