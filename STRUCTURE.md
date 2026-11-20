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
│   └── public/            FE tĩnh, Firebase Hosting trỏ vào đây
│       ├── index.html     câu hỏi mở đầu
│       ├── pages/         6 trang còn lại + admin
│       ├── assets/css/    style.css (token/chung) + admin.css (quản lý)
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
│   │   ├── db.rs          Firebase REST adapter
│   │   └── api.rs         routes, CORS, rate limit, integration test
│   └── .cargo/config.toml linker cho toolchain GNU trên Windows
├── database/
│   └── README.md          schema node, rules, backup
├── scripts/
│   ├── set-domain.ps1     thay YOUR_PROJECT + YOUR_API_HOST
│   ├── deploy-fe.ps1      deploy FE khi co FIREBASE_TOKEN
│   └── build.ps1          fmt + test + clippy + release build
└── docs/
    ├── architecture.md    sơ đồ + quyết định thiết kế
    ├── api.md             endpoint, body, lỗi
    └── deploy.md          Firebase Hosting + Leapcell + keep-alive
```

## Luồng

```
FE (Firebase Hosting) -> API (Leapcell) -> Firebase RTDB
```

## Đổi tên miền

Cập nhật 4 chỗ: `firebase.json` (nếu đổi public), `robots.txt`, `sitemap.xml` (thay `YOUR_DOMAIN`), `ALLOWED_ORIGIN` trong env BE.
