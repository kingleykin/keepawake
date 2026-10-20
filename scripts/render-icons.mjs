// Renders the app icon and tray icons from the SVG shapes below.
// Usage: npm run icons
import { mkdirSync, writeFileSync } from "node:fs";
import { Resvg } from "@resvg/resvg-js";

const CUP_BODY = "M7 13h15v6a6 6 0 0 1-6 6h-3a6 6 0 0 1-6-6z";
const HANDLE = "M22 15h2a3 3 0 0 1 0 6h-2";
const SAUCER = "M5 27.5h19";
const STEAM = [11, 14.5, 18].map((x) => `M${x} 5c-1.5 1.5 1.5 3 0 4.5`).join("");

/** A 32×32 cup. `filled` draws the body solid and adds steam. */
function cup({ color, filled, width = 2.2 }) {
  const line = `fill="none" stroke="${color}" stroke-width="${width}" stroke-linecap="round" stroke-linejoin="round"`;
  return [
    `<path d="${CUP_BODY}" ${filled ? `fill="${color}" stroke="${color}" stroke-width="${width}" stroke-linejoin="round"` : line}/>`,
    `<path d="${HANDLE}" ${line}/>`,
    `<path d="${SAUCER}" ${line}/>`,
    filled ? `<path d="${STEAM}" ${line}/>` : "",
  ].join("");
}

const tray = (body) =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">${body}</svg>`;

const appIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#34d399"/>
      <stop offset="1" stop-color="#15803d"/>
    </linearGradient>
  </defs>
  <rect x="64" y="64" width="896" height="896" rx="200" fill="url(#bg)"/>
  <g transform="translate(192 200) scale(20)">${cup({ color: "#ffffff", filled: true, width: 2.4 })}</g>
</svg>`;

const outputs = {
  "assets/icon.svg": appIcon,
  "assets/icon.png": [appIcon, 1024],
  "src-tauri/icons/tray/on.png": [tray(cup({ color: "#16a34a", filled: true })), 64],
  "src-tauri/icons/tray/off.png": [tray(cup({ color: "#9ca3af", filled: false })), 64],
  "src-tauri/icons/tray/on-template.png": [tray(cup({ color: "#000000", filled: true })), 64],
  "src-tauri/icons/tray/off-template.png": [tray(cup({ color: "#000000", filled: false })), 64],
};

mkdirSync("assets", { recursive: true });
mkdirSync("src-tauri/icons/tray", { recursive: true });
for (const [path, value] of Object.entries(outputs)) {
  if (typeof value === "string") {
    writeFileSync(path, value);
  } else {
    const [svg, size] = value;
    const png = new Resvg(svg, { fitTo: { mode: "width", value: size } }).render().asPng();
    writeFileSync(path, png);
  }
  console.log("wrote", path);
}
