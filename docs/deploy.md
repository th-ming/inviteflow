# Deploy

## Trạng thái hiện tại

- **FE**: Render static site `inviteflow-fe` → `https://inviteflow-fe.onrender.com`.
- **Domain chính**: `https://motloimoi.cc.cd` — đã verify trong Render, CNAME `@` → `inviteflow-fe.onrender.com` khai trong DNSHE.
- **API**: Cloudflare quick tunnel `https://minneapolis-exhibits-hip-relates.trycloudflare.com` → Rust chạy local cổng 8787.
- **DB**: GitHub repo `th-ming/inviteflow-db`, file `responses.json` (GitHub Contents API).
- **Admin**: `https://motloimoi.cc.cd/pages/admin.html`, token mặc định `inviteflow-admin-2026`.

Quick tunnel đổi URL khi restart. Muốn cố định cần named tunnel (Cloudflare account) hoặc host BE có domain riêng.

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

## Lịch sử

- Firebase Hosting / Leapcell / Stackryze `.indevs.in` đã bị bỏ vì thiếu tài khoản hoặc KYC.
- DNSHE được chọn vì có API key hợp lệ, đăng ký domain không cần KYC GitHub.
