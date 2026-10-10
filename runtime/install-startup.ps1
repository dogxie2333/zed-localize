param([switch]$RepairOnly,[switch]$Uninstall)
$ErrorActionPreference='Stop'
$zedInstallMutex=New-Object System.Threading.Mutex($false,'Local\ZedZhShortcutInstall')
$zedInstallAcquired=$false
try {
    try {$zedInstallAcquired=$zedInstallMutex.WaitOne(30000)} catch [System.Threading.AbandonedMutexException] {$zedInstallAcquired=$true}
    if(-not $zedInstallAcquired){throw 'Another shortcut installer is still running.'}
$zedRoot=Split-Path -Parent $PSScriptRoot
$zedLocal=Join-Path $zedRoot 'local'
if($env:ZED_ZH_STATE_DIR){$zedLocal=$env:ZED_ZH_STATE_DIR}
$zedConfigPath=Join-Path $zedLocal 'settings.json'
$zedLauncher=Join-Path $zedLocal 'bin\zed-zh-launcher.exe'
if($env:ZED_ZH_LAUNCHER){$zedLauncher=$env:ZED_ZH_LAUNCHER}
$zedWorkingDirectory=Split-Path -Parent $zedLauncher
$zedExe='D:\Zed\Zed.exe'
$zedAutoStart=$false
$zedReplaceShortcuts=$true
if(Test-Path -LiteralPath $zedConfigPath){
    $zedConfig=Get-Content -LiteralPath $zedConfigPath -Raw|ConvertFrom-Json
    $zedExe=$zedConfig.zed_exe
    $zedAutoStart=$zedConfig.startup_mode -eq 'auto'
    if($zedConfig.PSObject.Properties.Name -contains 'replace_shortcuts'){$zedReplaceShortcuts=[bool]$zedConfig.replace_shortcuts}
}
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
if($RepairOnly -and -not $zedReplaceShortcuts){Write-Output 'Shortcut takeover disabled.';exit 0}
if(-not(Test-Path -LiteralPath $zedLauncher)){throw 'Build/install the Rust GUI launcher before replacing shortcuts.'}
$zedLinkRoots=@([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('StartMenu'),"$env:APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar")
$zedInventory=@()
if(Test-Path -LiteralPath $zedInventoryPath){$zedInventory=@(Get-Content -LiteralPath $zedInventoryPath -Raw|ConvertFrom-Json)}
if(-not $zedReplaceShortcuts){
    foreach($zedEntry in $zedInventory){
        if(Test-Path -LiteralPath $zedEntry.path){
            $zedExistingLink=$zedShell.CreateShortcut($zedEntry.path)
            if($zedExistingLink.TargetPath -ieq $zedLauncher){Copy-Item -LiteralPath $zedEntry.backup -Destination $zedEntry.path -Force}
        }
    }
}
$zedRetired=Join-Path $zedLocal 'retired-shortcuts'
$zedLegacySettings=Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\Zed\Zed 汉化设置.lnk'
if(-not $RepairOnly -and (Test-Path -LiteralPath $zedLegacySettings)){
    $zedSettingsLink=$zedShell.CreateShortcut($zedLegacySettings)
    if((Split-Path -Leaf $zedSettingsLink.TargetPath) -eq 'zed-zh-launcher.exe' -and $zedSettingsLink.Arguments -eq '--settings'){
        New-Item -ItemType Directory -Path $zedRetired -Force|Out-Null
        Move-Item -LiteralPath $zedLegacySettings -Destination (Join-Path $zedRetired 'Zed 汉化设置.lnk') -Force
    }
}
$zedChanged=0
foreach($zedLinkRoot in $zedLinkRoots){
    if(-not(Test-Path -LiteralPath $zedLinkRoot)){continue}
    Get-ChildItem -LiteralPath $zedLinkRoot -Filter '*.lnk' -Recurse -ErrorAction SilentlyContinue|ForEach-Object{
        $zedLinkPath=$_.FullName
        $zedLink=$zedShell.CreateShortcut($_.FullName)
        if([string]::IsNullOrWhiteSpace($zedLink.TargetPath)){return}
        if($zedReplaceShortcuts -and (((Split-Path -Leaf $zedLink.TargetPath) -eq 'zed-zh-launcher.exe' -and $zedLink.Arguments -eq '' -and $zedLink.TargetPath -ine $zedLauncher) -or $zedLink.TargetPath -ieq $zedExe -or $zedLink.TargetPath -ieq (Join-Path (Split-Path -Parent $zedExe) 'bin\Zed.exe'))){
            if(-not($zedInventory|Where-Object{$_.path -eq $zedLinkPath})){
                $zedBackupFile=Join-Path $zedBackup (([Guid]::NewGuid().ToString())+'.lnk')
                Copy-Item -LiteralPath $_.FullName -Destination $zedBackupFile
                $zedInventory+=@{path=$_.FullName;backup=$zedBackupFile}
            }
            $zedLink.TargetPath=$zedLauncher
            $zedLink.WorkingDirectory=$zedWorkingDirectory
            $zedLink.IconLocation=$zedLauncher+',0'
            $zedLink.Description='Zed 中文补丁启动器（官方更新渠道）'
            $zedLink.Save()
            $zedChanged++
        }
        elseif($zedReplaceShortcuts -and $zedLink.TargetPath -ieq $zedLauncher -and $zedLink.Arguments -in @('','--watch')){
            if($zedLink.IconLocation -ine ($zedLauncher+',0') -or $zedLink.WorkingDirectory -ine $zedWorkingDirectory){
                $zedLink.IconLocation=$zedLauncher+',0';$zedLink.WorkingDirectory=$zedWorkingDirectory;$zedLink.Save()
            }
        }
    }
}
$zedInventoryTemporary=$zedInventoryPath+'.'+[Guid]::NewGuid().ToString()+'.tmp'
ConvertTo-Json -InputObject @($zedInventory) -Depth 4|Set-Content -LiteralPath $zedInventoryTemporary -Encoding utf8
Move-Item -LiteralPath $zedInventoryTemporary -Destination $zedInventoryPath -Force
if(-not $RepairOnly){
    $zedMenu=Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\Zed'
    New-Item -ItemType Directory -Path $zedMenu -Force|Out-Null
    foreach($zedItem in @(@{Name='Zed 中文';Args=''},@{Name='Zed 汉化补丁';Args='--watch'},@{Name='Zed（原版英文）';Args='--english'})){
        $zedLink=$zedShell.CreateShortcut((Join-Path $zedMenu ($zedItem.Name+'.lnk')))
        $zedLink.TargetPath=$zedLauncher;$zedLink.Arguments=$zedItem.Args;$zedLink.WorkingDirectory=$zedWorkingDirectory
        $zedLink.IconLocation=if($zedItem.Args -eq '--english'){$zedExe+',0'}else{$zedLauncher+',0'}
        $zedLink.Save()
    }
    $zedTrayLink=$zedShell.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Desktop')) 'Zed 汉化补丁.lnk'))
    $zedTrayLink.TargetPath=$zedLauncher;$zedTrayLink.Arguments='--watch';$zedTrayLink.WorkingDirectory=$zedWorkingDirectory;$zedTrayLink.IconLocation=$zedLauncher+',0';$zedTrayLink.Save()
    if($zedAutoStart){
        New-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'ZedZhShortcutGuard' -Value ('"'+$zedLauncher+'" --watch') -PropertyType String -Force|Out-Null
    }else{
        Remove-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'ZedZhShortcutGuard' -ErrorAction SilentlyContinue
    }
}
Write-Output ('Zed shortcuts repaired: '+$zedChanged)
} finally {
    if($zedInstallAcquired){$zedInstallMutex.ReleaseMutex()}
    $zedInstallMutex.Dispose()
}
