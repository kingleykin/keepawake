#!/usr/bin/env node
// Writes and checks latest.json, the file installed copies read to find updates.
//
//   node scripts/latest-json.mjs write <version> <release.json> <sig dir>  > latest.json
//   node scripts/latest-json.mjs check <version> <release.json> <sig dir> <latest.json>
//
// release.json is the release as the GitHub API returns it. The sig dir holds
// every .sig file of that release, under its own name. The release workflow
// writes latest.json once, after every build has uploaded its files, so the
// builds cannot overwrite each other's entries.

import { readFileSync } from "node:fs";
import { join } from "node:path";

// Each key the updater looks up, and the file it installs. Tauri looks for
// `<os>-<arch>-<bundle>` first, then `<os>-<arch>`, so a copy installed from a
// .deb updates with a .deb.
function updates(version) {
  const v = version;
  const mac = `KeepAwake_${v}_universal.app.tar.gz`;
  const files = {
    "windows-x86_64": `KeepAwake_${v}_x64-setup.exe`,
    "windows-x86_64-nsis": `KeepAwake_${v}_x64-setup.exe`,
    "windows-x86_64-msi": `KeepAwake_${v}_x64_en-US.msi`,
    "windows-aarch64": `KeepAwake_${v}_arm64-setup.exe`,
    "windows-aarch64-nsis": `KeepAwake_${v}_arm64-setup.exe`,
    "linux-x86_64": `KeepAwake_${v}_amd64.AppImage`,
    "linux-x86_64-appimage": `KeepAwake_${v}_amd64.AppImage`,
    "linux-x86_64-deb": `KeepAwake_${v}_amd64.deb`,
    "linux-x86_64-rpm": `KeepAwake-${v}-1.x86_64.rpm`,
    "linux-aarch64": `KeepAwake_${v}_aarch64.AppImage`,
    "linux-aarch64-appimage": `KeepAwake_${v}_aarch64.AppImage`,
    "linux-aarch64-deb": `KeepAwake_${v}_arm64.deb`,
    "linux-aarch64-rpm": `KeepAwake-${v}-1.aarch64.rpm`,
  };
  for (const arch of ["aarch64", "x86_64", "universal"]) {
    files[`darwin-${arch}`] = mac;
    files[`darwin-${arch}-app`] = mac;
  }
  return files;
}

// Files people download by hand, with no update of their own.
const downloads = (v) => [`KeepAwake_${v}_universal.dmg`];

// The platforms latest.json must have, as the release builds them, checked
// with the problems found so far. Never throws, so every problem is reported.
function expected(version, release, sigDir, problems) {
  const assets = new Map(release.assets.map((a) => [a.name, a]));
  const present = (name) => {
    const asset = assets.get(name);
    if (!asset) problems.push(`the release has no ${name}`);
    else if (asset.state !== "uploaded" || !(asset.size > 0))
      problems.push(`${name} did not finish uploading`);
    else return asset;
    return null;
  };
  for (const name of downloads(version)) present(name);
  const platforms = {};
  for (const [key, name] of Object.entries(updates(version))) {
    const asset = present(name);
    const sig = present(`${name}.sig`);
    if (!asset || !sig) continue;
    let signature;
    try {
      signature = readFileSync(join(sigDir, `${name}.sig`), "utf8").trim();
    } catch {
      problems.push(`${name}.sig was not downloaded`);
      continue;
    }
    if (!signature) {
      problems.push(`${name}.sig is empty`);
      continue;
    }
    // The API address works while the release is still a draft, and keeps working after.
    platforms[key] = { signature, url: asset.url };
  }
  return platforms;
}

function fail(problems) {
  for (const p of new Set(problems)) console.error(`latest.json: ${p}`);
  process.exit(1);
}

const [command, version, releaseFile, sigDir, latestFile] = process.argv.slice(2);
if (!/^\d+\.\d+\.\d+$/.test(version ?? "") || !releaseFile || !sigDir) {
  console.error("usage: latest-json.mjs write|check <version> <release.json> <sig dir> [latest.json]");
  process.exit(2);
}
const release = JSON.parse(readFileSync(releaseFile, "utf8"));
const problems = [];
const platforms = expected(version, release, sigDir, problems);

if (command === "write") {
  if (problems.length) fail(problems);
  const latest = {
    version,
    notes: release.body ?? "",
    pub_date: new Date().toISOString(),
    platforms,
  };
  process.stdout.write(JSON.stringify(latest, null, 2) + "\n");
} else if (command === "check" && latestFile) {
  let latest;
  try {
    latest = JSON.parse(readFileSync(latestFile, "utf8"));
  } catch (e) {
    fail([...problems, `cannot be read: ${e.message}`]);
  }
  if (latest.version !== version)
    problems.push(`version is ${JSON.stringify(latest.version)}, expected "${version}"`);
  if (Number.isNaN(Date.parse(latest.pub_date))) problems.push("pub_date is not a date");
  const found = latest.platforms ?? {};
  for (const key of Object.keys(updates(version))) {
    const want = platforms[key];
    const got = found[key];
    if (!got) problems.push(`no update for ${key}`);
    else if (want && got.url !== want.url)
      problems.push(`${key} points to ${got.url}, expected ${want.url} (${updates(version)[key]})`);
    else if (want && got.signature !== want.signature)
      problems.push(`${key} has a signature that differs from ${updates(version)[key]}.sig`);
  }
  for (const key of Object.keys(found))
    if (!(key in updates(version))) problems.push(`unexpected platform ${key}`);
  if (problems.length) fail(problems);
  console.log(`latest.json: ${version}, ${Object.keys(found).length} entries, all signed and in the release`);
} else {
  console.error("usage: latest-json.mjs write|check <version> <release.json> <sig dir> [latest.json]");
  process.exit(2);
}
