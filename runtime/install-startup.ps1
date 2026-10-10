param([switch]$RepairOnly,[switch]$Uninstall)
$ErrorActionPreference='Stop'
$zedRoot=Split-Path -Parent $PSScriptRoot
$zedLocal=Join-Path $zedRoot 'local'
$zedConfigPath=Join-Path $zedLocal 'settings.json'
$zedLauncher=Join-Path $zedLocal 'bin\zed-zh-launcher.exe'
$zedExe='D:\Zed\Zed.exe'
if(Test-Path -LiteralPath $zedConfigPath){$zedConfig=Get-Content -LiteralPath $zedConfigPath -Raw|ConvertFrom-Json;$zedExe=$zedConfig.zed_exe}
$zedShell=New-Object -ComObject WScript.Shell
$zedBackup=Join-Path $zedLocal 'shortcut-backups'
$zedInventoryPath=Join-Path $zedLocal 'shortcut-inventory.json'
New-Item -ItemType Directory -Path $zedBackup -Force|Out-Null
if($Uninstall){
    Get-CimInstance Win32_Process -Filter "Name = 'zed-zh-launcher.exe'" | Where-Object{$_.ExecutablePath -ieq $zedLauncher -and $_.CommandLine -match '--watch'} | ForEach-Object{Stop-Process -Id $_.ProcessId}
    if(Test-Path -LiteralPath $zedInventoryPath){
        $zedInventory=Get-Content -LiteralPath $zedInventoryPath -Raw|ConvertFrom-Json
        foreach($zedEntry in $zedInventory){Copy-Item -LiteralPath $zedEntry.backup -Destination $zedEntry.path -Force}
    }
    Remove-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'ZedZhShortcutGuard' -ErrorAction SilentlyContinue
    Write-Output 'Original shortcuts restored; official executable was not modified.'
    exit 0
}
if(-not(Test-Path -LiteralPath $zedLauncher)){throw 'Build/install the Rust GUI launcher before replacing shortcuts.'}
$zedLinkRoots=@([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('StartMenu'),"$env:APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar")
$zedInventory=@()
if(Test-Path -LiteralPath $zedInventoryPath){$zedInventory=@(Get-Content -LiteralPath $zedInventoryPath -Raw|ConvertFrom-Json)}
$zedChanged=0
foreach($zedLinkRoot in $zedLinkRoots){
    if(-not(Test-Path -LiteralPath $zedLinkRoot)){continue}
    Get-ChildItem -LiteralPath $zedLinkRoot -Filter '*.lnk' -Recurse -ErrorAction SilentlyContinue|ForEach-Object{
        $zedLinkPath=$_.FullName
        $zedLink=$zedShell.CreateShortcut($_.FullName)
        if($zedLink.TargetPath -ieq $zedExe -or $zedLink.TargetPath -ieq (Join-Path (Split-Path -Parent $zedExe) 'bin\Zed.exe')){
            if(-not($zedInventory|Where-Object{$_.path -eq $zedLinkPath})){
                $zedBackupFile=Join-Path $zedBackup (([Guid]::NewGuid().ToString())+'.lnk')
                Copy-Item -LiteralPath $_.FullName -Destination $zedBackupFile
                $zedInventory+=@{path=$_.FullName;backup=$zedBackupFile}
            }
            $zedLink.TargetPath=$zedLauncher
            $zedLink.WorkingDirectory=$zedRoot
            $zedLink.IconLocation=$zedExe+',0'
            $zedLink.Description='Zed 中文补丁启动器（官方更新渠道）'
            $zedLink.Save()
            $zedChanged++
        }
    }
}
ConvertTo-Json -InputObject @($zedInventory) -Depth 4|Set-Content -LiteralPath $zedInventoryPath -Encoding utf8
if(-not $RepairOnly){
    $zedMenu=Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\Zed'
    New-Item -ItemType Directory -Path $zedMenu -Force|Out-Null
    foreach($zedItem in @(@{Name='Zed 汉化设置';Args='--settings'},@{Name='Zed（原版英文）';Args='--english'})){
        $zedLink=$zedShell.CreateShortcut((Join-Path $zedMenu ($zedItem.Name+'.lnk')))
        $zedLink.TargetPath=$zedLauncher;$zedLink.Arguments=$zedItem.Args;$zedLink.WorkingDirectory=$zedRoot;$zedLink.IconLocation=$zedExe+',0';$zedLink.Save()
    }
    New-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'ZedZhShortcutGuard' -Value ('"'+$zedLauncher+'" --watch') -PropertyType String -Force|Out-Null
}
Write-Output ('Zed shortcuts repaired: '+$zedChanged)
