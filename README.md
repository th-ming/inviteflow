# Inviteflow

Hệ thống lời mời đi chơi gồm 7 bước, tách thành 3 lớp cloud:

- FE: HTML/CSS/JS thuần, deploy Firebase Hosting.
- BE: Rust + axum, deploy Leapcell.
- DB: Firebase Realtime Database (Spark).

Không cần build FE, không chạy local trong cấu hình mặc định.

## Bắt đầu

Đọc [STRUCTURE.md](STRUCTURE.md) để xem cây thư mục, [docs/deploy.md](docs/deploy.md) để deploy, [docs/api.md](docs/api.md) để biết API.

## Chạy test

```
cd backend
cargo test
```

## Backup

```
cd backend
cargo run --release -- --backup > ../database/backup.json
```
