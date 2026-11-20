# Database

Firebase Realtime Database là nguồn dữ liệu duy nhất trên cloud. Backend là client hợp pháp, FE không đụng trực tiếp DB.

## Cấu trúc node

```
responses/{uuid}
  id: string
  consent: "Vâng" | "Không Nha"
  free_day: "YYYY-MM-DD" | null
  food: string | null
  drink: string | null
  place: string | null
  created_at: RFC3339
```

Backend ghi qua `PUT /responses/{id}.json`, đọc/xóa qua REST. Không dùng SDK trên FE.

## Rules mẫu

Giới hạn quyền theo môi trường. Với backend chạy server-side không có auth Firebase, node `responses` cần cho phép ghi từ server; tuyệt đối không để test mode public lâu dài. Khi có service account, chuyển sang Admin SDK và khóa Rules ở chế độ deny-all cho client.

## Backup

Chạy từ `backend/`:

```
cargo run --release -- --backup > ../database/backup.json
```

Dữ liệu trả về dạng JSON, có thể đưa vào cron/CI để export định kỳ.
