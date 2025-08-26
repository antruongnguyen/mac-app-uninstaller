# 🗑 App Uninstaller GUI (Rust + egui)

Ứng dụng GUI nhỏ gọn cho **macOS** cho phép:

- Liệt kê tất cả ứng dụng cài đặt trong `/Applications` và `~/Applications`.
- Phát hiện ứng dụng đang chạy (không thể gỡ).
- Xem và chọn các file/dữ liệu liên quan để xóa (LaunchAgents, Logs, Preferences, Receipts...).
- Chuyển ứng dụng và dữ liệu vào **Trash** thay vì xóa ngay lập tức.
- Giao diện giống **System Preferences** trên macOS, icon `.icns` native.

---

## 📦 Yêu cầu hệ thống

- **macOS 11.0 Big Sur** trở lên (khuyến nghị macOS 12+).
- **Rust toolchain** (Nightly hoặc Stable mới nhất).
- `cargo-bundle` để build `.app`:
  ```bash
  cargo install cargo-bundle
  ```

* **Xcode Command Line Tools** (để build cho macOS):

  ```bash
  xcode-select --install
  ```
* Font San Francisco (SF Pro Text) — macOS đã có sẵn.

---

## 📁 Cấu trúc thư mục

```
mac_uninstaller_gui/
 ├─ src/                  # Source code Rust
 ├─ resources/
 │   ├─ MyIcon.icns       # Icon macOS app
 │   └─ SF-Pro-Text-Regular.otf (tùy chọn)
 ├─ Cargo.toml
 └─ README.md
```

---

## 🚀 Cách chạy ứng dụng (Debug mode)

```bash
git clone https://github.com/yourname/mac_uninstaller_gui.git
cd mac_uninstaller_gui
cargo run
```

Ứng dụng sẽ chạy trong cửa sổ debug của egui.

---

## 🖥 Build ra file `.app` cho macOS

1. **Tạo icon `.icns`** nếu chưa có:

   ```bash
   mkdir MyIcon.iconset
   sips -z 16 16     icon.png --out MyIcon.iconset/icon_16x16.png
   sips -z 32 32     icon.png --out MyIcon.iconset/icon_16x16@2x.png
   sips -z 32 32     icon.png --out MyIcon.iconset/icon_32x32.png
   sips -z 64 64     icon.png --out MyIcon.iconset/icon_32x32@2x.png
   sips -z 128 128   icon.png --out MyIcon.iconset/icon_128x128.png
   sips -z 256 256   icon.png --out MyIcon.iconset/icon_128x128@2x.png
   sips -z 256 256   icon.png --out MyIcon.iconset/icon_256x256.png
   sips -z 512 512   icon.png --out MyIcon.iconset/icon_256x256@2x.png
   sips -z 512 512   icon.png --out MyIcon.iconset/icon_512x512.png
   cp icon.png MyIcon.iconset/icon_512x512@2x.png
   iconutil -c icns MyIcon.iconset
   mv MyIcon.icns resources/
   ```

2. **Build**:

   ```bash
   cargo bundle --release
   ```

3. File `.app` sẽ nằm ở:

   ```
   target/release/bundle/osx/App Uninstaller.app
   ```

---

## 🛠 Quyền truy cập

* Ứng dụng cần quyền **Full Disk Access** để xóa file trong một số thư mục hệ thống (`~/Library`,
  `/Library`).
* Cấp quyền trong:
  **System Preferences** → **Security & Privacy** → **Privacy** → **Full Disk Access** → thêm ứng
  dụng `.app`.

---

## 📜 License

MIT License — Bạn có thể sửa đổi và phân phối tự do.

---

## 💡 Lưu ý

* Ứng dụng chỉ xóa được app **không đang chạy**.
* Các file liên quan được tìm ở:

  * `~/Library/Preferences`
  * `~/Library/Application Support`
  * `~/Library/Caches`
  * `~/Library/Logs`
  * `~/Library/LaunchAgents`
  * `/Library/Preferences`
  * `/Library/Application Support`
  * `/private/var/db/receipts`
* Chế độ mặc định là **chuyển vào Trash** để tránh mất dữ liệu không mong muốn.
