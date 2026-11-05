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

Cập nhật 31/10/2026: bản Windows v1.1.0 đã được cài và dùng thử trên máy Windows thật, hoạt động bình thường. Mục Installed apps hiện đúng nhà phát hành "KeepAwake contributors". Linux vẫn chưa thử tay, nhất là việc gập máy trên GNOME.

## 11. Kế hoạch v1.2 (04/11/2026, đã sửa sau hai vòng review)

Mục tiêu: bốn nâng cấp nhỏ, ai dùng cũng thấy lợi, không làm app nặng thêm. Mỗi phần là một PR riêng, review và CI xong mới merge, cuối cùng phát hành v1.2.0.

Kế hoạch được review hai vòng:
- Vòng 1 do một agent độc lập làm, vì lúc đó Codex hết hạn mức. Agent kiểm tra với mã nguồn, `Cargo.lock`, mã của tauri-action và tài liệu chính thức. Những chỗ sửa theo vòng này ghi *(review)*.
- Vòng 2 do Codex làm cùng ngày, không thấy lỗi mức cao. Những chỗ sửa theo vòng này ghi *(Codex)*.

### 11.0 Sửa trước: tranh chấp giữa luồng hẹn giờ và thao tác của người dùng *(review)*

Luồng hẹn giờ (`lib.rs:460-466`) đọc `snapshot()` mà không giữ khóa `op`, rồi mới gọi `stop()`, và `stop()` không kiểm tra lại gì. Nếu người dùng bấm "+15 phút" (hoặc tắt rồi bật lại) đúng giữa hai bước đó, luồng hẹn giờ sẽ tắt nhầm phiên vừa được gia hạn hoặc vừa bật.

- Mỗi `Session` có số thứ tự `generation`, tăng mỗi lần `start()`.
- `generation` tăng dần suốt vòng đời app. Không bị đặt lại về 0 khi phiên về `Session::default()`, và `renew` giữ nguyên nó *(Codex)*.
- Thêm `stop_if(generation, điều kiện)`: giữ khóa `op`, đọc lại phiên, chỉ tắt khi vẫn đúng phiên đó và điều kiện vẫn đúng (`now >= ends_at` hoặc lý do pin). Luồng hẹn giờ và phần bảo vệ pin chỉ tắt qua hàm này.
- `stop_if` dùng phần tắt nội bộ, không gọi lại `stop()` vì `stop()` cũng lấy khóa `op` *(Codex)*.
- `stop_if` trả về việc nó có thật sự tắt hay không (`Option<Reason>`). Luồng hẹn giờ chỉ `publish` và gửi thông báo khi có tắt thật. Nếu không, phiên vừa được gia hạn vẫn chạy mà người dùng lại nhận thông báo "hết giờ" sai *(Codex)*.
- Có unit test tái hiện hai thứ tự. Cả hai trường hợp đều không được tắt và không được thông báo:
  - luồng hẹn giờ thấy hết giờ → người dùng gia hạn → lệnh tắt cũ chạy;
  - luồng hẹn giờ thấy hết giờ → người dùng tắt rồi bật phiên mới → lệnh tắt cũ chạy.

### 11.1 Thời gian còn lại trên thanh menu (macOS)

- Khi đang bật và có hẹn giờ, cạnh biểu tượng hiện thời gian còn lại dạng ngắn: `29m`, `1h05`. Làm tròn lên theo phút, nên còn 10 giây vẫn hiện `1m`. Trên 24 giờ vẫn hiện đúng, ví dụ `24h59`, vì kiểu "đến giờ" có thể dài khoảng 25 giờ vào đêm đổi giờ mùa đông. Không giới hạn hoặc đang tắt thì không hiện chữ.
- Dùng `TrayIcon::set_title`, chạy trên luồng chính (đã kiểm chứng trong mã tray-icon 0.25.1). Chỉ gọi khi chuỗi thay đổi.
- Chỉ macOS. Windows không hỗ trợ. Trên Linux, tài liệu Tauri khuyên không hiện chữ trừ khi người dùng yêu cầu.
- Cài đặt mới "Hiện thời gian trên thanh menu", mặc định bật, chỉ hiện trên macOS.
- Kiểm thử: unit test hàm định dạng với 10 giây, 59 giây, 1 phút, 59 phút, 1 giờ, 24 giờ 59 phút.

### 11.2 Nút "+15 phút"

- Menu khay luôn có mục "Thêm 15 phút". Mục này bị mờ khi đang tắt hoặc không giới hạn, vì menu của Tauri 2.12 không ẩn mục được *(review)*. Cửa sổ chính có nút "+15 phút" dưới dòng trạng thái, chỉ hiện khi đang hẹn giờ.
- Giữ khóa `op` và chỉ làm khi phiên đang bật và `now < ends_at`. Cộng 15 phút vào `ends_at_ms` và `total_ms`. Áp dụng cho cả kiểu "đến 18:30". Lựa chọn đã lưu không đổi.
- Thời gian còn lại tối đa 24 giờ. Nếu cộng đủ 15 phút sẽ vượt trần thì chỉ cộng đến đúng 24 giờ.
- **Không bao giờ rút ngắn phiên** *(Codex)*. Kiểu "đến giờ" vào đêm đổi giờ có thể còn khoảng 25 giờ (mục 11.1). Khi đó, kéo về 24 giờ sẽ làm mất gần một giờ, nên nút "+15" không làm gì.
- `total_ms` cộng đúng số phút thật sự được thêm, không mặc định cộng 15 phút *(Codex)*.
- Ô chọn "Đến 18:30" vẫn hiện lựa chọn đã lưu. Sau khi gia hạn, dòng chi tiết ghi giờ kết thúc thật, ví dụ "còn 44:58 · đến 18:45", để không bị nhầm *(review)*.
- *(review)* Plugin notification 2.8.0 **có** hỗ trợ nút trong thông báo trên máy tính (`on_action`, thêm từ bản 2.6.0). Nhưng lúc thông báo hết giờ hiện lên thì phiên đã tắt, nên nút đó phải bật một phiên 15 phút mới. Để v1.3, sau khi thử trên cả ba hệ điều hành.
- Kiểm thử: unit test hàm gia hạn với các trường hợp:
  - cộng bình thường;
  - cộng chạm trần 24 giờ;
  - đã còn trên 24 giờ, ví dụ đêm đổi giờ, thì không đổi;
  - không giới hạn;
  - đang tắt;
  - vừa hết giờ.
- Cửa sổ cố định 320×460 và không cuộn được. Phải kiểm tra màn chính và màn Cài đặt (nhất là tiếng Việt) vẫn vừa sau khi thêm nút và cài đặt mới. Nếu không vừa thì cho màn Cài đặt cuộn *(review)*.

### 11.3 Bảo vệ pin

- Cài đặt mới "Khi dùng pin": **Vẫn giữ thức** (mặc định) / **Tắt khi rút sạc** / **Tắt khi pin dưới 20%**. Lưu là `batteryGuard`: `off`, `unplugged`, `below20`. Bỏ mức 10% cho gọn *(review)*.
- Ngưỡng phần trăm **chỉ tính khi máy đang chạy pin**. Đang cắm sạc mà pin tụt (sạc yếu, sạc tối ưu) thì không tắt *(review)*.
- **Chỉ phản ứng khi tình trạng thay đổi trong lúc đang bật** (rút sạc, hoặc pin tụt qua 20% lúc đang chạy pin). Cụ thể *(review, Codex)*:
  - `start()` đọc pin ngay lúc bật để làm mẫu đầu tiên. Mẫu không mang sang từ phiên trước.
  - Mỗi lần đọc thành công, app so mẫu mới với **mẫu hợp lệ ngay trước nó**, rồi lưu mẫu mới thay mẫu cũ. Không so mãi với mẫu lúc bật. Nhờ vậy vẫn bắt được trường hợp bật khi đang chạy pin, cắm sạc, rồi rút ra; hoặc bật khi dưới 20%, sạc lên trên 20%, rồi tụt xuống lại *(Codex)*.
  - "Rút sạc" là mẫu trước đang cắm sạc và mẫu sau đang chạy pin.
  - "Dưới 20%" là mẫu sau đang chạy pin, dưới 20%, trong khi mẫu trước đang cắm sạc hoặc từ 20% trở lên. Như vậy ngưỡng được kích hoạt lại sau khi pin đã lên lại *(Codex)*.
  - Lần đọc lỗi không thay mẫu trước. Lần đọc thành công kế tiếp so với mẫu hợp lệ gần nhất *(Codex)*.
  - Kết quả đọc pin mang theo `generation` của phiên và số phiên bản cài đặt lúc bắt đầu đọc. Nếu đọc xong mà phiên đã đổi hoặc cài đặt `batteryGuard` đã đổi, kết quả bị bỏ *(Codex)*.
  - Mẫu trước nằm trong trạng thái của phiên. `renew` đang dựng lại `Session` (`lib.rs:56`), nên phải mang mẫu này theo *(Codex)*.
  - Đổi cài đặt `batteryGuard` giữa chừng thì đọc lại mẫu.
  - Người dùng tự bật khi đã ở dưới ngưỡng thì vẫn bật theo ý họ.
- Khi tự tắt:
  - Dừng phiên qua `stop_if` (mục 11.0) và cập nhật khay cùng cửa sổ.
  - `Session` và `Snapshot` có thêm `stopped_reason`, nên dù không có thông báo, cửa sổ vẫn ghi lý do, ví dụ "Đã tắt: máy đang dùng pin". Dòng này mất khi bật lại *(review)*.
  - Gửi thông báo nếu cài đặt thông báo đang bật. Nhãn cài đặt đổi thành "Thông báo khi KeepAwake tự tắt" để bao cả hai trường hợp.
- Phân biệt rõ ba trường hợp *(Codex)*:
  - **Chắc chắn không có pin:** cài đặt bị mờ, kèm dòng "Không tìm thấy pin".
  - **Không đọc được trạng thái pin:** cài đặt vẫn dùng được, kèm dòng "Không đọc được trạng thái pin".
  - **Có pin.**
- `Snapshot` có trường `battery: "present" | "absent" | "unknown"`. Trường này được đọc lúc khởi động và mỗi lần mở Cài đặt, không phụ thuộc việc app đang bật hay tắt *(review)*.
- App không bao giờ tự tắt vì lỗi đọc pin.
- Luồng hẹn giờ được viết lại để kiểm tra pin cả khi **không giới hạn thời gian**, vì đây là trường hợp dùng chính *(review)*. Cứ 30 giây đọc pin một lần, chỉ khi đang bật và cài đặt khác "Vẫn giữ thức". Đọc pin không giữ khóa nào, có giới hạn thời gian chờ, để không làm chậm đồng hồ đếm ngược.
- Module mới `power.rs` trả về `Result<Option<Power>, Error>` *(Codex)*:
  - `Ok(None)` là chắc chắn không có pin; `Err` là đọc lỗi.
  - `Power { on_battery: Option<bool>, percent: Option<u8> }`. Hai trường được biết riêng: Windows có thể biết đang cắm sạc mà không biết phần trăm.
  - Chế độ "Tắt khi rút sạc" vẫn chạy khi chỉ biết trạng thái sạc.
  - Không thêm crate mới *(review: đã kiểm chứng bằng `cargo tree`; Codex kiểm lại)*. Vẫn phải khai báo trực tiếp các crate theo từng hệ điều hành, và cập nhật `Cargo.lock` trong PR 2 *(Codex)*.
  - **macOS:** khai báo thẳng `objc2-io-kit 0.3.2` với tính năng `ps` và `objc2-core-foundation 0.3.2`. Hai crate này đã có trong cây qua `keepawake`. Dùng `IOPSCopyPowerSourcesInfo` + `IOPSCopyPowerSourcesList` + `IOPSGetPowerSourceDescription`.
  - **Windows:** dùng `windows 0.62.2`, đã có sẵn tính năng `Win32_System_Power`, để gọi `GetSystemPowerStatus`. Xử lý từng trường riêng:
    - `BatteryFlag = 128` là không có pin.
    - `ACLineStatus = 255` là không rõ trạng thái sạc.
    - `BatteryLifePercent = 255` là không rõ phần trăm.
  - **Linux:** UPower qua `zbus`, giữ một kết nối **system bus** và đặt thời gian chờ. `OnBattery` lấy từ `org.freedesktop.UPower`, phần trăm lấy từ `DisplayDevice`. Chỉ coi là có pin khi `IsPresent` đúng **và** `Type` là pin hoặc UPS. Máy bàn vẫn có UPower nên không thể coi "có UPower" là "có pin" *(review)*.
  - Bỏ phương án dự phòng `starship-battery` vì nó kéo thêm crate mới *(review)*.
- `settings.json`:
  - Hiện chỉ cần một giá trị lạ là cả file bị đọc lỗi và mọi cài đặt về mặc định (`settings.rs:86-87`).
  - Kiểu `batteryGuard` sẽ đọc giá trị không biết thành `off`, để bản cũ hơn hay file sửa tay không xóa cài đặt khác *(review)*.
  - Làm bằng `#[serde(other)]` trên biến thể `Off`, vì chỉ `#[serde(default)]` thì không đủ *(Codex)*.
  - Cách này chỉ cứu trường `batteryGuard`. Giá trị lỗi ở trường khác vẫn làm file về mặc định như hiện nay.
  - Cập nhật test `round_trip`. Thêm test đọc file cũ không có `batteryGuard`, và file có giá trị lạ *(Codex)*.
- Logic quyết định là hàm thuần `battery_should_stop(guard, previous, current) -> Option<Reason>`. Unit test cho các trường hợp *(Codex: thêm các chuỗi nhiều bước)*:
  - rút sạc;
  - cắm lại;
  - tụt qua 20% lúc chạy pin;
  - tụt khi đang sạc;
  - bật khi đã dưới ngưỡng;
  - bật lúc chạy pin → cắm sạc → rút ra;
  - dưới 20% → sạc lên trên 20% → tụt lại;
  - một lần đọc lỗi xen giữa;
  - chỉ biết trạng thái sạc mà không biết phần trăm;
  - không có pin;
  - không đọc được pin.

### 11.4 Bản cho chip ARM (Windows và Linux)

- **Thử trước trong CI** *(review)*. `ci.yml` thêm hai máy chạy thật bằng ARM, đều miễn phí cho repo công khai: `windows-11-arm` và `ubuntu-22.04-arm`. Cả hai chạy đủ format, clippy, test, build và chạy thử app. Như vậy lỗi ARM lộ ra ngay ở PR, không đợi tới ngày phát hành. Ví dụ, `ring` cần `clang` khi build cho Windows ARM.
- `release.yml`:
  - Build Windows ARM64 trên máy `windows-11-arm`. Chỉ ra bộ cài NSIS. Đây là chọn phạm vi cho gọn: bundler của Tauri CLI 2.12.4 có hỗ trợ MSI ARM64 trong mã nguồn, chỉ là chưa ai thử *(Codex)*. Dự phòng: build chéo trên `windows-latest` với `--target aarch64-pc-windows-msvc`.
  - Build Linux ARM64 trên máy `ubuntu-22.04-arm`, ra AppImage, `.deb` và `.rpm`.
  - Mỗi bản build có khóa cache Rust riêng (`key: ${{ matrix.name }}`).
  - Bảng tải về trong ghi chú phát hành thêm các file ARM.
- **Phát hành thử an toàn** *(review, Codex)*:
  - Hiện mọi tag đều được đăng bằng `--latest` (`release.yml:108`), và bản đã cài lấy `latest.json` từ bản latest.
  - Tách job `publish` làm hai *(Codex)*. Bước kiểm tra `latest.json` chạy với **mọi** tag. Chỉ bước đăng bản có điều kiện `!contains(github.ref_name, '-')`. Nếu bỏ qua cả job thì bản rc cũng bỏ qua luôn bước kiểm tra, và mất ý nghĩa thử.
  - Bản thử như `v1.2.0-rc.1` vì vậy chỉ là bản nháp, không tới tay người dùng.
  - Thay đổi này nằm trong PR 3 và phải merge trước khi đẩy tag rc.
  - Ra `v1.2.0-rc.1` trước để kiểm tra đủ file và chữ ký, rồi mới ra `v1.2.0`.
- **`latest.json` chỉ có một nơi ghi** *(Codex)*:
  - tauri-action đọc `latest.json` cũ, xóa, rồi tải bản mới lên. 6 bản build chạy song song nên có thể đè lên nhau.
  - Các job build đặt `uploadUpdaterJson: false`. Một job cuối, chạy sau khi mọi bản build xong, tự dựng `latest.json` từ các file `.sig` đã tải lên rồi mới tải nó lên. Dự phòng nếu cách này rắc rối: giữ tauri-action ghi, nhưng thêm bước kiểm tra dưới đây và chạy lại job bị thiếu.
  - Khóa nền tảng: `darwin-aarch64`, `darwin-x86_64`, `linux-x86_64`, `windows-x86_64`, `windows-aarch64`, `linux-aarch64`. Tên file và khóa đã đối chiếu với mã nguồn tauri-action (`build.ts`, `upload-version-json.ts`):
    - `KeepAwake_x_arm64-setup.exe`
    - `_aarch64.AppImage`
    - `_arm64.deb`
    - `-1.aarch64.rpm`
- **Bước kiểm tra `latest.json` chặt hơn** *(Codex)*. Hiện bước này chỉ xem chữ ký có rỗng không. Bản mới kiểm tra:
  - `version` đúng với tag;
  - đủ 6 khóa nền tảng;
  - mỗi `url` trỏ tới đúng file có trong bản phát hành, đúng kiến trúc;
  - mỗi `signature` trùng với nội dung file `.sig` đi kèm.
- **Trang web và README:**
  - Bảng tải về thêm các file ARM. Phần Cài đặt trên trang web thêm link ARM.
  - Sửa chữ "x86_64" trong `site/main.js`.
  - Nút tải chính vẫn là bản x64, vì trình duyệt trên Windows ARM thường tự báo là x64.
  - Regex hiện tại không khớp tên file ARM, nên không bị lấy nhầm *(đã kiểm chứng)*.
  - Lưu ý: bản x64 đang chạy giả lập trên máy ARM sẽ tiếp tục nhận bản cập nhật x64.
- **Kênh phân phối:**
  - winget thêm installer arm64, khai báo kiến trúc rõ ràng (`<url>|arm64`), vì bộ cài NSIS của bản ARM là file x86.
  - Homebrew không đổi.
  - **Bỏ AUR aarch64**, vì AUR chính thức chỉ có x86_64 *(review)*.

### 11.5 Gập máy trên Linux (chỉ làm nếu cần)

Chờ bạn thử. `LidSwitchIgnoreInhibited=yes` là mặc định của logind. Nhưng KDE PowerDevil, hoặc GNOME khi cắm màn hình ngoài, tự giữ quyền xử lý việc gập máy, nên mặc định đó chưa đủ để kết luận *(review)*. Nếu máy vẫn thức khi gập:
- Đo trên cả GNOME và KDE bằng `systemd-inhibit --list` trước khi sửa.
- **Không** bỏ thẳng cờ `FLAG_SUSPEND`, vì có thể làm KDE tự ngủ dù app đang bật.

### 11.6 Thứ tự và phát hành

1. **PR 1:** mục 11.0, 11.1 và 11.2, kèm kiểm tra cửa sổ vẫn vừa.
2. **PR 2:** mục 11.3, kèm cập nhật `Cargo.lock`. Nếu phần Linux (UPower) gặp khó khi thử, tách ra làm sau, macOS và Windows ra trước *(review)*.
3. **PR 3:** mục 11.4, gồm việc tách bước đăng bản. Phải merge trước khi đẩy tag rc *(Codex)*.
4. **Mỗi PR:** review, sửa hết lỗi, CI đạt, bạn đồng ý mới merge. Thử tay trên macOS. Phần pin cần bạn rút sạc MacBook để thử.
5. **Phát hành:**
   - Tăng phiên bản ở `package.json`, `Cargo.toml` và `tauri.conf.json`. Cập nhật luôn `Cargo.lock` và `package-lock.json`, vì CI dùng `--locked` *(Codex)*. Viết CHANGELOG.
   - Ra `v1.2.0-rc.1` để kiểm tra đủ 6 nền tảng có chữ ký trong `latest.json`.
   - Ra `v1.2.0`.
   - Cập nhật tap Homebrew, winget (hai link x64 và arm64) và PKGBUILD AUR (vẫn chỉ x86_64).
   - Cập nhật danh sách tính năng trên trang web.

### 11.7 Không làm trong v1.2

- Nút "+15 phút" trong thông báo hết giờ: làm được nhưng phải bật phiên mới, để v1.3.
- Chữ trên khay ở Linux và Windows.
- Bản AUR cho aarch64.
- Giữ thức theo app đang chạy, điều khiển bằng link hoặc dòng lệnh, lịch cố định: để v1.3.
