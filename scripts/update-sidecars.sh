#!/usr/bin/env bash
set -euo pipefail

# Fetches fresh yt-dlp release binaries for all supported target triples.
# NOTE: only the Linux x86_64 binary is vendored in-repo right now; the macOS
# and Windows assets are refreshed here so a single `source` updates every
# sidecar when cross-platform builds are needed.
mkdir -p sidecars/yt-dlp
base="https://github.com/yt-dlp/yt-dlp/releases/latest/download"
curl -L "$base/yt-dlp_linux" -o sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu
chmod +x sidecars/yt-dlp/yt-dlp-x86_64-unknown-linux-gnu
curl -L "$base/yt-dlp_macos" -o sidecars/yt-dlp/yt-dlp-aarch64-apple-darwin
chmod +x sidecars/yt-dlp/yt-dlp-aarch64-apple-darwin
curl -L "$base/yt-dlp_macos" -o sidecars/yt-dlp/yt-dlp-x86_64-apple-darwin
chmod +x sidecars/yt-dlp/yt-dlp-x86_64-apple-darwin
curl -L "$base/yt-dlp.exe" -o sidecars/yt-dlp/yt-dlp-x86_64-pc-windows-msvc.exe
