// KeepAwake window. All state lives in the Rust side; this file only draws it.
(() => {
  "use strict";

  const tauri = window.__TAURI__;
  const invoke = tauri.core.invoke;
  const RING = 389.56; // 2 * PI * r, r = 62
  const DURATIONS = [15, 30, 60, 120, 240, 0];
  const REPO_URL = "https://github.com/kingleykin/keepawake";
  const MAX_CUSTOM_MIN = 1440;
  const SHORTCUT_KEYS = /Mac/.test(navigator.userAgent) ? "⌥⌘K" : "Ctrl+Alt+K";

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
    notify: $("notify"),
    shortcut: $("shortcut"),
    shortcutKeys: $("shortcut-keys"),
    shortcutError: $("shortcut-error"),
    shortcutNote: $("shortcut-note"),
    openSettings: $("open-settings"),
    update: $("update"),
    updateText: $("update-text"),
    install: $("install"),
    picker: $("picker"),
    pickerForm: $("picker-form"),
    pickerHint: $("picker-hint"),
    customMinutes: $("custom-minutes"),
    customUntil: $("custom-until"),
    language: $("language"),
    settingsError: $("settings-error"),
    version: $("version"),
    versionShort: $("version-short"),
  };

  let state = null;
  let version = "";
  let busy = false;
  let installing = false;

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

  const hhmm = (hour, minute) =>
    `${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}`;

  function customLabel(t, custom) {
    return custom.kind === "until"
      ? fill(t.until, { t: hhmm(custom.hour, custom.minute) })
      : fill(t.customMinutes, { n: custom.minutes });
  }

  function chip(label, checked, onClick) {
    const node = document.createElement("button");
    node.type = "button";
    node.className = "chip";
    node.setAttribute("role", "radio");
    node.setAttribute("aria-checked", String(checked));
    node.textContent = label;
    node.addEventListener("click", onClick);
    return node;
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

    const custom = state.useCustom && state.custom;
    const customChip = chip(custom ? customLabel(t, custom) : t.custom, Boolean(custom), openPicker);
    customChip.classList.add("wide");
    el.durations.replaceChildren(
      ...DURATIONS.map((minutes) =>
        chip(t.durations[minutes], !custom && minutes === state.durationMin, () =>
          call("set_duration", { minutes })
        )
      ),
      customChip
    );
    el.durations.setAttribute("role", "radiogroup");

    document.querySelectorAll('input[name="mode"]').forEach((radio) => {
      radio.checked = radio.value === state.mode;
    });
    el.autostart.checked = state.autostart;
    el.closeToTray.checked = state.closeToTray;
    el.notify.checked = state.notifyWhenDone;
    el.shortcut.checked = state.shortcut;
    el.shortcutKeys.textContent = SHORTCUT_KEYS;
    el.shortcutError.hidden = !(state.shortcut && state.shortcutError);
    el.shortcutError.textContent = fill(t.shortcutTaken, { k: SHORTCUT_KEYS });
    el.shortcutError.title = state.shortcutError || "";
    el.shortcutNote.hidden = !state.shortcutMayFail;
    el.update.hidden = !state.update;
    el.updateText.textContent = state.update ? fill(t.updateAvailable, { v: state.update }) : "";
    el.install.textContent = installing ? t.installing : t.install;
    el.install.disabled = installing;
    el.openSettings.classList.toggle("has-update", Boolean(state.update));
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
      const total = state.totalMs || 0;
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
    closePicker();
    el.home.hidden = show;
    el.settings.hidden = !show;
    (show ? $("back") : $("open-settings")).focus();
    // Look for a newer release each time Settings opens. Offline is fine.
    if (show) invoke("check_update").then((next) => {
      state = next;
      render();
    }, () => {});
  }

  async function installUpdate() {
    if (installing) return;
    installing = true;
    render();
    try {
      // The app restarts into the new version when this succeeds.
      await invoke("install_update");
    } catch (error) {
      el.settingsError.textContent = String(error);
      el.settingsError.hidden = false;
    }
    installing = false;
    render();
  }

  // ---- custom time picker ----

  const pickerKind = () =>
    document.querySelector('input[name="custom-kind"]:checked')?.value || "minutes";

  function setPickerKind(kind) {
    document.querySelectorAll('input[name="custom-kind"]').forEach((radio) => {
      radio.checked = radio.value === kind;
    });
  }

  /** The custom time typed in the picker, or null if it is not valid. */
  function pickedCustom() {
    if (pickerKind() === "minutes") {
      const minutes = Number(el.customMinutes.value);
      return Number.isInteger(minutes) && minutes >= 1 && minutes <= MAX_CUSTOM_MIN
        ? { kind: "minutes", minutes }
        : null;
    }
    // Some Linux webviews show a plain text box, so accept "18:30" typed by hand too.
    const match = /^\s*(\d{1,2})[:.h](\d{2})/.exec(el.customUntil.value);
    if (!match) return null;
    const hour = Number(match[1]);
    const minute = Number(match[2]);
    return hour < 24 && minute < 60 ? { kind: "until", hour, minute } : null;
  }

  function renderPickerHint() {
    const t = strings();
    const custom = pickedCustom();
    el.pickerHint.classList.remove("error");
    if (!custom) {
      el.pickerHint.textContent = t.customInvalid;
      el.pickerHint.classList.add("error");
    } else if (custom.kind === "until") {
      const now = new Date();
      const passed = custom.hour * 60 + custom.minute <= now.getHours() * 60 + now.getMinutes();
      el.pickerHint.textContent = fill(passed ? t.endsTomorrow : t.endsToday, {
        t: hhmm(custom.hour, custom.minute),
      });
    } else {
      el.pickerHint.textContent = "";
    }
  }

  function openPicker() {
    const saved = state.custom;
    const soon = new Date(Date.now() + 60 * 60_000);
    el.customMinutes.value = saved?.kind === "minutes" ? saved.minutes : 45;
    el.customUntil.value =
      saved?.kind === "until" ? hhmm(saved.hour, saved.minute) : hhmm(soon.getHours(), 0);
    setPickerKind(saved?.kind || "minutes");
    el.pickerHint.textContent = "";
    el.pickerHint.classList.remove("error");
    el.picker.hidden = false;
    (pickerKind() === "until" ? el.customUntil : el.customMinutes).focus();
  }

  function closePicker() {
    if (el.picker.hidden) return;
    el.picker.hidden = true;
    if (!el.home.hidden) el.durations.querySelector(".wide")?.focus();
  }

  // ---- wiring ----

  el.toggle.addEventListener("click", toggle);
  document.querySelectorAll('input[name="mode"]').forEach((radio) => {
    radio.addEventListener("change", () => call("set_mode", { mode: radio.value }));
  });
  $("open-settings").addEventListener("click", () => showSettings(true));
  $("back").addEventListener("click", () => showSettings(false));
  $("hide").addEventListener("click", () => invoke("hide_to_tray"));
  el.install.addEventListener("click", installUpdate);
  // Typing in a field picks its option.
  el.customMinutes.addEventListener("focus", () => setPickerKind("minutes"));
  el.customUntil.addEventListener("focus", () => setPickerKind("until"));
  [el.customMinutes, el.customUntil].forEach((field) =>
    field.addEventListener("input", renderPickerHint)
  );
  document.querySelectorAll('input[name="custom-kind"]').forEach((radio) => {
    radio.addEventListener("change", renderPickerHint);
  });
  el.pickerForm.addEventListener("submit", async (event) => {
    event.preventDefault();
    const custom = pickedCustom();
    if (!custom) {
      renderPickerHint();
      return;
    }
    closePicker();
    await call("set_custom", { custom });
  });
  $("picker-cancel").addEventListener("click", closePicker);
  el.picker.addEventListener("click", (event) => {
    if (event.target === el.picker) closePicker();
  });
  $("quit").addEventListener("click", () => invoke("quit_app"));
  $("github").addEventListener("click", () => tauri.opener.openUrl(REPO_URL));
  el.autostart.addEventListener("change", () =>
    call("set_autostart", { enabled: el.autostart.checked }).then(render)
  );
  el.closeToTray.addEventListener("change", () =>
    call("set_close_to_tray", { enabled: el.closeToTray.checked })
  );
  el.notify.addEventListener("change", () =>
    call("set_notify_when_done", { enabled: el.notify.checked })
  );
  el.shortcut.addEventListener("change", () =>
    call("set_shortcut", { enabled: el.shortcut.checked })
  );
  el.language.addEventListener("change", () =>
    call("set_language", { language: el.language.value })
  );
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    if (!el.picker.hidden) closePicker();
    else if (!el.settings.hidden) showSettings(false);
  });
  // No browser context menu in a desktop app.
  document.addEventListener("contextmenu", (event) => event.preventDefault());

  tauri.event.listen("state", (event) => {
    state = event.payload;
    render();
  });
  tauri.event.listen("show-settings", () => showSettings(true));
  tauri.event.listen("show-custom", () => {
    if (!el.settings.hidden) showSettings(false);
    if (state) openPicker();
  });

  setInterval(() => state && state.active && renderLive(), 1000);

  (async () => {
    version = await tauri.app.getVersion().catch(() => "");
    state = await invoke("get_state");
    render();
  })();
})();
