# KeepAwake – Kế hoạch dự án

> Ứng dụng desktop siêu nhỏ, một việc duy nhất: **giữ máy tính không ngủ** (không sleep, tùy chọn không tắt màn hình).
> Chạy trên macOS / Windows / Linux, cài đặt bằng một file, nằm ở khay hệ thống (tray).

---

## 1. Mục tiêu

| Tiêu chí | Mục tiêu |
|---|---|
| Đơn giản | Mở app → bấm 1 nút → máy không ngủ. Không cần đọc hướng dẫn. |
| Nhẹ | Bộ cài < 10 MB, RAM khi chạy < 40 MB (nếu dùng Tauri). |
| Cài dễ | macOS: `.dmg` kéo thả. Windows: `.exe` / `.msi` next‑next‑finish. Linux: `.AppImage` / `.deb`. |
| Không phụ thuộc | Không cần cài runtime, không cần quyền admin (trừ Windows installer). |
| Mã nguồn mở | Repo public trên GitHub, phát hành qua GitHub Releases, build tự động bằng GitHub Actions. |

---

## 2. Lựa chọn công nghệ (cần bạn duyệt)

### Phương án A – Tauri 2 + Rust (khuyến nghị)

| | |
|---|---|
| Backend | Rust, dùng crate `keepawake` (gọi đúng API hệ điều hành) |
| Frontend | HTML/CSS/JS thuần, không framework → bundle rất nhỏ |
| Tray | Tauri 2 có sẵn tray icon + menu |
| Khởi động cùng máy | `tauri-plugin-autostart` |
| Lưu cài đặt | `tauri-plugin-store` (file JSON) |
| Kích thước | ~5–8 MB bộ cài, ~25–40 MB RAM |
| Nhược điểm | Máy bạn **chưa có Rust** → tôi sẽ cài `rustup` (1 lệnh, không cần admin). Build lần đầu ~3–5 phút. |

Cách giữ máy thức theo từng hệ điều hành (crate `keepawake` xử lý sẵn):

- **macOS**: `IOPMAssertionCreateWithName` (giống `caffeinate`)
- **Windows**: `SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)`
- **Linux**: D‑Bus `org.freedesktop.ScreenSaver.Inhibit` / `org.freedesktop.PowerManagement` / systemd‑logind

### Phương án B – Electron

| | |
|---|---|
| Ưu | Chỉ cần Node (đã có). API `powerSaveBlocker` có sẵn, code backend ~20 dòng. |
| Nhược | Bộ cài 80–120 MB, RAM 100–150 MB cho một app chỉ làm một việc. |

**Đề xuất: Phương án A.** Nếu bạn không muốn cài Rust lên máy, chọn B.

---

## 3. Danh sách chức năng

### v1.0 (MVP – làm ngay)

| # | Chức năng | Mô tả |
|---|---|---|
| F1 | **Bật / tắt giữ thức** | Một nút lớn, một cú click. Trạng thái hiện rõ bằng màu + icon tray. |
| F2 | **Hẹn giờ** | Chọn nhanh: 15 phút · 30 phút · 1 giờ · 2 giờ · 4 giờ · Vô thời hạn · Tùy chỉnh. Hết giờ tự tắt. |
| F3 | **Chế độ** | `Giữ cả màn hình` (không tắt màn hình) hoặc `Chỉ giữ hệ thống` (màn hình được phép tắt, máy vẫn chạy). |
| F4 | **Tray icon + menu** | Icon đổi màu theo trạng thái. Menu: Bật/Tắt, chọn thời gian nhanh, Mở cửa sổ, Cài đặt, Thoát. |
| F5 | **Đếm ngược** | Hiển thị thời gian còn lại trong cửa sổ và tooltip tray. |
| F6 | **Khởi động cùng hệ thống** | Bật/tắt trong Cài đặt. |
| F7 | **Tự bật khi mở app** | Tùy chọn: mở app là giữ thức luôn, không cần bấm. |
| F8 | **Đóng cửa sổ = ẩn xuống tray** | App không thoát khi đóng cửa sổ, chỉ thoát qua menu tray. |
| F9 | **Nhớ cài đặt** | Lưu chế độ, thời gian mặc định, tùy chọn khởi động. |
| F10 | **Chế độ sáng / tối** | Theo hệ thống. |

### v1.1 (sau khi v1.0 ổn)

| # | Chức năng |
|---|---|
| F11 | Phím tắt toàn cục bật/tắt (ví dụ `Ctrl/Cmd + Shift + K`) |
| F12 | Tự tắt khi pin dưới X% (laptop) |
| F13 | Thông báo khi hết giờ |
| F14 | Tự cập nhật phiên bản (tauri‑plugin‑updater) |
| F15 | Đa ngôn ngữ (Tiếng Việt / English) |

### Không làm (giữ app đơn giản)

- Lịch theo ngày/giờ, tích hợp đám mây, thống kê, tài khoản người dùng.

---

## 4. Giao diện

Mockup có thể xem trực tiếp: `docs/ui-mockup.html` (mở bằng trình duyệt).

### 4.1 Cửa sổ chính (320 × 420, không resize)

```
┌─────────────────────────────────┐
│  ☕ KeepAwake              ⚙  ─ ✕ │
├─────────────────────────────────┤
│                                 │
│            ╭───────╮            │
│            │  ☕   │   ← nút lớn │
│            │       │     bật/tắt│
│            ╰───────╯            │
│                                 │
│        ĐANG GIỮ THỨC            │
│        còn 1:29:58              │
│                                 │
│  Thời gian                      │
│  [15p] [30p] [1h] [2h] [4h]     │
│  [∞ Vô hạn] [⏱ Tùy chỉnh]       │
│                                 │
│  Chế độ                         │
│  (●) Giữ cả màn hình            │
│  ( ) Chỉ giữ hệ thống           │
│                                 │
├─────────────────────────────────┤
│  v1.0.0          Ẩn xuống tray  │
└─────────────────────────────────┘
```

- Nút tròn lớn: **xám** = tắt, **xanh lá / cam** = đang giữ thức, có vòng tiến trình đếm ngược quanh nút.
- Click chip thời gian khi đang bật → đặt lại bộ đếm ngay.

### 4.2 Menu tray

```
 ☕ KeepAwake — đang bật (1:29:58)
 ─────────────────────────────
 ✓ Giữ thức
 ─────────────────────────────
   15 phút
   30 phút
   1 giờ
   2 giờ
   Vô thời hạn
 ─────────────────────────────
   Mở cửa sổ
   Cài đặt…
 ─────────────────────────────
   Thoát
```

Icon tray: ☕ xám (tắt) · ☕ màu (bật). Click trái = bật/tắt nhanh, click phải = menu.

### 4.3 Cài đặt (sheet nhỏ trong cùng cửa sổ)

```
 Cài đặt
 ───────────────────────────────
 [x] Khởi động cùng hệ thống
 [ ] Tự giữ thức khi mở app
 [x] Đóng cửa sổ → ẩn xuống tray
 Thời gian mặc định   [ 1 giờ ▾ ]
 Chế độ mặc định      [ Giữ cả màn hình ▾ ]
 ───────────────────────────────
 Phiên bản 1.0.0 · Kiểm tra cập nhật · GitHub
```

---

## 5. Cấu trúc repo

```
keepawake/
├── .github/workflows/
│   ├── ci.yml            # lint + build khi push / PR
│   └── release.yml       # build 3 OS + tạo GitHub Release khi push tag v*
├── docs/
│   ├── PLAN.md
│   └── ui-mockup.html
├── src/                  # frontend (HTML/CSS/JS thuần)
│   ├── index.html
│   ├── style.css
│   └── main.js
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── awake.rs      # bọc crate keepawake
│       ├── tray.rs       # tray icon + menu
│       └── settings.rs   # đọc/ghi store
├── package.json
├── README.md             # hướng dẫn cài đặt (có ảnh), tải về
├── LICENSE               # MIT
└── CHANGELOG.md
```

---

## 6. Đóng gói & phát hành

| OS | Định dạng | Ghi chú cài đặt |
|---|---|---|
| macOS | `.dmg` (Apple Silicon + Intel, universal) | Chưa ký Apple Developer → lần đầu mở: chuột phải → Open, hoặc `xattr -cr /Applications/KeepAwake.app`. Ghi rõ trong README. |
| Windows | `.exe` (NSIS) + `.msi` | SmartScreen sẽ cảnh báo vì chưa ký → "More info → Run anyway". Ghi rõ trong README. |
| Linux | `.AppImage` + `.deb` (+ `.rpm`) | AppImage: `chmod +x` rồi chạy. |

Quy trình phát hành:

1. `git tag v1.0.0 && git push --tags`
2. GitHub Actions (`tauri-apps/tauri-action`) build song song trên `macos-latest`, `windows-latest`, `ubuntu-22.04`.
3. Tự tạo GitHub Release, đính kèm toàn bộ file cài.
4. README có bảng link tải theo OS.

---

## 7. Các bước tôi sẽ thực hiện sau khi bạn duyệt

| Bước | Việc | Thời gian ước tính |
|---|---|---|
| 1 | Cài Rust (`rustup`) trên máy bạn, kiểm tra `cargo tauri` | 5 phút |
| 2 | Khởi tạo project Tauri 2, cấu trúc như mục 5, LICENSE MIT | 10 phút |
| 3 | Backend: giữ thức + hẹn giờ + tray + autostart + store | 30–45 phút |
| 4 | Frontend: cửa sổ chính + cài đặt theo mockup, sáng/tối | 30 phút |
| 5 | Build & chạy thử trên macOS của bạn (`.dmg`) | 10 phút |
| 6 | GitHub Actions CI + release workflow | 15 phút |
| 7 | README (Tiếng Việt + English), ảnh chụp màn hình | 15 phút |
| 8 | Tạo repo public `kingleykin/keepawake`, push, tag `v1.0.0`, chờ CI build 3 OS, kiểm tra Release | 20 phút + thời gian CI |

---

## 8. Cần bạn quyết định

1. **Công nghệ**: Tauri (A, khuyến nghị – tôi cần cài Rust lên máy bạn) hay Electron (B)?
2. **Tên app / tên repo**: `KeepAwake` (repo `kingleykin/keepawake`)? Hay tên khác?
3. **Giấy phép**: MIT (mặc định)?
4. **Ngôn ngữ giao diện v1.0**: Tiếng Việt, English, hay cả hai (theo hệ thống)?
5. **Phạm vi v1.0**: giữ nguyên F1–F10 ở trên, hay cắt/thêm gì?

---

## 9. Điều chỉnh sau review của Codex (20/10/2026)

Codex (GPT) đã review bản kế hoạch và mockup. Các điểm tiếp thu và cách xử lý:

| # | Phát hiện | Điều chỉnh trong kế hoạch |
|---|---|---|
| 1 | Lời hứa "không ngủ" quá rộng. Windows không thể chặn người dùng chủ động Sleep / gập máy; cờ `ES_DISPLAY_REQUIRED` không được dùng ở chế độ "chỉ giữ hệ thống". `IOPMAssertionCreateWithName` đã bị Apple đánh dấu deprecated (vẫn hoạt động). | Định nghĩa rõ v1.0 = **chặn ngủ do không hoạt động (idle sleep)**. Chế độ "Chỉ giữ hệ thống" không bật cờ display. Chỉ hiện "ĐANG GIỮ THỨC" **sau khi** hệ điều hành chấp nhận assertion; nếu lỗi thì hiện cảnh báo đỏ. |
| 2 | Linux/Wayland: idle‑inhibit của Wayland gắn với cửa sổ đang hiển thị, app ẩn ở tray có thể không có tác dụng. Cần thứ tự fallback rõ ràng. | Thứ tự: `org.freedesktop.ScreenSaver` → `org.freedesktop.PowerManagement` → systemd‑logind. Nếu tất cả thất bại → báo lỗi, không hiện trạng thái giả. Thêm mục test GNOME + KDE trên cả Wayland và X11 vào checklist phát hành. |
| 3 | Hướng dẫn cài bản chưa ký đang xem nhẹ ma sát lần đầu (Gatekeeper, SmartScreen, Smart App Control). | v1.0 vẫn **không ký** (không tốn phí), nhưng README ghi rõ "bản chưa ký", kèm ảnh từng bước mở lần đầu, và test trên máy sạch. Ký Apple Developer ID / Windows code‑sign đưa vào v1.2 nếu bạn muốn đầu tư. |
| 4 | "Universal dmg" chưa khai báo target build. | Workflow dùng target `universal-apple-darwin` (cài cả 2 Rust target), `permissions: contents: write`, cài đủ dependency WebKitGTK cho runner Ubuntu. |
| 5 | "Không cần runtime" không đúng tuyệt đối: Tauri dùng webview hệ thống (WebKitGTK trên Linux, WebView2 trên Windows). | Sửa mục tiêu thành: **không cần cài thêm gì trên macOS/Windows 10+** (WebView2 có sẵn trên Win 10/11 cập nhật; installer tự tải nếu thiếu). Linux: ghi rõ yêu cầu `webkit2gtk-4.1`, hỗ trợ Ubuntu 22.04+ / Fedora 38+. |
| 6 | Tooltip tray và click‑trái‑bật/tắt **không được hỗ trợ trên Linux** trong Tauri. | Đếm ngược hiển thị trong cửa sổ và dòng đầu menu tray (không dựa vào tooltip). Click trái: macOS/Windows = bật/tắt nhanh; Linux = mở menu. |
| 7 | Mockup có thể không vừa 320×420; dùng nút traffic‑light của macOS cho app đa nền tảng. | Dùng **title bar gốc** của từng OS, không vẽ title bar riêng. Cửa sổ 320×460, không resize, kiểm tra lại khi code thật. |
| 8 | v1.0 hơi nhiều; mockup lộ "Ngôn ngữ" và "Kiểm tra cập nhật" là tính năng v1.1. | **Cắt khỏi v1.0**: F7 (tự bật khi mở app), ô "Ngôn ngữ", "Kiểm tra cập nhật", "Thời gian tùy chỉnh". v1.0 giữ: F1 bật/tắt, F2 hẹn giờ (chip cố định + vô hạn), F3 chế độ, F4 tray, F5 đếm ngược, F6 khởi động cùng máy, F8 ẩn xuống tray, F9 nhớ cài đặt, F10 sáng/tối. Đảm bảo **thoát app / hết giờ luôn nhả assertion**. |

### v1.0 sau điều chỉnh

- Bật/tắt · Hẹn giờ (15p/30p/1h/2h/4h/∞) · Chế độ màn hình/hệ thống · Tray + menu · Đếm ngược · Khởi động cùng máy · Ẩn xuống tray · Nhớ cài đặt · Sáng/tối.
- Trạng thái "đang giữ thức" chỉ hiện khi OS xác nhận; có trạng thái lỗi.
- Title bar gốc của OS; cửa sổ 320×460.

### v1.1

- Thời gian tùy chỉnh, tự bật khi mở app, phím tắt, thông báo hết giờ, ngôn ngữ, tự cập nhật, tắt khi pin yếu.

---

## 10. Ghi chú triển khai v1.0.0 (20/10/2026)

Những điểm khác so với kế hoạch ban đầu, quyết định trong lúc code:

| Chủ đề | Thực tế |
|---|---|
| Ngôn ngữ | Tiếng Anh (mặc định) và tiếng Việt, đổi trong Cài đặt. Menu tray cũng đổi theo. |
| Luồng giữ thức | Mọi yêu cầu tới hệ điều hành chạy trên **một luồng riêng**, vì `SetThreadExecutionState` của Windows gắn với luồng gọi nó. |
| Linux | Thử **XDG desktop portal** (`org.freedesktop.portal.Inhibit`) trước, sau đó mới tới `org.freedesktop.ScreenSaver` + systemd‑logind (crate `keepawake`). Lỗi khi nhả khóa không làm sập app. |
| Cài đặt | Bỏ "Thời gian mặc định" và "Chế độ mặc định" khỏi màn Cài đặt. Lựa chọn thời gian và chế độ ở màn chính được nhớ tự động. Lưu bằng file JSON tự viết, không dùng `tauri-plugin-store`. |
| Một phiên bản | Thêm `tauri-plugin-single-instance`: mở app lần hai chỉ hiện lại cửa sổ đang chạy. |
| macOS | Khi ẩn xuống tray, app ẩn luôn icon ở Dock. |
| Windows | Bộ cài NSIS cài cho người dùng hiện tại, không cần quyền admin, có tiếng Anh và tiếng Việt. |
| Giao diện | Title bar gốc của hệ điều hành, cửa sổ 320×460. Ảnh thật nằm ở `docs/screenshots/`. `ui-mockup.html` là bản thiết kế trước review. |

Đã kiểm tra trên macOS (Apple Silicon): bật/tắt, đổi chế độ, đổi thời gian, đổi ngôn ngữ và tự tắt khi hết giờ, đối chiếu với `pmset -g assertions`. Windows và Linux được build và kiểm tra tự động trên GitHub Actions, chưa thử tay trên máy thật.
