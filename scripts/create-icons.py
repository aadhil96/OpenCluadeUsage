#!/usr/bin/env python3
"""
Generate placeholder app icons for ClaudeUsage.
Run once before `bun run tauri:build` if you don't have your own icon.

Usage:
    python3 scripts/create-icons.py

Requires: Pillow  (`pip install Pillow`)

After generating a proper icon source (512x512 PNG), run:
    bun run tauri:icon path/to/your-icon.png
to regenerate all sizes automatically via the Tauri CLI.
"""

import os
import sys

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("Pillow not found. Install it with:  pip install Pillow")
    sys.exit(1)

ICON_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(ICON_DIR, exist_ok=True)


def make_icon(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Dark rounded square background
    padding = size // 8
    radius = size // 4
    draw.rounded_rectangle(
        [padding, padding, size - padding, size - padding],
        radius=radius,
        fill=(30, 30, 40, 255),
    )

    # "C" letter in blue
    font_size = size // 2
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", font_size)
    except Exception:
        font = ImageFont.load_default()

    text = "C"
    bbox = draw.textbbox((0, 0), text, font=font)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    tx = (size - tw) // 2 - bbox[0]
    ty = (size - th) // 2 - bbox[1]
    draw.text((tx, ty), text, font=font, fill=(96, 165, 250, 255))  # blue-400

    return img


def save_png(img: Image.Image, name: str):
    path = os.path.join(ICON_DIR, name)
    img.save(path, "PNG")
    print(f"  {path}")


print("Generating icons…")

save_png(make_icon(32), "32x32.png")
save_png(make_icon(128), "128x128.png")
save_png(make_icon(256), "128x128@2x.png")
save_png(make_icon(512), "app-icon.png")   # source for `tauri icon`

# Tray icon (template image — should be grayscale/white on transparent)
tray_size = 22
tray = Image.new("RGBA", (tray_size * 2, tray_size * 2), (0, 0, 0, 0))  # @2x
draw = ImageDraw.Draw(tray)
try:
    font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 22)
except Exception:
    font = ImageFont.load_default()
draw.text((4, 3), "C", font=font, fill=(255, 255, 255, 220))
tray.save(os.path.join(ICON_DIR, "tray-icon.png"), "PNG")
print(f"  {os.path.join(ICON_DIR, 'tray-icon.png')}")

print("\nNote: For a proper .icns and .ico, run:")
print("  bun run tauri:icon src-tauri/icons/app-icon.png")
