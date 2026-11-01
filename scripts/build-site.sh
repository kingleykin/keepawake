#!/bin/sh
# Assembles the website in _site/ from site/ and the images it shares with the README.
# Vercel runs this (see vercel.json). Preview locally with: python3 -m http.server -d _site
set -eu
cd "$(dirname "$0")/.."

rm -rf _site
cp -R site _site
cp docs/demo.gif assets/icon.svg _site/
cp src-tauri/icons/128x128@2x.png _site/icon.png
