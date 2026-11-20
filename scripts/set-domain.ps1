param(
  [string]$Project,
  [string]$ApiBase,
  [string]$Domain
)

if (-not $Project -and -not $ApiBase -and -not $Domain) {
  throw 'Can truyen -Project, -ApiBase hoac -Domain.'
}

$root = Split-Path -Parent $PSScriptRoot
$changed = @()
$origin = if ($Domain) { "https://" + ($Domain.Trim() -replace '^https?://', '').TrimEnd('/') } else { '' }

$files = Get-ChildItem -LiteralPath (Join-Path $root 'frontend\public') -Recurse -File |
  Where-Object { $_.Extension -in @('.html', '.txt', '.xml') }

foreach ($file in $files) {
  $content = Get-Content -LiteralPath $file.FullName -Raw -Encoding UTF8
  $original = $content

  if ($Domain) {
    $content = $content -replace 'https://YOUR_PROJECT\.web\.app', $origin
    $content = $content -replace 'https://YOUR_DOMAIN', $origin
    $content = $content -replace 'YOUR_DOMAIN', $origin.TrimEnd('/')
  } elseif ($Project) {
    $content = $content -replace 'YOUR_PROJECT', $Project.Trim()
  }

  if ($ApiBase) {
    $content = $content -replace 'YOUR_API_HOST', $ApiBase.Trim().TrimEnd('/')
  }

  if ($content -ne $original) {
    [System.IO.File]::WriteAllText($file.FullName, $content, [System.Text.UTF8Encoding]::new($false))
    $changed += $file.FullName.Replace($root + '\', '')
  }
}

$envFile = Join-Path $root '.env'
if ((Test-Path -LiteralPath $envFile) -and ($Domain -or $Project)) {
  $envOrigin = if ($Domain) { $origin } else { "https://$($Project.Trim()).web.app" }
  $lines = Get-Content -LiteralPath $envFile -Encoding UTF8
  $lines = $lines | ForEach-Object {
    if ($_ -match '^ALLOWED_ORIGIN=') { "ALLOWED_ORIGIN=$envOrigin" } else { $_ }
  }
  [System.IO.File]::WriteAllLines($envFile, $lines, [System.Text.UTF8Encoding]::new($false))
  $changed += '.env (ALLOWED_ORIGIN)'
}

if ($changed.Count -eq 0) { 'Khong co placeholder de thay.' }
else { $changed | ForEach-Object { "updated: $_" } }
