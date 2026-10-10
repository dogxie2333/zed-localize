param(
    [Parameter(Mandatory=$true)][string]$Launcher,
    [Parameter(Mandatory=$true)][string]$State,
    [ValidateSet('tray','editor')][string]$Scenario='tray',
    [int]$Seconds=30
)
$ErrorActionPreference='Stop'
$env:ZED_ZH_STATE_DIR=$State
function Measure-IdleProcess([int]$ProcessId) {
    $first=Get-Process -Id $ProcessId
    $cpu=$first.TotalProcessorTime.TotalSeconds
    $timer=[Diagnostics.Stopwatch]::StartNew()
    $samples=@()
    1..$Seconds | ForEach-Object {
        $current=Get-Process -Id $ProcessId
        $samples+=@{working_set_mib=$current.WorkingSet64/1MB;private_bytes_mib=$current.PrivateMemorySize64/1MB}
        Start-Sleep -Seconds 1
    }
    $current=Get-Process -Id $ProcessId
    $cpuDelta=$current.TotalProcessorTime.TotalSeconds-$cpu
    return [ordered]@{
        seconds=$timer.Elapsed.TotalSeconds
        cpu_seconds=$cpuDelta
        cpu_percent_one_core=100*$cpuDelta/$timer.Elapsed.TotalSeconds
        cpu_percent_machine=100*$cpuDelta/$timer.Elapsed.TotalSeconds/[Environment]::ProcessorCount
        working_set_mib_max=($samples.working_set_mib|Measure-Object -Maximum).Maximum
        private_bytes_mib_max=($samples.private_bytes_mib|Measure-Object -Maximum).Maximum
    }
}
if($Scenario -eq 'tray') {
    $owned=Start-Process -FilePath $Launcher -ArgumentList '--watch' -WindowStyle Hidden -PassThru
    try {
        Start-Sleep -Seconds 3
        $result=[ordered]@{scenario='tray';measurement=(Measure-IdleProcess $owned.Id);status=(Get-Content -LiteralPath (Join-Path $State 'tray-status.json') -Raw|ConvertFrom-Json);children=@(Get-CimInstance Win32_Process|Where-Object {$_.ParentProcessId -eq $owned.Id}|Select-Object Name,ProcessId)}
        $result|ConvertTo-Json -Depth 6
    } finally {
        Start-Process -FilePath $Launcher -ArgumentList '--stop' -WindowStyle Hidden -Wait
    }
} else {
    $profile=Join-Path $State 'isolated-zed'
    New-Item -ItemType Directory -Path (Join-Path $profile 'config') -Force|Out-Null
    '{"auto_update":false,"telemetry":{"metrics":false,"diagnostics":false}}'|Set-Content -LiteralPath (Join-Path $profile 'config\settings.json') -Encoding utf8
    $savedStateless=$env:ZED_STATELESS
    $env:ZED_STATELESS='1'
    $owned=Start-Process -FilePath 'D:\Zed\Zed.exe' -ArgumentList @('--user-data-dir',$profile) -WindowStyle Hidden -PassThru
    $env:ZED_STATELESS=$savedStateless
    try {
        Start-Sleep -Seconds 12
        $before=Measure-IdleProcess $owned.Id
        '{"enabled":true}'|Set-Content -LiteralPath (Join-Path $State 'gadget-control.json') -Encoding ascii
        Start-Process -FilePath $Launcher -ArgumentList @('--attach',$owned.Id) -WindowStyle Hidden -Wait
        Start-Sleep -Seconds 5
        $after=Measure-IdleProcess $owned.Id
        $stats=@(Get-ChildItem -LiteralPath (Join-Path $State 'gadgets') -Filter stats.json -Recurse|ForEach-Object {Get-Content -LiteralPath $_.FullName -Raw|ConvertFrom-Json}|Where-Object {$_.pid -eq $owned.Id})
        if(-not($stats|Where-Object {$_.mode -eq 'chinese'})){throw 'Active Chinese display engine was not confirmed'}
        [ordered]@{scenario='same isolated Zed idle before and after active hooks; no frame-time benchmark';before=$before;after=$after;working_set_delta_mib=$after.working_set_mib_max-$before.working_set_mib_max;private_bytes_delta_mib=$after.private_bytes_mib_max-$before.private_bytes_mib_max;stats=$stats}|ConvertTo-Json -Depth 6
    } finally {
        # Only the editor created above belongs to this measurement.
        if(-not $owned.HasExited -and $owned.Path -eq 'D:\Zed\Zed.exe'){Stop-Process -Id $owned.Id}
    }
}
