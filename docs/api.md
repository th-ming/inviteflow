# API

Base URL là domain Leapcell của service. FE khai `window.API_BASE` trong mỗi trang.

| Method | Path | Auth | Mô tả |
|---|---|---|---|
| GET | `/api/health` | không | Health check, dùng cho keep-alive |
| POST | `/api/responses` | không | Tạo một lượt trả lời |
| GET | `/api/responses` | Bearer | Liệt kê toàn bộ |
| DELETE | `/api/responses/{id}` | Bearer | Xóa một bản ghi |
| DELETE | `/api/responses` | Bearer | Xóa toàn bộ |

## Body POST

```json
{
  "consent": "Vâng",
  "free_day": "2026-11-28",
  "food": "Cơm",
  "drink": "Trà Chanh",
  "place": "Công Viên"
}
```

`consent` bắt buộc, chỉ nhận `Vâng` hoặc `Không Nha`. Các field còn lại tùy chọn, tối đa 100 ký tự. `free_day` phải `YYYY-MM-DD`.

## Lỗi

```json
{ "error": "consent phải là Vâng hoặc Không Nha" }
```

Status: 400 (validation), 401 (thiếu/sai token), 429 (quá giới hạn), 500 (DB lỗi).

## Rate limit

Mặc định 10 request/phút mỗi IP, cấu hình qua `RATE_LIMIT` và `RATE_WINDOW_MS`.
