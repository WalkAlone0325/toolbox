#!/usr/bin/env python3
"""生成 Toolbox 应用图标 - 现代渐变 + 立体剪贴板"""
import os
import shutil
import subprocess
import tempfile
from PIL import Image, ImageDraw, ImageFilter

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(OUTPUT_DIR, exist_ok=True)

W, H = 1024, 1024

INDIGO = (99, 102, 241, 255)
INDIGO_LIGHT = (129, 140, 248, 255)
PURPLE = (168, 85, 247, 255)
PINK = (236, 72, 153, 255)
WHITE = (255, 255, 255, 255)
PAPER = (248, 250, 255, 255)
CLIP_DARK = (67, 56, 202, 255)
CLIP_LIGHT = (99, 102, 241, 255)
LINE_BLUE = (129, 140, 248, 255)


def lerp_color(c1, c2, t):
    return tuple(int(c1[i] + (c2[i] - c1[i]) * t) for i in range(len(c1)))


def make_gradient(w, h, colors):
    """生成多色水平+垂直渐变"""
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    px = img.load()
    for y in range(h):
        for x in range(w):
            tx = x / max(w - 1, 1)
            ty = y / max(h - 1, 1)
            t = tx * 0.6 + ty * 0.4
            seg = t * (len(colors) - 1)
            i = int(seg)
            f = seg - i
            if i >= len(colors) - 1:
                c = colors[-1]
            else:
                c = lerp_color(colors[i], colors[i + 1], f)
            px[x, y] = c
    return img


def draw_icon(size: int) -> Image.Image:
    s = size / W
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))

    grad = make_gradient(size, size, [INDIGO_LIGHT, PURPLE, PINK])
    mask = Image.new("L", (size, size), 0)
    mdraw = ImageDraw.Draw(mask)
    radius = int(228 * s)
    mdraw.rounded_rectangle([0, 0, size, size], radius=radius, fill=255)
    canvas.paste(grad, (0, 0), mask)

    overlay = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    od = ImageDraw.Draw(overlay)
    cx, cy = size / 2, size * 0.32
    r = size * 0.55
    for i in range(60):
        alpha = int(4 * (1 - i / 60))
        if alpha <= 0:
            continue
        offset = int(i * s * 5)
        od.ellipse(
            [cx - r + offset, cy - r - offset, cx + r + offset, cy + r - offset],
            fill=(255, 255, 255, alpha),
        )
    canvas = Image.alpha_composite(canvas, overlay)

    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    sd = ImageDraw.Draw(shadow)
    clip_w = int(560 * s)
    clip_h = int(620 * s)
    clip_x = (size - clip_w) // 2
    clip_y = int((size - clip_h) // 2 + 40 * s)
    clip_r = int(60 * s)
    for i in range(20, 0, -1):
        alpha = int(80 * (i / 20) * (i / 20))
        off = int(i * s * 1.2)
        sd.rounded_rectangle(
            [clip_x - 2, clip_y + off, clip_x + clip_w + 2, clip_y + clip_h + off],
            radius=clip_r,
            fill=(30, 20, 60, alpha),
        )
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=int(8 * s)))
    canvas = Image.alpha_composite(canvas, shadow)

    d = ImageDraw.Draw(canvas)
    d.rounded_rectangle(
        [clip_x, clip_y, clip_x + clip_w, clip_y + clip_h],
        radius=clip_r,
        fill=PAPER,
    )

    paper_top = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ptd = ImageDraw.Draw(paper_top)
    ptd.rounded_rectangle(
        [clip_x, clip_y, clip_x + clip_w, clip_y + int(clip_h * 0.5)],
        radius=clip_r,
        fill=(255, 255, 255, 40),
    )
    canvas = Image.alpha_composite(canvas, paper_top)
    d = ImageDraw.Draw(canvas)

    clip_w2 = int(280 * s)
    clip_h2 = int(120 * s)
    clip_x2 = (size - clip_w2) // 2
    clip_y2 = clip_y - int(50 * s)
    clip_r2 = int(30 * s)
    for i in range(15, 0, -1):
        alpha = int(60 * (i / 15) * (i / 15))
        off = int(i * s * 0.8)
        d.rounded_rectangle(
            [clip_x2, clip_y2 + off, clip_x2 + clip_w2, clip_y2 + clip_h2 + off],
            radius=clip_r2,
            fill=(30, 20, 60, alpha),
        )
    d.rounded_rectangle(
        [clip_x2, clip_y2, clip_x2 + clip_w2, clip_y2 + clip_h2],
        radius=clip_r2,
        fill=CLIP_DARK,
    )

    hole_w = int(110 * s)
    hole_h = int(28 * s)
    hole_x = (size - hole_w) // 2
    hole_y = clip_y2 + (clip_h2 - hole_h) // 2
    hole_r = int(10 * s)
    d.rounded_rectangle(
        [hole_x, hole_y, hole_x + hole_w, hole_y + hole_h],
        radius=hole_r,
        fill=PAPER,
    )

    line_x1 = clip_x + int(90 * s)
    line_x2 = clip_x + clip_w - int(90 * s)
    line_y_start = clip_y + int(200 * s)
    line_h = max(int(14 * s), 3)
    line_r = line_h // 2
    for i in range(3):
        y = line_y_start + i * int(85 * s)
        end_x = line_x2 if i != 2 else line_x1 + int((line_x2 - line_x1) * 0.6)
        d.rounded_rectangle(
            [line_x1, y, end_x, y + line_h],
            radius=line_r,
            fill=LINE_BLUE,
        )

    return canvas


def save_png(img: Image.Image, name: str, size: int):
    resized = img.resize((size, size), Image.LANCZOS)
    path = os.path.join(OUTPUT_DIR, name)
    resized.save(path, "PNG", optimize=True)
    print(f"saved: {name} ({size}x{size})")


def save_ico(img: Image.Image):
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    path = os.path.join(OUTPUT_DIR, "icon.ico")
    img.save(path, format="ICO", sizes=sizes)
    print("saved: icon.ico")


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
        print("saved: icon.icns")
    finally:
        shutil.rmtree(iconset, ignore_errors=True)


def main():
    master = draw_icon(W)
    save_png(master, "32x32.png", 32)
    save_png(master, "128x128.png", 128)
    save_png(master, "128x128@2x.png", 256)
    save_ico(master)
    save_icns(master)
    print("\nAll icons generated.")


if __name__ == "__main__":
    main()
