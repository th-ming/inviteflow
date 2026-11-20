# Kiến trúc

```
Trình duyệt (FE tĩnh)
  └── assets/js/api.js ──HTTP──> Leapcell (Rust API)
                                   ├── config.rs   env + validate token
                                   ├── validate.rs payload -> ResponseRecord
                                   ├── db.rs       Firebase REST adapter
                                   └── api.rs      routes + CORS + rate limit
                                          │
                                          ▼
                              Firebase Realtime Database
                                   node: responses/{uuid}
```

FE không chứa config DB hay token quản lý. `ADMIN_TOKEN` chỉ nằm trong env của BE; admin page hỏi token và lưu trong `sessionStorage`.

## Quyết định thiết kế

- **Tách lớp**: FE gọi API, BE làm việc với DB. Không có driver file trong cấu hình mặc định (`DB_DRIVER=firebase`).
- **Không build FE**: HTML/CSS/JS module thuần, Firebase Hosting serve trực tiếp.
- **Server-side validation**: mọi payload đi qua `validate.rs` trước khi ghi DB.
- **Auth quản lý**: Bearer token so sánh hằng thời gian, chỉ áp dụng cho GET/DELETE.
- **Rate limit**: in-memory theo IP, chặn spam POST.

## Luồng dữ liệu

```
index.html -> thanks -> date -> food -> drink -> place -> done
   |                                                     |
   +-- POST "Không Nha" (lần đầu)                        +-- POST payload đầy đủ
                                                          |
                                                    admin.html (GET/DELETE, token)
```
