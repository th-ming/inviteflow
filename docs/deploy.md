# Deploy

## FE — Firebase Hosting

1. Tạo project Firebase id `YOUR_PROJECT` (6–30 ký tự), chọn gói **Spark**.
2. Bật Hosting, giữ `firebase.json` như cũ (`public: frontend/public`).
3. `firebase deploy --only hosting` → site chạy tại `https://YOUR_PROJECT.web.app`.
4. Domain đã chọn: **https://motloimoi.indevs.in** (đuôi `.indevs.in` từ Stackryze — free managed domain, repo `stackryze/FreeDomains`). Thêm domain này trong Firebase Hosting, lấy bản ghi A/TXT Firebase rồi khai ở DNS Stackryze; HTTPS do Firebase cấp.
5. Nếu chưa kịp gắn domain tùy chỉnh, Firebase vẫn có `https://<project>.web.app` làm địa chỉ dự phòng.

Kiểm tra thực tế từ bảng giá chính thức: Spark **“No payment method needed”**, Hosting 10 GB storage + 360 MB/day transfer, Realtime Database 1 GB lưu + 100 kết nối + ~10 GB/tháng. Dùng trong hạn mức này thì không phát sinh tiền.

## BE — Leapcell

Runtime: Rust. Khai trong Service Settings:

| Field | Value |
|---|---|
| Root Directory | `backend` |
| Build Command | `cargo build --release` |
| Start Command | `./target/release/inviteflow-api` |
| Port | 8080 |

Biến môi trường lấy từ `.env.example`: `ADMIN_TOKEN`, `DB_DRIVER=firebase`, `DATABASE_URL` (bắt buộc — server từ chối khởi động nếu còn giá trị mẫu), `DATABASE_PATH`, `ALLOWED_ORIGIN=https://YOUR_PROJECT.web.app`, `RATE_LIMIT`.

FE gọi API qua `window.API_BASE` trong từng trang — phải thay `YOUR_API_HOST` bằng domain Leapcell sau khi deploy BE, nếu không request sẽ đi về Firebase Hosting và trả 404.

Health check: `GET /api/health`. Deploy tự động khi push vào branch đã liên kết.

## DB — Firebase Realtime Database

Tạo database trên Spark, cấu hình node `responses`. Không đưa service account vào repo; nếu dùng Admin SDK thì khai trong secret của host.

## GitHub Actions

- `deploy-fe.yml`: deploy FE khi push vào `main`. Cần secret `FIREBASE_TOKEN`.
- `keep-alive.yml`: cron 5 phút gọi `GET /api/health`. Cần secret `API_BASE` (domain Leapcell).

## Keep-alive

Nếu muốn tránh cold start, dùng workflow `keep-alive.yml`, UptimeRobot hoặc cron-job.org gọi `GET /api/health` mỗi 5–10 phút. Ngân sách 100k invocation/tháng của Leapcell đủ cho việc này.

## Thay placeholder

Domain FE đã ghi: `motloimoi.indevs.in`. Còn lại `YOUR_API_HOST` trong 8 trang — sau khi deploy BE:

```
powershell -File scripts/set-domain.ps1 -ApiBase https://<app>.leapcell.dev
```

Script cũng nhận `-Domain https://ten-mien` (tự bỏ `https://` thừa) và `-Project <id>` nếu cần đổi lại. Cập nhật `ALLOWED_ORIGIN` trong `.env` khi chạy.

## Kiểm tra free

- Firebase Spark: bảng giá chính thức ghi rõ “No payment method needed”. Hosting + RTDB nằm trong hạn mức ở trên.
- Leapcell: catalog độc lập ghi free, không thẻ, 20 project / 100k invocation / 3 GB-giờ / 2 GB transfer; trang pricing chính thức trả 502 lúc kiểm tra — cần xác nhận lại trong lúc đăng ký.
