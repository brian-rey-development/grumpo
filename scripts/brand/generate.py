"""Builds the cover, social previews, logos, icon and favicons from brand/ and docs/images/.

Needs python3 with Pillow 10 or newer and `pnpm install` in app/, whose Fredoka and
Nunito variable fonts are used so the images match the app. Screenshots come from
docs/images/, which are captured from the UI at a device scale of 2.

    python3 scripts/brand/generate.py
"""
import os

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
BRAND = os.path.join(ROOT, "brand")
SHOTS = os.path.join(ROOT, "docs", "images")
FONTS = os.path.join(ROOT, "app", "node_modules", "@fontsource-variable")
SITE_PUBLIC = os.path.join(ROOT, "site", "public")
APP_PUBLIC = os.path.join(ROOT, "app", "public")

INK = (31, 31, 35)
SURFACE = (255, 246, 234)
RAISED = (255, 254, 251)
SECONDARY = (103, 95, 86)
ORANGE = (255, 138, 61)
RED_TEXT = (196, 30, 30)
RED = (255, 77, 77)
DOT = (31, 31, 35, 20)

SS = 2
COVER_W, COVER_H = 2560, 1280
ICON_ALPHA_FLOOR = 200
ICON_EDGE_ALPHA = 250
MANIFEST_ICON_SIZES = (192, 512)


def font(family, size, weight):
    name = "fredoka-latin-wght-normal.woff2" if family == "display" else "nunito-latin-wght-normal.woff2"
    face = ImageFont.truetype(os.path.join(FONTS, family_dir(family), "files", name), size * SS)
    face.set_variation_by_axes([weight])
    return face


def family_dir(family):
    return "fredoka" if family == "display" else "nunito"


def px(value):
    return round(value * SS)


def app_icon():
    """The squircle with its baked-in soft shadow removed, cropped to the tile."""
    art = Image.open(os.path.join(BRAND, "app-icon-1024.png")).convert("RGBA")
    alpha = art.getchannel("A").point(lambda a: max(0, min(255, (a - ICON_ALPHA_FLOOR) * 255 // (255 - ICON_ALPHA_FLOOR))))
    art.putalpha(alpha)
    return art.crop(square_box(art.getchannel("A").point(lambda a: 255 if a > ICON_EDGE_ALPHA else 0).getbbox()))


def square_box(box):
    """The smallest square around box, centred on it, so resizing never distorts the tile."""
    left, top, right, bottom = box
    side = max(right - left, bottom - top)
    cx, cy = (left + right) // 2, (top + bottom) // 2
    return (cx - side // 2, cy - side // 2, cx - side // 2 + side, cy - side // 2 + side)


def rounded_mask(size, radius):
    mask = Image.new("L", size, 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, size[0] - 1, size[1] - 1), radius=radius, fill=255)
    return mask


def sticker(content, radius, outline, offset):
    """Content clipped to a rounded rect with an ink outline and a hard offset shadow, as in the app."""
    w, h = content.size
    pad = outline + offset
    card = Image.new("RGBA", (w + pad + outline, h + pad + outline), (0, 0, 0, 0))
    draw = ImageDraw.Draw(card)
    draw.rounded_rectangle((offset, offset, w + 2 * outline + offset - 1, h + 2 * outline + offset - 1), radius + outline, fill=INK)
    draw.rounded_rectangle((0, 0, w + 2 * outline - 1, h + 2 * outline - 1), radius + outline, fill=INK)
    card.paste(content, (outline, outline), rounded_mask(content.size, radius))
    return card


def screenshot(name, crop, width):
    shot = Image.open(os.path.join(SHOTS, name)).convert("RGB").crop(crop)
    return shot.resize((width, round(shot.height * width / shot.width)), Image.LANCZOS)


def dot_grid(canvas, box, step, radius):
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)
    x0, y0, x1, y1 = box
    for y in range(y0, y1, step):
        for x in range(x0, x1, step):
            draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=DOT)
    canvas.alpha_composite(layer)


def chip(draw, x, y, label, dot_color):
    face = font("body", 29, 700)
    text_w = draw.textlength(label, font=face)
    w, h = px(60) + text_w, px(62)
    draw.rounded_rectangle((x, y, x + w, y + h), h // 2, fill=RAISED, outline=INK, width=px(3))
    draw.ellipse((x + px(22), y + h // 2 - px(6), x + px(34), y + h // 2 + px(6)), fill=dot_color)
    draw.text((x + px(46), y + h // 2), label, font=face, fill=INK, anchor="lm")
    return x + w + px(14)


def logo_row(canvas, x, y, icon_size, text_size, ink):
    tile = app_icon().resize((px(icon_size), px(icon_size)), Image.LANCZOS)
    canvas.alpha_composite(tile, (x, y))
    draw = ImageDraw.Draw(canvas)
    draw.text((x + px(icon_size * 1.22), y + px(icon_size) // 2), "Grumpo", font=font("display", text_size, 700), fill=ink, anchor="lm")


def cover_text(canvas):
    draw = ImageDraw.Draw(canvas)
    x = px(170)
    logo_row(canvas, x, px(230), 88, 72, INK)
    headline = font("display", 104, 700)
    draw.text((x, px(430)), "Grumpo says no.", font=headline, fill=INK)
    draw.text((x, px(550)), "You get things done.", font=headline, fill=RED_TEXT)
    body = font("body", 38, 500)
    lines = ["A strict, private focus blocker and screen", "time tracker for macOS. Rules hold, unlocks", "cost a pause, and nothing leaves your Mac."]
    for i, line in enumerate(lines):
        draw.text((x, px(738 + i * 56)), line, font=body, fill=SECONDARY)
    cx = x
    for label, color in (("Blocks sites and apps", ORANGE), ("Daily limits", RED), ("100% on device", INK)):
        cx = chip(draw, cx, px(950), label, color)


def cover_visual(canvas):
    window = sticker(screenshot("today.png", (0, 0, 2560, 1490), px(1080)), px(24), px(4), px(16))
    canvas.alpha_composite(window, (px(1340), px(200)))
    block = screenshot("block-schedule.png", (560, 20, 2000, 1560), px(400))
    card = sticker(block, px(24), px(4), px(12)).rotate(4, resample=Image.BICUBIC, expand=True)
    canvas.alpha_composite(card, (px(1200), px(600)))


def cover():
    canvas = Image.new("RGBA", (px(COVER_W), px(COVER_H)), SURFACE + (255,))
    dot_grid(canvas, (px(1240), px(40), px(COVER_W), px(COVER_H)), px(34), px(2.2))
    cover_text(canvas)
    cover_visual(canvas)
    return canvas.resize((COVER_W, COVER_H), Image.LANCZOS).convert("RGB")


def fit_on_surface(image, size):
    scale = min(size[0] / image.width, size[1] / image.height)
    resized = image.resize((round(image.width * scale), round(image.height * scale)), Image.LANCZOS)
    canvas = Image.new("RGB", size, SURFACE)
    canvas.paste(resized, ((size[0] - resized.width) // 2, (size[1] - resized.height) // 2))
    return canvas


def logo(ink):
    face = font("display", 92, 700)
    text_w = ImageDraw.Draw(Image.new("L", (1, 1))).textlength("Grumpo", font=face)
    canvas = Image.new("RGBA", (round(px(112 * 1.22) + text_w + px(8)), px(128)), (0, 0, 0, 0))
    logo_row(canvas, 0, px(8), 112, 92, ink)
    return canvas


def opaque_icon(size):
    """A full-bleed square for apple-touch-icon, which iOS and macOS mask themselves."""
    tile = app_icon()
    inset = round(tile.width * 0.09)
    art = tile.crop((inset, inset, tile.width - inset, tile.height - inset))
    base = Image.new("RGBA", art.size, ORANGE + (255,))
    base.alpha_composite(art)
    return base.convert("RGB").resize((size, size), Image.LANCZOS)


def save(image, *parts):
    path = os.path.join(*parts)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    image.save(path, optimize=True)
    print("wrote", os.path.relpath(path, ROOT))


def favicons(directory):
    icon = app_icon()
    save(icon.resize((32, 32), Image.LANCZOS), directory, "favicon-32.png")
    save(icon.resize((16, 16), Image.LANCZOS), directory, "favicon-16.png")
    save(opaque_icon(180), directory, "apple-touch-icon.png")
    icon.resize((256, 256), Image.LANCZOS).save(os.path.join(directory, "favicon.ico"), sizes=[(16, 16), (32, 32), (48, 48)])
    print("wrote", os.path.relpath(os.path.join(directory, "favicon.ico"), ROOT))


SITE_SHOTS = {"today": "today.png", "rules": "rules.png", "today-limit": "today-limit.png"}
SITE_ART = {
    "grumpo-push": "poses/grumpo-push.png",
    "grumpo-desk": "poses/grumpo-desk.png",
    "grumpo-lock": "poses/grumpo-lock.png",
    "grumpo-shield": "poses/grumpo-shield.png",
    "grumpo-arms-crossed": "grumpo-arms-crossed.png",
    "sticker-hourglass": "stickers/hourglass.png",
    "sticker-laptop": "stickers/laptop.png",
    "sticker-sleep": "stickers/sleep.png",
    "sticker-wave": "stickers/wave.png",
    "sticker-approve": "stickers/approve.png",
}
SHOT_WIDTH = 1600
ART_MAX = 720
WEBP_QUALITY = 84


def save_webp(image, name):
    path = os.path.join(SITE_PUBLIC, "img", name + ".webp")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    image.save(path, "WEBP", quality=WEBP_QUALITY, method=6)
    print("wrote", os.path.relpath(path, ROOT), image.size)


def site_images():
    for name, source in SITE_SHOTS.items():
        shot = Image.open(os.path.join(SHOTS, source)).convert("RGB")
        save_webp(shot.resize((SHOT_WIDTH, round(shot.height * SHOT_WIDTH / shot.width)), Image.LANCZOS), name)
    for name, source in SITE_ART.items():
        art = Image.open(os.path.join(BRAND, source)).convert("RGBA")
        art.thumbnail((ART_MAX, ART_MAX), Image.LANCZOS)
        save_webp(art, name)
    save_webp(app_icon().resize((256, 256), Image.LANCZOS), "icon")


def main():
    art = cover()
    save(art, BRAND, "cover.png")
    save(art.resize((1280, 640), Image.LANCZOS), BRAND, "social-preview.png")
    save(fit_on_surface(art, (1200, 630)), SITE_PUBLIC, "og.png")
    save(logo(INK), BRAND, "logo.png")
    save(logo((255, 244, 230)), BRAND, "logo-on-dark.png")
    save(app_icon().resize((512, 512), Image.LANCZOS), BRAND, "icon.png")
    favicons(APP_PUBLIC)
    favicons(SITE_PUBLIC)
    for size in MANIFEST_ICON_SIZES:
        save(opaque_icon(size), SITE_PUBLIC, f"icon-{size}.png")
    site_images()


if __name__ == "__main__":
    main()
