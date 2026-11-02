// KeepAwake website: language switch, download button for the visitor's system,
// install tabs, copy buttons and the demo pause button. The page works without this
// file, in English, with every install section shown.

const RELEASES = "https://github.com/kingleykin/keepawake/releases/latest";
const API = "https://api.github.com/repos/kingleykin/keepawake/releases/latest";

const ASSETS = {
  mac: /_universal\.dmg$/,
  win: /_x64-setup\.exe$/,
  appimage: /_amd64\.AppImage$/,
  deb: /_amd64\.deb$/,
  rpm: /\.x86_64\.rpm$/,
};

// English text comes from the HTML. Only Vietnamese and the dynamic strings live here.
const VI = {
  title: "Giữ máy tính luôn thức.",
  lead: "Một cú bấm, từ cửa sổ nhỏ hoặc khay hệ thống. Miễn phí, mã nguồn mở, cho macOS, Windows và Linux.",
  dl: "Tải về",
  dlMeta: "Miễn phí · macOS, Windows và Linux",
  dlOther: "Hệ điều hành khác và Homebrew",
  demoAlt: "Chọn 30 phút, bật KeepAwake, rồi đặt giữ thức đến 18:30. Biểu tượng trên khay chuyển sang màu xanh khi đang bật.",
  featuresTitle: "Làm được gì",
  f1t: "Một cú bấm",
  f1: "Bấm nút lớn, hoặc bấm biểu tượng trên khay ở macOS và Windows. Biểu tượng chuyển xanh khi đang bật.",
  f2t: "Hẹn giờ",
  f2: "Từ 15 phút đến 4 giờ, không giới hạn, số phút tùy ý, hoặc đến một giờ cụ thể như 18:30.",
  f3t: "Hai chế độ",
  f3: "Giữ màn hình luôn sáng, hoặc chỉ giữ máy thức và để màn hình tự tắt.",
  f4t: "Phím tắt",
  f4: "Tùy chọn ⌥⌘K trên macOS hoặc Ctrl+Alt+K trên Windows và Linux. Bật trong Cài đặt. Trên Linux cần X11.",
  f5t: "Không làm phiền",
  f5: "Nằm gọn trên khay, tự chạy khi đăng nhập nếu bạn muốn, và báo khi hết giờ.",
  f6t: "Nhẹ và riêng tư",
  f6: "Vài megabyte, viết bằng Rust. Không cần tài khoản, không theo dõi, không đổi cài đặt nguồn của máy.",
  installTitle: "Cài đặt",
  macBrew: "Bằng Homebrew:",
  macDmg: 'Hoặc tải file <a class="asset" data-asset="mac" href="' + RELEASES + '">.dmg</a> (macOS 10.15 trở lên, chip Apple và Intel) rồi kéo KeepAwake vào Applications.',
  macNote: "App chưa được Apple công chứng nên lần đầu macOS sẽ chặn. Mở <b>System Settings → Privacy &amp; Security</b> và bấm <b>Open Anyway</b>, hoặc chạy lệnh này một lần:",
  winExe: 'Tải <a class="asset" data-asset="win" href="' + RELEASES + '">bộ cài</a> cho Windows 10 hoặc 11 rồi chạy. App chỉ cài cho tài khoản của bạn, không cần quyền admin.',
  winNote: "Bộ cài chưa được ký số. Nếu SmartScreen báo “Windows protected your PC”, bấm <b>More info → Run anyway</b>.",
  linuxFiles: 'Tải <a class="asset" data-asset="appimage" href="' + RELEASES + '">AppImage</a> cho mọi bản phân phối, <a class="asset" data-asset="deb" href="' + RELEASES + '">.deb</a> cho Debian và Ubuntu, hoặc <a class="asset" data-asset="rpm" href="' + RELEASES + '">.rpm</a> cho Fedora và openSUSE. Tất cả cho máy x86_64.',
  linuxNote: 'GNOME gốc ẩn biểu tượng trên khay. Cài <a href="https://extensions.gnome.org/extension/615/appindicator-support/">tiện ích AppIndicator</a> để thấy KeepAwake. Ubuntu đã có sẵn.',
  faqTitle: "Câu hỏi thường gặp",
  q1: "Gập máy thì sao?",
  a1: "Trên macOS và Windows, máy vẫn ngủ như bình thường để không bị nóng khi bỏ trong túi. KeepAwake chỉ ngăn máy ngủ vì không có ai dùng. Khi mở máy ra, app vẫn đang bật và bộ hẹn giờ vẫn tính tiếp.",
  q2: "App có đổi cài đặt nguồn của máy không?",
  a2: "Không. App chỉ giữ một yêu cầu trong lúc đang bật. Khi tắt hoặc thoát app, yêu cầu biến mất. Nếu app bị lỗi và tắt đột ngột, hệ điều hành tự bỏ yêu cầu đó.",
  q3: "Vì sao lần đầu mở app máy lại cảnh báo?",
  a3: "Các bản build chưa được ký số vì chứng chỉ ký tốn phí. Các bước trong phần Cài đặt chỉ cần làm một lần.",
  q4: "App có thu thập dữ liệu không?",
  a4: "Không. App chỉ kết nối mạng để hỏi GitHub có bản mới không, và để tải bản đó về sau khi bạn bấm Cài đặt. Riêng trang web này đếm lượt xem ẩn danh bằng Vercel Web Analytics, không dùng cookie.",
  footer: 'Giấy phép MIT · <a href="https://github.com/kingleykin/keepawake">Mã nguồn trên GitHub</a> · Làm bằng <a href="https://tauri.app">Tauri</a> và Rust',
};

const DYNAMIC = {
  en: {
    dlFor: (os) => "Download for " + os,
    universal: "Apple Silicon and Intel",
    linuxMeta: "AppImage, x86_64. .deb and .rpm below",
    mobile: "For macOS, Windows and Linux. Open this page on your computer.",
    copy: "Copy",
    copied: "Copied",
    pause: "Pause",
    play: "Play",
    other: "Tiếng Việt",
  },
  vi: {
    dlFor: (os) => "Tải cho " + os,
    universal: "chip Apple và Intel",
    linuxMeta: "AppImage, x86_64. .deb và .rpm ở bên dưới",
    mobile: "Cho macOS, Windows và Linux. Hãy mở trang này trên máy tính.",
    copy: "Chép",
    copied: "Đã chép",
    pause: "Tạm dừng",
    play: "Phát",
    other: "English",
  },
};

const OS_NAMES = { mac: "macOS", win: "Windows", linux: "Linux" };

const EN = {};
let lang = "en";
let release = null;
const os = detectOs();

function detectOs() {
  const ua = navigator.userAgent;
  const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
  if (/Android|iPhone|iPad|iPod/i.test(ua)) return null;
  if (/Mac/i.test(platform) || /Macintosh/.test(ua)) {
    // iPadOS reports itself as a Mac, but has a touch screen.
    return navigator.maxTouchPoints > 1 ? null : "mac";
  }
  if (/Win/i.test(platform) || /Windows/.test(ua)) return "win";
  if (/Linux|X11/i.test(platform) || /Linux|X11/.test(ua)) return "linux";
  return null;
}

function savedLang() {
  try {
    const fromUrl = new URLSearchParams(location.search).get("lang");
    if (fromUrl === "en" || fromUrl === "vi") return fromUrl;
    const stored = localStorage.getItem("lang");
    if (stored === "en" || stored === "vi") return stored;
  } catch {}
  return (navigator.language || "").toLowerCase().startsWith("vi") ? "vi" : "en";
}

function applyLang(next) {
  lang = next;
  document.documentElement.lang = lang;
  for (const el of document.querySelectorAll("[data-i18n]")) {
    const key = el.dataset.i18n;
    el.textContent = lang === "vi" ? VI[key] : EN[key];
  }
  for (const el of document.querySelectorAll("[data-i18n-html]")) {
    const key = el.dataset.i18nHtml;
    el.innerHTML = lang === "vi" ? VI[key] : EN[key];
  }
  for (const el of document.querySelectorAll("[data-i18n-alt]")) {
    const key = el.dataset.i18nAlt;
    el.alt = lang === "vi" ? VI[key] : EN[key];
  }
  const button = document.getElementById("lang");
  button.textContent = DYNAMIC[lang].other;
  button.lang = lang === "vi" ? "en" : "vi";
  document.title = lang === "vi" ? "KeepAwake – giữ máy tính luôn thức" : EN.pageTitle;
  updateDownloads();
  addCopyButtons();
  updateDemoButton();
}

function findAsset(kind) {
  if (!release) return null;
  return release.assets.find((a) => ASSETS[kind].test(a.name)) || null;
}

function megabytes(bytes) {
  return (bytes / 1048576).toFixed(1) + " MB";
}

function updateDownloads() {
  const t = DYNAMIC[lang];
  for (const link of document.querySelectorAll("a.asset")) {
    const asset = findAsset(link.dataset.asset);
    if (asset) link.href = asset.browser_download_url;
  }

  const button = document.getElementById("dl");
  const meta = document.getElementById("dl-meta");
  if (!os) {
    meta.textContent = t.mobile;
    return;
  }
  button.textContent = t.dlFor(OS_NAMES[os]);
  const asset = findAsset(os === "linux" ? "appimage" : os);
  if (!asset) return;
  button.href = asset.browser_download_url;
  const version = release.tag_name;
  if (os === "mac") meta.textContent = [version, t.universal, megabytes(asset.size)].join(" · ");
  else if (os === "win") meta.textContent = [version, "Windows 10 / 11", megabytes(asset.size)].join(" · ");
  else meta.textContent = [version, t.linuxMeta, megabytes(asset.size)].join(" · ");
}

async function loadRelease() {
  try {
    const response = await fetch(API, { headers: { Accept: "application/vnd.github+json" } });
    if (!response.ok) return;
    release = await response.json();
    updateDownloads();
  } catch {
    // Offline or rate-limited: the links keep pointing at the releases page.
  }
}

function selectTab(tab, focus) {
  for (const other of document.querySelectorAll('[role="tab"]')) {
    const selected = other === tab;
    other.setAttribute("aria-selected", String(selected));
    other.tabIndex = selected ? 0 : -1;
    document.getElementById(other.getAttribute("aria-controls")).hidden = !selected;
  }
  if (focus) tab.focus();
}

function setUpTabs() {
  const tabs = [...document.querySelectorAll('[role="tab"]')];
  for (const tab of tabs) {
    tab.addEventListener("click", () => selectTab(tab, false));
    tab.addEventListener("keydown", (event) => {
      const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
      if (!step) return;
      event.preventDefault();
      const next = tabs[(tabs.indexOf(tab) + step + tabs.length) % tabs.length];
      selectTab(next, true);
    });
  }
  document.querySelector('[role="tablist"]').hidden = false;
  selectTab(document.getElementById("tab-" + (os || "mac")), false);
}

// A GIF cannot be paused, so pausing draws the current frame onto a canvas and shows that.
let demoPaused = false;

function setDemoPaused(paused) {
  const img = document.querySelector(".demo img");
  const canvas = document.querySelector(".demo canvas");
  if (paused) {
    canvas.width = img.naturalWidth;
    canvas.height = img.naturalHeight;
    canvas.getContext("2d").drawImage(img, 0, 0);
  }
  demoPaused = paused;
  canvas.hidden = !paused;
  img.hidden = paused;
  updateDemoButton();
}

function updateDemoButton() {
  const button = document.getElementById("demo-toggle");
  const canvas = document.querySelector(".demo canvas");
  button.textContent = demoPaused ? DYNAMIC[lang].play : DYNAMIC[lang].pause;
  button.setAttribute("aria-pressed", String(demoPaused));
  canvas.setAttribute("aria-label", document.querySelector(".demo img").alt);
}

function setUpDemo() {
  const img = document.querySelector(".demo img");
  const button = document.getElementById("demo-toggle");
  button.hidden = false;
  button.addEventListener("click", () => setDemoPaused(!demoPaused));
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
    if (img.complete) setDemoPaused(true);
    else img.addEventListener("load", () => setDemoPaused(true), { once: true });
  }
}

function addCopyButtons() {
  if (!navigator.clipboard) return;
  for (const pre of document.querySelectorAll("pre")) {
    let button = pre.querySelector(".copy");
    if (!button) {
      button = document.createElement("button");
      button.type = "button";
      button.className = "copy";
      button.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(pre.querySelector("code").textContent);
          button.textContent = DYNAMIC[lang].copied;
          setTimeout(() => (button.textContent = DYNAMIC[lang].copy), 1500);
        } catch {}
      });
      pre.append(button);
    }
    button.textContent = DYNAMIC[lang].copy;
  }
}

for (const el of document.querySelectorAll("[data-i18n]")) EN[el.dataset.i18n] = el.textContent;
for (const el of document.querySelectorAll("[data-i18n-html]")) EN[el.dataset.i18nHtml] = el.innerHTML;
for (const el of document.querySelectorAll("[data-i18n-alt]")) EN[el.dataset.i18nAlt] = el.alt;
EN.pageTitle = document.title;

document.getElementById("lang").addEventListener("click", () => {
  const next = lang === "vi" ? "en" : "vi";
  try {
    localStorage.setItem("lang", next);
    // A ?lang= in the address would win over the saved choice on the next visit.
    const url = new URL(location.href);
    if (url.searchParams.has("lang")) {
      url.searchParams.delete("lang");
      history.replaceState(null, "", url);
    }
  } catch {}
  applyLang(next);
});

document.documentElement.classList.add("js");
setUpTabs();
setUpDemo();
applyLang(savedLang());
loadRelease();
