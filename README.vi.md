<p align="center">
  <img src="assets/icon.png" width="96" alt="Biểu tượng KeepAwake">
</p>

<h1 align="center">KeepAwake</h1>

<p align="center">
  Giữ máy tính luôn thức. Một cú bấm, từ cửa sổ nhỏ hoặc khay hệ thống.<br>
  macOS · Windows · Linux
</p>

<p align="center"><a href="README.md">English</a></p>

<p align="center">
  <img src="docs/screenshots/on-dark.png" width="240" alt="KeepAwake đang giữ thức, giao diện tối">
  <img src="docs/screenshots/off-light.png" width="240" alt="KeepAwake đang tắt, giao diện sáng">
  <img src="docs/screenshots/settings-vi.png" width="240" alt="Màn cài đặt tiếng Việt">
</p>

## Chức năng

- **Một cú bấm.** Bấm nút tròn lớn, hoặc click trái vào icon trên khay (macOS và Windows).
- **Hẹn giờ.** 15 phút, 30 phút, 1 giờ, 2 giờ, 4 giờ, hoặc vô hạn. Hết giờ tự tắt.
- **Hai chế độ.** Giữ cả màn hình, hoặc chỉ giữ hệ thống và cho phép màn hình tắt.
- **Nằm ở khay hệ thống.** Menu hiện thời gian còn lại. Chọn một mốc thời gian trong menu sẽ bật luôn.
- **Khởi động cùng hệ thống.** Chạy ngầm ở khay, không mở cửa sổ.
- **Tiếng Anh và tiếng Việt.** Mặc định tiếng Anh, đổi trong Cài đặt.
- **Nhẹ.** Viết bằng [Tauri](https://tauri.app) và Rust, bộ cài vài MB.
- **Trạng thái trung thực.** Chỉ hiện "Đang giữ thức" khi hệ điều hành đã chấp nhận. Nếu bị từ chối, app hiện lỗi.

## Cài đặt

Tải file theo hệ điều hành ở **[bản phát hành mới nhất](https://github.com/kingleykin/keepawake/releases/latest)**.

| Hệ điều hành | File |
|---|---|
| macOS 10.15+ (Apple Silicon và Intel) | `KeepAwake_x.y.z_universal.dmg` |
| Windows 10 / 11 | `KeepAwake_x.y.z_x64-setup.exe` (khuyên dùng) hoặc `.msi` |
| Linux, mọi bản phân phối | `KeepAwake_x.y.z_amd64.AppImage` |
| Ubuntu / Debian | `KeepAwake_x.y.z_amd64.deb` |
| Fedora / openSUSE | `KeepAwake-x.y.z-1.x86_64.rpm` |

> Bản cài chưa được ký số vì chứng chỉ ký tốn phí. Lần mở đầu tiên hệ điều hành sẽ cảnh báo. Chỉ cần làm các bước dưới đây một lần.

### macOS

1. Mở file `.dmg`, kéo **KeepAwake** vào **Applications**.
2. Mở KeepAwake. macOS báo không xác minh được nhà phát triển. Bấm **Done**.
3. Mở **System Settings → Privacy & Security**, kéo xuống, bấm **Open Anyway** cạnh KeepAwake.

Hoặc chạy lệnh này một lần trong Terminal thay cho bước 2 và 3:

```bash
xattr -dr com.apple.quarantine /Applications/KeepAwake.app
```

### Windows

1. Chạy `KeepAwake_x.y.z_x64-setup.exe`. Bộ cài chỉ cài cho tài khoản của bạn, không cần quyền admin.
2. Nếu SmartScreen báo "Windows protected your PC", bấm **More info → Run anyway**.

Máy công ty hoặc máy bật Smart App Control có thể không có nút "Run anyway". Khi đó hãy hỏi quản trị viên, hoặc tự build từ mã nguồn.

### Linux

AppImage:

```bash
chmod +x KeepAwake_*_amd64.AppImage
./KeepAwake_*_amd64.AppImage
```

Debian hoặc Ubuntu:

```bash
sudo apt install ./KeepAwake_*_amd64.deb
```

Fedora:

```bash
sudo dnf install ./KeepAwake-*.x86_64.rpm
```

GNOME gốc ẩn icon khay. Cài [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/) để thấy icon KeepAwake. Ubuntu đã có sẵn.

## "Giữ thức" nghĩa là gì

KeepAwake chặn máy tự ngủ **khi không có ai dùng**. Ở chế độ "Giữ cả màn hình", màn hình cũng không tự tắt.

App **không** chặn việc bạn chủ động cho máy ngủ, ví dụ gập laptop hoặc chọn Sleep trong menu.

| Hệ điều hành | Cách hoạt động | Kiểm tra |
|---|---|---|
| macOS | IOKit power assertion `PreventUserIdleSystemSleep` và `PreventUserIdleDisplaySleep` | `pmset -g assertions` |
| Windows | `SetThreadExecutionState` với `ES_SYSTEM_REQUIRED`, thêm `ES_DISPLAY_REQUIRED` ở chế độ màn hình | `powercfg /requests` |
| Linux | Thử XDG desktop portal `Inhibit` trước, sau đó `org.freedesktop.ScreenSaver` và systemd-logind | `systemd-inhibit --list` |

**Lưu ý trên Linux.** GNOME và KDE hoạt động trên cả Wayland và X11 qua desktop portal. Một số compositor Wayland tối giản như Sway, Hyprland có thể bỏ qua yêu cầu này. Nếu không có dịch vụ nào chấp nhận, app hiện lỗi thay vì giả vờ đang chạy. Trên Linux, click trái vào icon khay sẽ mở menu.

## Build từ mã nguồn

Cần [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 18 trở lên, và [các gói Tauri yêu cầu](https://v2.tauri.app/start/prerequisites/) cho hệ điều hành của bạn.

```bash
git clone https://github.com/kingleykin/keepawake.git
cd keepawake
npm install
npm run dev
```

Tạo bộ cài cho hệ điều hành đang dùng:

```bash
npm run build
```

Chi tiết cấu trúc dự án và quy trình phát hành xem trong [README tiếng Anh](README.md#build-from-source).

## Đóng góp

Rất hoan nghênh báo lỗi và pull request. Xin giữ app đơn giản: làm một việc và làm tốt. Khi báo lỗi trên Linux, hãy ghi rõ bản phân phối, môi trường desktop, và đang dùng Wayland hay X11.

## Giấy phép

[MIT](LICENSE). Tự do sử dụng, chỉnh sửa và chia sẻ.
