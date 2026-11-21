# Bản đồ dự án — Inviteflow

## Cây thư mục

```
inviteflow/
├── README.md              tổng quan, lệnh nhanh
├── STRUCTURE.md           bản đồ này
├── .env.example           mẫu biến môi trường BE
├── .gitignore
├── firebase.json          cấu hình Firebase Hosting
├── .github/workflows/     deploy FE + keep-alive API
├── frontend/
│   └── public/            FE tĩnh, Render static site trỏ vào đây
│       ├── index.html     câu hỏi mở đầu, ảnh mèo
│       ├── thanks.html    cảm ơn + chuyển bước
│       ├── date.html      chọn ngày
│       ├── food.html      chọn món (lưới ảnh)
│       ├── drink.html     chọn nước (lưới ảnh)
│       ├── place.html     chọn nơi (lưới ảnh)
│       ├── done.html      phiếu hẹn + gửi API
│       ├── admin.html     bảng quản lý (giao diện tĩnh + API thật, login qua /api/login)
│       ├── assets/css/    style.css (layout rail + lưới ảnh)
│       ├── assets/img/    logo-mark.png, logo-micro.png, favicon.png (bộ nhận diện v1.0)
│       ├── assets/js/     app.js (localStorage/UI), api.js (HTTP)
│       ├── robots.txt
│       ├── sitemap.xml
│       └── og-image.svg   ảnh dùng cho og:image
├── backend/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs        entrypoint, --backup
│   │   ├── config.rs      env, ADMIN_TOKEN bắt buộc
│   │   ├── model.rs       NewResponse, ResponseRecord
│   │   ├── validate.rs    validation + unit test
│   │   ├── db.rs          GitHub Contents API adapter (responses.json)
│   │   └── api.rs         routes, CORS, rate limit, integration test
│   └── .cargo/config.toml linker cho toolchain GNU trên Windows
├── database/
│   └── README.md          schema node, rules, backup
├── scripts/
│   ├── set-domain.ps1     thay YOUR_PROJECT + YOUR_API_HOST
│   ├── deploy-fe.ps1      deploy FE qua Firebase token (không dùng)
│   ├── backup.ps1         xuất content + responses qua /api/export
│   └── build.ps1          fmt + test + clippy + release build
└── docs/
    ├── architecture.md    sơ đồ + quyết định thiết kế
    ├── api.md             endpoint, body, lỗi
    └── deploy.md          Render + DNSHE + Cloudflare tunnel
```

## Luồng

```
FE (Render static) -> API (Render Rust web service) -> GitHub JSON DB (responses.json + content.json + admin.json)
```

Admin `/api/content` (GET public, PUT cần token editor/admin), `/api/responses` (GET public, POST public, PATCH/DELETE cần token), `/api/export` + `/api/restore` (cần token).

## Đổi tên miền

Chạy `scripts/set-domain.ps1 -Domain https://ten-mien-moi`, rồi cập nhật ALLOWED_ORIGIN trong env API và custom domain trong Render.
