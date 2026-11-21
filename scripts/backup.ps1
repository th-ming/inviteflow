param(
  [string]$Api = 'https://inviteflow-api2.onrender.com',
  [string]$Out = (Join-Path $PSScriptRoot '..\backups')
)
if (-not $env:INVITEFLOW_ADMIN_TOKEN) { throw 'Đặt INVITEFLOW_ADMIN_TOKEN trước khi chạy' }
New-Item -ItemType Directory -Path $Out -Force | Out-Null
$file = 'inviteflow-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.json'
$data = Invoke-RestMethod -Method Get -Uri ($Api.TrimEnd('/') + '/api/export') -Headers @{ Authorization = 'Bearer ' + $env:INVITEFLOW_ADMIN_TOKEN }
$data | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $Out $file) -Encoding UTF8
Write-Host ('Saved ' + $file)
