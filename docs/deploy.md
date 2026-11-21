# Deploy

## Trạng thái hiện tại

- **FE**: Render static site `inviteflow-fe` → `https://inviteflow-fe.onrender.com`.
- **Domain chính**: `https://motloimoi.cc.cd` — đã verify trong Render, CNAME `@` → `inviteflow-fe.onrender.com` khai trong DNSHE.
- **API**: Render web service `inviteflow-api2` → `https://inviteflow-api2.onrender.com` (Rust native, build `cargo build --release`, start `./target/release/inviteflow-api`, rootDir `backend`).
- **DB**: GitHub repo `th-ming/inviteflow-db`, file `responses.json` (GitHub Contents API).
- **Admin**: `https://motloimoi.cc.cd/admin.html`. Token lấy trong `D:\creds-REXI.md`, không lưu trong repo.

Không còn dùng quick tunnel. IP thật lấy từ header `cf-connecting-ip`, tra địa lý qua `ipwho.is`.

## DNSHE

API: `https://api005.dnshe.com/index.php?m=domain_hub`, auth qua header `X-API-Key` / `X-API-Secret`.

- Domain: `motloimoi.cc.cd`, `subdomain_id = 6982875840`.
- Record: `CNAME @ → inviteflow-fe.onrender.com`, TTL 600, `record_id = 783802758711327`.
- Credential nằm trong `D:\creds-REXI.md`, không đưa vào repo.

## Build

```
powershell -File scripts/build.ps1
```

Yêu cầu toolchain GNU + MinGW tại `D:\Tools\mingw` (đã cấu hình trong `backend/.cargo/config.toml`).

## Đổi domain

```
powershell -File scripts/set-domain.ps1 -Domain https://ten-mien-moi
```

Script cập nhật robots, sitemap, og:image/canonical và `ALLOWED_ORIGIN`. Nhớ cập nhật lại biến môi trường `ALLOWED_ORIGIN` của API và custom domain trong Render.

## Backup

```
$env:INVITEFLOW_ADMIN_TOKEN = '<token lấy từ D:\creds-REXI.md>'
powershell -File scripts/backup.ps1
```

Script gọi `GET /api/export`, ghi một file JSON có timestamp vào `backups/` (nội dung + phản hồi). Đặt lịch Task Scheduler hằng ngày nếu cần tự động. Không lưu token trong repo hay script.

## Lịch sử

- Firebase Hosting / Leapcell / Stackryze `.indevs.in` đã bị bỏ vì thiếu tài khoản hoặc KYC.
- DNSHE được chọn vì có API key hợp lệ, đăng ký domain không cần KYC GitHub.
