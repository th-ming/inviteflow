param(
  [Parameter(Mandatory = $true)]
  [string]$Domain
)

$root = Split-Path -Parent $PSScriptRoot
& (Join-Path $PSScriptRoot 'set-domain.ps1') -Domain $Domain

if (-not $env:FIREBASE_TOKEN) {
  throw 'Can dat FIREBASE_TOKEN truoc khi chay deploy.'
}

Set-Location -LiteralPath $root
npm install -g firebase-tools
firebase deploy --only hosting --token $env:FIREBASE_TOKEN --non-interactive

'FE da deploy. BE: day code len repo da lien ket Leapcell de tu dong build.'
'Leapcell: Root=backend, Build=cargo build --release, Start=./target/release/inviteflow-api, Port=8080'
'GitHub secrets can them: FIREBASE_TOKEN, API_BASE'
