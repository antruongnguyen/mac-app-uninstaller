#!/usr/bin/env bash
set -euo pipefail

# Usage:
# ./svg-to-icns.sh path/to/icon.svg output_name.icns
# Example:
# ./svg-to-icns.sh resources/icon.svg resources/icon.icns

SVG_SRC="$1"
OUT_ICNS="${2:-icon.icns}"
TMP_DIR="icon.iconset"

if [[ -z "$SVG_SRC" || ! -f "$SVG_SRC" ]]; then
  echo "Usage: $0 path/to/icon.svg [out.icns]"
  echo "Example: $0 resources/icon.svg resources/icon.icns"
  exit 1
fi

command -v rsvg-convert >/dev/null 2>&1 || {
  echo "rsvg-convert not found. Install librsvg (brew install librsvg) and try again."
  exit 2
}
command -v iconutil >/dev/null 2>&1 || {
  echo "iconutil not found. This script must be run on macOS with Xcode command line tools installed."
  exit 3
}

# Clean/create tmp iconset dir
if [[ -d "$TMP_DIR" ]]; then
  rm -rf "$TMP_DIR"
fi
mkdir -p "$TMP_DIR"

# Generate PNGs
# Each command: rsvg-convert -w <px> input.svg -o output.png
# Note: for @2x images we generate the larger size and name accordingly.

echo "Generating PNGs in $TMP_DIR ..."

# 16x16
rsvg-convert -w 16  "$SVG_SRC" -o "$TMP_DIR/icon_16x16.png"
# 16x16@2x -> 32x32
rsvg-convert -w 32  "$SVG_SRC" -o "$TMP_DIR/icon_16x16@2x.png"

# 32x32
rsvg-convert -w 32  "$SVG_SRC" -o "$TMP_DIR/icon_32x32.png"
# 32x32@2x -> 64x64
rsvg-convert -w 64  "$SVG_SRC" -o "$TMP_DIR/icon_32x32@2x.png"

# 128x128
rsvg-convert -w 128 "$SVG_SRC" -o "$TMP_DIR/icon_128x128.png"
# 128x128@2x -> 256x256
rsvg-convert -w 256 "$SVG_SRC" -o "$TMP_DIR/icon_128x128@2x.png"

# 256x256
rsvg-convert -w 256 "$SVG_SRC" -o "$TMP_DIR/icon_256x256.png"
# 256x256@2x -> 512x512
rsvg-convert -w 512 "$SVG_SRC" -o "$TMP_DIR/icon_256x256@2x.png"

# 512x512
rsvg-convert -w 512 "$SVG_SRC" -o "$TMP_DIR/icon_512x512.png"
# 512x512@2x -> 1024x1024
rsvg-convert -w 1024 "$SVG_SRC" -o "$TMP_DIR/icon_512x512@2x.png"

echo "All PNGs generated."

# Convert to .icns
echo "Building ICNS -> $OUT_ICNS"
iconutil -c icns "$TMP_DIR" -o "$OUT_ICNS"

echo "ICNS created at: $OUT_ICNS"

# Optional: cleanup temp iconset
read -p "Remove temporary folder $TMP_DIR? [Y/n] " yn
yn=${yn:-Y}
if [[ "$yn" =~ ^[Yy] ]]; then
  rm -rf "$TMP_DIR"
  echo "Removed $TMP_DIR"
else
  echo "Kept $TMP_DIR (you can remove it manually later)"
fi

echo "Done."
