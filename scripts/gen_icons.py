#!/usr/bin/env python3
"""生成 Toolbox 应用图标 (PNG/ICO/ICNS)"""
import os
import shutil
import subprocess
import tempfile
from PIL import Image, ImageDraw, ImageFilter

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(OUTPUT_DIR, exist_ok=True)

W, H = 1024, 1024


def draw_icon(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    s = size / 1024

    radius = int(220 * s)
    bbox = [0, 0, size, size]
    d.rounded_rectangle(bbox, radius=radius, fill=(99, 102, 241, 255))

    overlay = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    od = ImageDraw.Draw(overlay)
    cx, cy = size / 2, size / 2
    r = size / 2
    for i in range(40):
        alpha = int(8 * (1 - i / 40))
        if alpha <= 0:
            continue
        offset = int(i * s * 4)
        od.ellipse(
            [cx - r + offset, cy - r - offset, cx + r + offset, cy + r - offset],
            fill=(255, 255, 255, alpha),
        )
    img = Image.alpha_composite(img, overlay)
    d = ImageDraw.Draw(img)

    pad = int(260 * s)
    clip_w = size - pad * 2
    clip_h = int(560 * s)
    clip_x = pad
    clip_y = (size - clip_h) // 2 + int(20 * s)
    clip_r = int(48 * s)
    d.rounded_rectangle(
        [clip_x, clip_y, clip_x + clip_w, clip_y + clip_h],
        radius=clip_r,
        fill=(255, 255, 255, 255),
    )

    clip_w2 = int(220 * s)
    clip_h2 = int(110 * s)
    clip_x2 = (size - clip_w2) // 2
    clip_y2 = clip_y - int(35 * s)
    clip_r2 = int(28 * s)
    d.rounded_rectangle(
        [clip_x2, clip_y2, clip_x2 + clip_w2, clip_y2 + clip_h2],
        radius=clip_r2,
        fill=(220, 220, 235, 255),
    )

    hole_w = int(70 * s)
    hole_h = int(20 * s)
    hole_x = (size - hole_w) // 2
    hole_y = clip_y2 + (clip_h2 - hole_h) // 2
    d.rounded_rectangle(
        [hole_x, hole_y, hole_x + hole_w, hole_y + hole_h],
        radius=int(8 * s),
        fill=(99, 102, 241, 255),
    )

    line_color = (99, 102, 241, 255)
    line_x1 = clip_x + int(70 * s)
    line_x2 = clip_x + clip_w - int(70 * s)
    line_y_start = clip_y + int(160 * s)
    line_w = max(int(8 * s), 2)
    for i in range(3):
        y = line_y_start + i * int(80 * s)
        d.rounded_rectangle(
            [line_x1, y, line_x2, y + line_w],
            radius=line_w // 2,
            fill=line_color,
        )

    return img


def save_png(img: Image.Image, name: str, size: int):
    resized = img.resize((size, size), Image.LANCZOS)
    path = os.path.join(OUTPUT_DIR, name)
    resized.save(path, "PNG", optimize=True)
    print(f"saved: {path} ({size}x{size})")


def save_ico(img: Image.Image):
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    path = os.path.join(OUTPUT_DIR, "icon.ico")
    img.save(path, format="ICO", sizes=sizes)
    print(f"saved: {path}")


def save_icns(img: Image.Image):
    iconset = tempfile.mkdtemp(prefix="toolbox_iconset_", suffix=".iconset")
    try:
        sizes = {
            "icon_16x16.png": 16,
            "icon_16x16@2x.png": 32,
            "icon_32x32.png": 32,
            "icon_32x32@2x.png": 64,
            "icon_128x128.png": 128,
            "icon_128x128@2x.png": 256,
            "icon_256x256.png": 256,
            "icon_256x256@2x.png": 512,
            "icon_512x512.png": 512,
            "icon_512x512@2x.png": 1024,
        }
        for name, sz in sizes.items():
            resized = img.resize((sz, sz), Image.LANCZOS)
            resized.save(os.path.join(iconset, name), "PNG", optimize=True)
        icns_path = os.path.join(OUTPUT_DIR, "icon.icns")
        subprocess.run(
            ["iconutil", "-c", "icns", iconset, "-o", icns_path],
            check=True,
        )
        print(f"saved: {icns_path}")
    finally:
        shutil.rmtree(iconset, ignore_errors=True)


def main():
    master = draw_icon(W)
    save_png(master, "32x32.png", 32)
    save_png(master, "128x128.png", 128)
    save_png(master, "128x128@2x.png", 256)
    save_ico(master)
    save_icns(master)
    print("All icons generated.")


if __name__ == "__main__":
    main()
