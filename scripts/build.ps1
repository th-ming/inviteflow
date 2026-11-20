$env:RUSTUP_TOOLCHAIN = 'stable-x86_64-pc-windows-gnu'
$env:PATH = 'D:\Tools\mingw\mingw64\bin;' + $env:PATH
Set-Location -LiteralPath (Join-Path $PSScriptRoot '..\backend')

cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
