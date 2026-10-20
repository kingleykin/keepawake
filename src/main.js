// KeepAwake window. All state lives in the Rust side; this file only draws it.
(() => {
  "use strict";

  const tauri = window.__TAURI__;
  const invoke = tauri.core.invoke;
  const RING = 389.56; // 2 * PI * r, r = 62
  const DURATIONS = [15, 30, 60, 120, 240, 0];
  const REPO_URL = "https://github.com/kingleykin/keepawake";

  const $ = (id) => document.getElementById(id);
  const el = {
    home: $("home"),
    settings: $("settings"),
    toggle: $("toggle"),
    progress: $("progress"),
    status: $("status"),
    detail: $("detail"),
    durations: $("durations"),
    autostart: $("autostart"),
    closeToTray: $("close-to-tray"),
    language: $("language"),
    settingsError: $("settings-error"),
    version: $("version"),
    versionShort: $("version-short"),
  };

  let state = null;
  let version = "";
  let busy = false;

  const strings = () => window.I18N[state?.language] || window.I18N.en;
  const fill = (template, values) =>
    template.replace(/\{(\w+)\}/g, (_, key) => values[key] ?? "");

  function clock(ms) {
    const total = Math.ceil(ms / 1000);
    const h = Math.floor(total / 3600);
    const m = Math.floor((total % 3600) / 60);
    const s = total % 60;
    const pad = (n) => String(n).padStart(2, "0");
    return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
  }

  // ---- drawing ----

  function renderStatic() {
    const t = strings();
    document.documentElement.lang = state.language;
    document.querySelectorAll("[data-i18n]").forEach((node) => {
      node.textContent = t[node.dataset.i18n];
    });
    document.querySelectorAll("[data-i18n-label]").forEach((node) => {
      node.setAttribute("aria-label", t[node.dataset.i18nLabel]);
      node.title = t[node.dataset.i18nLabel];
    });

    el.durations.replaceChildren(
      ...DURATIONS.map((minutes) => {
        const chip = document.createElement("button");
        chip.type = "button";
        chip.className = "chip";
        chip.setAttribute("role", "radio");
        chip.setAttribute("aria-checked", String(minutes === state.durationMin));
        chip.textContent = t.durations[minutes];
        chip.addEventListener("click", () => call("set_duration", { minutes }));
        return chip;
      })
    );
    el.durations.setAttribute("role", "radiogroup");

    document.querySelectorAll('input[name="mode"]').forEach((radio) => {
      radio.checked = radio.value === state.mode;
    });
    el.autostart.checked = state.autostart;
    el.closeToTray.checked = state.closeToTray;
    el.language.value = state.language;
    el.version.textContent = version ? fill(t.version, { v: version }) : "";
    el.versionShort.textContent = version ? `v${version}` : "";
  }

  function renderLive() {
    const t = strings();
    const failed = !state.active && Boolean(state.error);

    el.toggle.classList.toggle("on", state.active);
    el.toggle.classList.toggle("error", failed);
    el.toggle.setAttribute("aria-pressed", String(state.active));
    el.toggle.setAttribute("aria-label", state.active ? t.toggleOff : t.toggleOn);
    el.status.className = "status" + (state.active ? " on" : failed ? " error" : "");
    el.status.textContent = state.active ? t.statusOn : failed ? t.statusError : t.statusOff;
    el.detail.classList.toggle("error", failed);

    let progress = 0;
    if (state.active && state.endsAtMs) {
      const remaining = Math.max(0, state.endsAtMs - Date.now());
      const total = state.durationMin * 60_000;
      progress = total > 0 ? Math.min(1, remaining / total) : 0;
      el.detail.textContent = fill(t.left, { t: clock(remaining) });
    } else if (state.active) {
      progress = 1;
      el.detail.textContent = t.noLimit;
    } else {
      el.detail.textContent = failed ? state.error : t.clickToStart;
    }
    el.detail.title = failed ? state.error : "";
    el.progress.style.strokeDashoffset = String(RING * (1 - progress));
    el.progress.style.opacity = state.active ? "1" : "0";
  }

  function render() {
    if (!state) return;
    renderStatic();
    renderLive();
  }

  // ---- talking to Rust ----

  async function call(command, args) {
    try {
      const next = await invoke(command, args);
      if (next) {
        state = next;
        render();
      }
      el.settingsError.hidden = true;
    } catch (error) {
      el.settingsError.textContent = String(error);
      el.settingsError.hidden = false;
    }
  }

  async function toggle() {
    if (busy || !state) return;
    busy = true;
    el.toggle.setAttribute("aria-busy", "true");
    await call("set_active", { active: !state.active });
    el.toggle.removeAttribute("aria-busy");
    busy = false;
  }

  function showSettings(show) {
    el.home.hidden = show;
    el.settings.hidden = !show;
    (show ? $("back") : $("open-settings")).focus();
  }

  // ---- wiring ----

  el.toggle.addEventListener("click", toggle);
  document.querySelectorAll('input[name="mode"]').forEach((radio) => {
    radio.addEventListener("change", () => call("set_mode", { mode: radio.value }));
  });
  $("open-settings").addEventListener("click", () => showSettings(true));
  $("back").addEventListener("click", () => showSettings(false));
  $("hide").addEventListener("click", () => invoke("hide_to_tray"));
  $("quit").addEventListener("click", () => invoke("quit_app"));
  $("github").addEventListener("click", () => tauri.opener.openUrl(REPO_URL));
  el.autostart.addEventListener("change", () =>
    call("set_autostart", { enabled: el.autostart.checked }).then(render)
  );
  el.closeToTray.addEventListener("change", () =>
    call("set_close_to_tray", { enabled: el.closeToTray.checked })
  );
  el.language.addEventListener("change", () =>
    call("set_language", { language: el.language.value })
  );
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !el.settings.hidden) showSettings(false);
  });
  // No browser context menu in a desktop app.
  document.addEventListener("contextmenu", (event) => event.preventDefault());

  tauri.event.listen("state", (event) => {
    state = event.payload;
    render();
  });
  tauri.event.listen("show-settings", () => showSettings(true));

  setInterval(() => state && state.active && renderLive(), 1000);

  (async () => {
    version = await tauri.app.getVersion().catch(() => "");
    state = await invoke("get_state");
    render();
  })();
})();
