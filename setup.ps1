# Register background tasks for the WTR server + erg agent. Run once in an ADMIN PowerShell.
# Re-running just replaces them (-Force). Build the binaries yourself first, then restart to start.

$RepoRoot = $PSScriptRoot
$ServerExe = "$RepoRoot\target\release\wtr-server.exe"
$AgentExe = "$RepoRoot\target\release\wtr-erg.exe"

$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive
$settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -ExecutionTimeLimit ([TimeSpan]::Zero)

# Server: start at logon.
Register-ScheduledTask -TaskName 'WTR Server' -Force -Principal $principal -Settings $settings `
    -Action  (New-ScheduledTaskAction -Execute 'conhost.exe' -Argument "--headless `"$ServerExe`"" -WorkingDirectory $RepoRoot) `
    -Trigger (New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME)

# Erg agent: start 1 minute after logon so Bluetooth is ready.
$agentTrigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$agentTrigger.Delay = 'PT1M'
Register-ScheduledTask -TaskName 'WTR Erg Agent' -Force -Principal $principal -Settings $settings `
    -Action  (New-ScheduledTaskAction -Execute 'conhost.exe' -Argument "--headless `"$AgentExe`"" -WorkingDirectory $RepoRoot) `
    -Trigger $agentTrigger

Write-Host "Registered. Restart the PC to start them."
