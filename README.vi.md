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
  <img src="docs/demo.gif" width="640" alt="Chọn 30 phút, bật KeepAwake, rồi đặt giữ thức đến 18:30. Icon trên khay chuyển sang màu xanh khi đang bật.">
</p>

<details>
<summary>Thêm ảnh chụp màn hình</summary>
<p align="center">
  <img src="docs/screenshots/on-dark.png" width="240" alt="KeepAwake đang giữ thức, giao diện tối">
  <img src="docs/screenshots/off-light.png" width="240" alt="KeepAwake đang tắt, giao diện sáng">
  <img src="docs/screenshots/settings-vi.png" width="240" alt="Màn cài đặt tiếng Việt">
</p>
</details>

## Chức năng

- **Một cú bấm.** Bấm nút tròn lớn, hoặc click trái vào icon trên khay (macOS và Windows).
- **Hẹn giờ.** 15 phút, 30 phút, 1 giờ, 2 giờ, 4 giờ, vô hạn, hoặc tùy chỉnh: số phút bất kỳ, hoặc "đến 18:30". Hết giờ tự tắt và báo bằng thông báo.
- **Phím tắt.** ⌥⌘K trên macOS, Ctrl+Alt+K trên Windows và Linux, dùng được từ mọi ứng dụng. Mặc định tắt, bật trong Cài đặt.
- **Hai chế độ.** Giữ cả màn hình, hoặc chỉ giữ hệ thống và cho phép màn hình tắt.
- **Nằm ở khay hệ thống.** Menu hiện thời gian còn lại. Chọn một mốc thời gian trong menu sẽ bật luôn.
- **Khởi động cùng hệ thống.** Chạy ngầm ở khay, không mở cửa sổ.
- **Cập nhật.** Cài đặt báo khi có bản mới. Chỉ cài khi bạn bấm Cập nhật.
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

Cài bằng [Homebrew](https://brew.sh):

```bash
brew install --cask kingleykin/tap/keepawake
```

Hoặc cài tay:

1. Mở file `.dmg`, kéo **KeepAwake** vào **Applications**.
2. Mở KeepAwake. macOS báo không xác minh được nhà phát triển. Bấm **Done**.
3. Mở **System Settings → Privacy & Security**, kéo xuống, bấm **Open Anyway** cạnh KeepAwake.

Hoặc chạy lệnh này một lần trong Terminal thay cho bước 2 và 3. Nếu cài bằng Homebrew mà macOS không cho mở app, bạn cũng chạy lệnh này:

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

App **không** chặn việc bạn chủ động cho máy ngủ, ví dụ gập laptop hoặc chọn Sleep trong menu. Xem [Gập máy thì sao?](#gập-máy-thì-sao)

| Hệ điều hành | Cách hoạt động | Kiểm tra |
|---|---|---|
| macOS | IOKit power assertion `PreventUserIdleSystemSleep` và `PreventUserIdleDisplaySleep` | `pmset -g assertions` |
| Windows | `SetThreadExecutionState` với `ES_SYSTEM_REQUIRED`, thêm `ES_DISPLAY_REQUIRED` ở chế độ màn hình | `powercfg /requests` |
| Linux | Thử XDG desktop portal `Inhibit` trước, sau đó `org.freedesktop.ScreenSaver` và systemd-logind | `systemd-inhibit --list` |

**Lưu ý trên Linux.** GNOME và KDE hoạt động trên cả Wayland và X11 qua desktop portal. Một số compositor Wayland tối giản như Sway, Hyprland có thể bỏ qua yêu cầu này. Nếu không có dịch vụ nào chấp nhận, app hiện lỗi thay vì giả vờ đang chạy. Trên Linux, click trái vào icon khay sẽ mở menu.

## Câu hỏi thường gặp

### Gập máy thì sao?

Trên macOS và Windows, máy vẫn ngủ như bình thường. Hai hệ điều hành này không cho app thông thường chặn việc đó, để laptop nằm trong balo không bị chạy và nóng lên. Trên Linux, KeepAwake cũng không đụng tới cài đặt gập máy, nhưng một số môi trường desktop có thể coi yêu cầu của app là lý do để không ngủ. Nếu máy bạn vẫn chạy khi gập, hãy tắt KeepAwake trước khi bỏ máy vào balo.

Khi mở máy lại, KeepAwake vẫn bật và tiếp tục giữ thức. Hẹn giờ tính theo giờ thật, nên thời gian máy ngủ cũng được tính. Nếu đã hết giờ trong lúc máy ngủ, app tự tắt ngay sau khi máy thức dậy, kèm thông báo nếu bạn bật thông báo.

Muốn gập máy mà máy vẫn chạy, hãy đổi cài đặt gập máy của hệ điều hành:

- **macOS:** dùng chế độ màn hình gập (clamshell): cắm sạc, nối màn hình ngoài và bàn phím hoặc chuột, rồi gập máy.
- **Windows:** Control Panel → Hardware and Sound → Power Options → **Choose what closing the lid does** → **Do nothing**.
- **Linux:** tạo file `/etc/systemd/logind.conf.d/lid.conf` gồm hai dòng `[Login]` và `HandleLidSwitch=ignore`, rồi khởi động lại máy.

Nhớ đổi lại khi bỏ laptop vào balo.

### KeepAwake có đổi cài đặt nguồn của máy không?

Không. App chỉ giữ một yêu cầu trong lúc đang bật. Khi tắt hoặc thoát app, yêu cầu đó mất. Nếu app bị lỗi và đóng đột ngột, hệ điều hành cũng tự bỏ yêu cầu đó.

### Vì sao lần đầu mở app máy lại cảnh báo?

App chưa được ký số. Xem phần [Cài đặt](#cài-đặt) để làm các bước cần thiết một lần.

### App có thu thập dữ liệu không?

Không. App chỉ kết nối mạng để kiểm tra bản mới (tải file `latest.json` từ mục Releases của repo này) và để tải bản cập nhật sau khi bạn bấm **Cập nhật**.

## Cài đặt trong app

| Mục | Mặc định | |
|---|---|---|
| Khởi động cùng hệ thống | Tắt | Chạy ẩn ở khay. |
| Nút đóng sẽ ẩn xuống tray | Bật | Nếu tắt, đóng cửa sổ là thoát app. |
| Báo khi hết giờ | Bật | Một thông báo khi hết giờ và app tự tắt giữ thức. |
| Phím tắt | Tắt | ⌥⌘K / Ctrl+Alt+K. Mặc định tắt vì tổ hợp này có nghĩa khác ở một số nơi: Ctrl+Alt+K là AltGr+K trên một số bàn phím Windows, còn ⌥⌘K mở Web Console của Firefox. Nếu ứng dụng khác đã dùng tổ hợp này, Cài đặt sẽ báo và app vẫn chạy bình thường. |
| Ngôn ngữ | Tiếng Anh | Tiếng Anh hoặc tiếng Việt. |

**Phím tắt trên Linux.** Phím tắt toàn cục cần X11. Trên Wayland chỉ chạy ở desktop có chuyển phím tắt cho app X11, nên khay và cửa sổ vẫn là cách chắc chắn nhất. Khi chạy trên Wayland, Cài đặt sẽ hiện ghi chú dưới công tắc này.

**Cập nhật.** Vài giây sau khi mở và mỗi lần mở Cài đặt, KeepAwake tải file `latest.json` của bản phát hành mới nhất trên GitHub. Không gửi dữ liệu nào khác. Bản mới chỉ được cài sau khi bạn bấm **Cập nhật**, và chỉ khi có chữ ký bằng khóa cập nhật của dự án. Trên Linux, bản AppImage, `.deb` và `.rpm` đều cập nhật được; bản `.deb` và `.rpm` sẽ hỏi mật khẩu.

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

Rất hoan nghênh báo lỗi và pull request. Mọi thay đổi vào `main` phải qua pull request, qua CI trên cả 3 hệ điều hành và được chủ dự án duyệt. Báo lỗi bảo mật xem [SECURITY.md](SECURITY.md). Xin giữ app đơn giản: làm một việc và làm tốt. Khi báo lỗi trên Linux, hãy ghi rõ bản phân phối, môi trường desktop, và đang dùng Wayland hay X11.

## Giấy phép

[MIT](LICENSE). Tự do sử dụng, chỉnh sửa và chia sẻ.
