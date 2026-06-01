from __future__ import annotations

import array
import json
import math
import shutil
import subprocess
import wave
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
PYTHON = Path(r"C:\Users\Admin\AppData\Local\Python\pythoncore-3.14-64\python.exe")


def tool(name: str) -> str:
    found = shutil.which(name)
    if not found:
        raise SystemExit(f"Missing required tool on PATH: {name}")
    return found


INKSCAPE = tool("inkscape")
FFMPEG = tool("ffmpeg")


def run(args: list[str]) -> None:
    print(" ".join(str(a) for a in args), flush=True)
    subprocess.run(args, check=True)


def svg_tag(name: str, **attrs: object) -> str:
    rendered = []
    for key, value in attrs.items():
        if value is None:
            continue
        attr = key.replace("_", "-")
        escaped = str(value).replace("&", "&amp;").replace('"', "&quot;")
        rendered.append(f'{attr}="{escaped}"')
    return f"<{name} {' '.join(rendered)}/>"


def rect(x: int | float, y: int | float, w: int | float, h: int | float, fill: str, **attrs: object) -> str:
    return svg_tag("rect", x=x, y=y, width=w, height=h, fill=fill, **attrs)


def circle(cx: int | float, cy: int | float, r: int | float, fill: str, **attrs: object) -> str:
    return svg_tag("circle", cx=cx, cy=cy, r=r, fill=fill, **attrs)


def ellipse(cx: int | float, cy: int | float, rx: int | float, ry: int | float, fill: str, **attrs: object) -> str:
    return svg_tag("ellipse", cx=cx, cy=cy, rx=rx, ry=ry, fill=fill, **attrs)


def polygon(points: list[tuple[int | float, int | float]], fill: str, **attrs: object) -> str:
    pts = " ".join(f"{x},{y}" for x, y in points)
    return svg_tag("polygon", points=pts, fill=fill, **attrs)


def line(x1: int | float, y1: int | float, x2: int | float, y2: int | float, stroke: str, **attrs: object) -> str:
    return svg_tag("line", x1=x1, y1=y1, x2=x2, y2=y2, stroke=stroke, **attrs)


def text(x: int | float, y: int | float, value: str, fill: str, size: int, **attrs: object) -> str:
    safe = value.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    attr = {
        "x": x,
        "y": y,
        "fill": fill,
        "font-size": size,
        "font-family": "Impact, Consolas, monospace",
        "font-weight": "700",
        "text-anchor": attrs.pop("text_anchor", "middle"),
        "dominant-baseline": attrs.pop("dominant_baseline", "middle"),
        **attrs,
    }
    rendered = " ".join(f'{k}="{str(v).replace("&", "&amp;").replace(chr(34), "&quot;")}"' for k, v in attr.items())
    return f"<text {rendered}>{safe}</text>"


def svg_doc(width: int, height: int, body: list[str], defs: str = "") -> str:
    return "\n".join(
        [
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" shape-rendering="crispEdges">',
            defs,
            *body,
            "</svg>",
        ]
    )


def write_svg(path: Path, width: int, height: int, body: list[str], defs: str = "") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    content = svg_doc(width, height, body, defs)
    if path.exists() and path.read_text(encoding="utf-8") == content:
        return
    path.write_text(content, encoding="utf-8")


def export_png(svg_path: Path, png_path: Path, width: int, height: int) -> None:
    png_path.parent.mkdir(parents=True, exist_ok=True)
    if png_path.exists() and png_path.stat().st_mtime >= svg_path.stat().st_mtime:
        print(f"skip {png_path}", flush=True)
        return
    run(
        [
            INKSCAPE,
            str(svg_path),
            "--export-type=png",
            f"--export-filename={png_path}",
            f"--export-width={width}",
            f"--export-height={height}",
        ]
    )


def save_art(rel_png: str, width: int, height: int, body: list[str], defs: str = "") -> None:
    png_path = ASSETS / rel_png
    svg_path = png_path.with_suffix(".svg")
    write_svg(svg_path, width, height, body, defs)
    export_png(svg_path, png_path, width, height)


def save_sheet(rel_png: str, frame_w: int, frame_h: int, frames: list[list[str]], defs: str = "") -> None:
    png_path = ASSETS / rel_png
    stem = png_path.with_suffix("")
    sheet_body: list[str] = []
    for index, frame in enumerate(frames, start=1):
        frame_png = stem.with_name(f"{stem.name}-frame-{index:02d}.png")
        frame_svg = frame_png.with_suffix(".svg")
        write_svg(frame_svg, frame_w, frame_h, frame, defs)
        export_png(frame_svg, frame_png, frame_w, frame_h)
        sheet_body.append(f'<g transform="translate({(index - 1) * frame_w},0)">')
        sheet_body.extend(frame)
        sheet_body.append("</g>")
    sheet_w = frame_w * len(frames)
    sheet_svg = png_path.with_suffix(".svg")
    write_svg(sheet_svg, sheet_w, frame_h, sheet_body, defs)
    export_png(sheet_svg, png_path, sheet_w, frame_h)


def beveled_box(w: int, h: int, color: str, light: str, dark: str, inset: int = 3) -> list[str]:
    return [
        rect(0, 0, w, h, dark),
        rect(1, 1, w - 2, h - 2, color),
        rect(2, 2, w - 4, inset, light),
        rect(2, 2, inset, h - 4, light),
        rect(2, h - inset - 2, w - 4, inset, dark),
        rect(w - inset - 2, 2, inset, h - 4, dark),
        rect(8, 7, w - 16, h - 14, color, opacity="0.82"),
    ]


def paddle(w: int, h: int, expanded: bool = False) -> list[str]:
    mid = "#d7e7ff"
    return [
        rect(4, 5, w - 8, h - 8, "#234c7d"),
        rect(8, 2, w - 16, 5, "#f5fbff"),
        rect(8, h - 7, w - 16, 5, "#102541"),
        rect(0, 7, 8, h - 12, "#cf2835"),
        rect(w - 8, 7, 8, h - 12, "#cf2835"),
        rect(10, 8, w - 20, 8, mid),
        rect(18, 12, w - 36, 4, "#75a8d9"),
        rect(w // 2 - 5, 5, 10, h - 7, "#fff27a" if expanded else "#b5d8ff"),
        rect(12, 4, 4, h - 8, "#ffffff", opacity="0.65"),
        rect(w - 16, 4, 4, h - 8, "#ffffff", opacity="0.65"),
    ]


def ball() -> list[str]:
    return [
        circle(8, 8, 7, "#2f66ff"),
        rect(3, 5, 10, 7, "#64d7ff"),
        rect(5, 3, 5, 3, "#ffffff"),
        rect(10, 11, 3, 3, "#1430a4"),
    ]


def border_frame() -> list[str]:
    # Portrait playfield frame: walls on top/left/right, open bottom.
    w, h, t = 600, 800, 20
    body = [
        rect(0, 0, w, t, "#23374f"),
        rect(0, 0, t, h, "#23374f"),
        rect(w - t, 0, t, h, "#23374f"),
        rect(4, 4, w - 8, 5, "#6aa0c8"),
        rect(4, 11, w - 8, 5, "#102238"),
        rect(4, 22, 5, h - 26, "#6aa0c8"),
        rect(11, 22, 5, h - 26, "#102238"),
        rect(w - 16, 22, 5, h - 26, "#6aa0c8"),
        rect(w - 9, 22, 5, h - 26, "#102238"),
    ]
    for x in range(40, w - 20, 40):
        body.append(rect(x, 6, 8, 8, "#c4d8f3"))
    for y in range(40, h - 20, 40):
        body.append(rect(6, y, 8, 8, "#c4d8f3"))
        body.append(rect(w - 14, y, 8, 8, "#c4d8f3"))
    return body


BRICKS = {
    "white": ("#f7f7e8", "#ffffff", "#9fa2aa"),
    "orange": ("#f07a22", "#ffc069", "#8f2d11"),
    "cyan": ("#26d9d9", "#9bffff", "#0c6a77"),
    "green": ("#46c34a", "#a4ff97", "#176a21"),
    "red": ("#e2373f", "#ff9299", "#7b1018"),
    "blue": ("#2e62d6", "#88b7ff", "#132f7e"),
    "pink": ("#ed5bb7", "#ffb4e4", "#7e1e61"),
    "yellow": ("#f3d53b", "#fff493", "#8b6b12"),
}


def brick(color_name: str) -> list[str]:
    color, light, dark = BRICKS[color_name]
    return beveled_box(56, 28, color, light, dark)


def silver_brick(cracks: int) -> list[str]:
    body = beveled_box(56, 28, "#bbc3cc", "#f4fbff", "#5d6670")
    body.append(rect(8, 8, 40, 5, "#dce5ee", opacity="0.72"))
    for i in range(cracks):
        x = 14 + i * 10
        body.extend(
            [
                line(x, 7, x + 5, 13, "#323840", stroke_width=2),
                line(x + 5, 13, x + 1, 21, "#323840", stroke_width=2),
            ]
        )
    return body


def capsule(letter: str, color: str) -> list[str]:
    return [
        rect(4, 2, 24, 12, "#ffffff"),
        circle(8, 8, 6, color),
        circle(24, 8, 6, color),
        rect(8, 2, 16, 12, color),
        rect(7, 4, 18, 2, "#ffffff", opacity="0.5"),
        text(16, 8.8, letter, "#07111f", 11),
    ]


def warp_gate() -> list[str]:
    return [
        rect(0, 0, 32, 80, "#0b1724"),
        rect(2, 2, 28, 76, "#375a7d"),
        rect(7, 8, 18, 64, "#151a38"),
        rect(10, 12, 12, 56, "#2efff0", opacity="0.75"),
        rect(13, 18, 6, 44, "#ffffff", opacity="0.5"),
        rect(4, 6, 4, 68, "#9ecaff"),
        rect(24, 6, 4, 68, "#102b46"),
    ]


def spawn_gate(open_pixels: int) -> list[str]:
    gap = open_pixels
    return [
        rect(0, 0, 48, 24, "#17283f"),
        rect(2, 2, 44, 20, "#39607f"),
        rect(4, 6, 40, 12, "#101827"),
        rect(4, 6, max(0, 20 - gap), 12, "#9ecaff"),
        rect(24 + gap, 6, max(0, 20 - gap), 12, "#9ecaff"),
        rect(6, 4, 36, 2, "#d9f4ff"),
    ]


def enemy_pyramid(frame: int) -> list[str]:
    dx = [0, 1, 0, -1][frame]
    return [
        polygon([(16, 2), (30, 28), (2, 28)], "#dd3954"),
        polygon([(16, 6), (24, 24), (8, 24)], "#ff9aa7"),
        rect(10 + dx, 17, 12, 5, "#fff279"),
        rect(14 + dx, 18, 4, 3, "#111827"),
    ]


def enemy_molecule(frame: int) -> list[str]:
    off = [0, 2, 0, -2][frame]
    return [
        line(10, 10, 22, 22, "#b8edff", stroke_width=3),
        line(22, 10, 10, 22, "#b8edff", stroke_width=3),
        circle(10 + off, 10, 6, "#56d3ff"),
        circle(22, 10 + off, 6, "#ff5f9f"),
        circle(10, 22 - off, 6, "#fff279"),
        circle(22 - off, 22, 6, "#74ff71"),
        circle(16, 16, 5, "#ffffff"),
    ]


def enemy_cube(frame: int) -> list[str]:
    shade = ["#54e0a8", "#7bffc5", "#54e0a8", "#35b987"][frame]
    return [
        polygon([(7, 8), (17, 3), (27, 8), (17, 13)], "#d4fff1"),
        polygon([(7, 8), (17, 13), (17, 29), (7, 23)], shade),
        polygon([(17, 13), (27, 8), (27, 23), (17, 29)], "#16846c"),
        polygon([(9, 10), (17, 6), (25, 10), (17, 14)], "#ffffff", opacity="0.55"),
    ]


def doh(frame: int) -> list[str]:
    mouth_h = [12, 24, 18, 8][frame]
    eye_y = [50, 48, 50, 52][frame]
    body = [
        rect(25, 12, 78, 116, "#7a3e72"),
        rect(17, 28, 94, 92, "#b05ea3"),
        rect(25, 20, 78, 8, "#e6a0db"),
        rect(25, 120, 78, 8, "#532650"),
        rect(11, 56, 14, 48, "#7a3e72"),
        rect(103, 56, 14, 48, "#7a3e72"),
        circle(43, eye_y, 13, "#fff3cc"),
        circle(85, eye_y, 13, "#fff3cc"),
        rect(39, eye_y - 2, 8, 8, "#111827"),
        rect(81, eye_y - 2, 8, 8, "#111827"),
        rect(44, 82, 40, mouth_h, "#271322"),
        rect(48, 86, 32, 4, "#f4d0ee", opacity="0.75"),
        rect(35, 35, 18, 5, "#5e2f58"),
        rect(75, 35, 18, 5, "#5e2f58"),
    ]
    return body


def burst(w: int, h: int, frame: int, color: str) -> list[str]:
    cx, cy = w // 2, h // 2
    radius = 4 + frame * 5
    body = [circle(cx, cy, max(1, radius), color, opacity=max(0.15, 0.9 - frame * 0.16))]
    for i in range(8):
        angle = math.tau * i / 8
        x1 = cx + math.cos(angle) * radius
        y1 = cy + math.sin(angle) * radius
        x2 = cx + math.cos(angle) * (radius + 8 + frame * 2)
        y2 = cy + math.sin(angle) * (radius + 8 + frame * 2)
        body.append(line(round(x1, 2), round(y1, 2), round(x2, 2), round(y2, 2), "#ffffff", stroke_width=2, opacity=max(0.1, 0.8 - frame * 0.12)))
    return body


def ui_screen(title: str, subtitle: str, accent: str) -> list[str]:
    # Portrait full-screen art; smaller heading so long titles fit the 600px width.
    w, h = 600, 800
    cx = w // 2
    body = [
        rect(0, 0, w, h, "#08101d"),
        rect(24, 24, w - 48, h - 48, "#111d31"),
        rect(40, 40, w - 80, h - 80, "#07111f"),
    ]
    for y in range(96, h - 60, 46):
        body.append(rect(72, y, w - 144, 3, "#243b5a"))
    body.extend(
        [
            text(cx, 300, title, accent, 64),
            text(cx, 380, subtitle, "#f7fbff", 26),
            rect(cx - 190, 470, 380, 18, accent),
            rect(cx - 150, 505, 300, 12, "#ffffff"),
            rect(cx - 100, 535, 200, 8, "#6aa0c8"),
        ]
    )
    return body


def generate_visuals() -> None:
    save_art("sprites/vaus/vaus.png", 96, 24, paddle(96, 24))
    save_art("sprites/vaus/vaus-expanded.png", 160, 24, paddle(160, 24, expanded=True))
    save_art("sprites/vaus/vaus-life-icon.png", 48, 12, paddle(48, 12))
    save_art("sprites/ball/ball.png", 16, 16, ball())
    save_art("sprites/playfield/border-frame.png", 600, 800, border_frame())
    save_art("sprites/playfield/warp-gate.png", 32, 80, warp_gate())
    save_sheet("sprites/playfield/spawn-gate.png", 48, 24, [spawn_gate(v) for v in [0, 8, 18]])

    for name in BRICKS:
        save_art(f"sprites/bricks/brick-{name}.png", 56, 28, brick(name))
    save_sheet("sprites/bricks/brick-silver.png", 56, 28, [silver_brick(i) for i in range(4)])
    save_art("sprites/bricks/brick-gold.png", 56, 28, beveled_box(56, 28, "#d99b22", "#fff093", "#7a5310"))

    capsule_defs = {
        "c-catch": ("C", "#72d6ff"),
        "l-laser": ("L", "#ff4d55"),
        "e-expand": ("E", "#72ff8a"),
        "d-disruption": ("D", "#d46dff"),
        "s-slow": ("S", "#fff070"),
        "b-break": ("B", "#ff9f42"),
        "p-player": ("P", "#ffffff"),
    }
    for slug, (letter, color) in capsule_defs.items():
        save_art(f"sprites/capsules/capsule-{slug}.png", 32, 16, capsule(letter, color))

    save_art(
        "sprites/weapons/laser-bolt.png",
        8,
        24,
        [rect(3, 0, 2, 24, "#ffffff"), rect(2, 2, 4, 20, "#ffef5f"), rect(1, 5, 6, 14, "#ff3344", opacity="0.8")],
    )
    save_sheet("sprites/enemies/enemy-pyramid.png", 32, 32, [enemy_pyramid(i) for i in range(4)])
    save_sheet("sprites/enemies/enemy-molecule.png", 32, 32, [enemy_molecule(i) for i in range(4)])
    save_sheet("sprites/enemies/enemy-cube.png", 32, 32, [enemy_cube(i) for i in range(4)])
    save_sheet("sprites/boss/doh.png", 128, 144, [doh(i) for i in range(4)])

    save_sheet("vfx/capsule-catch-flash.png", 64, 64, [burst(64, 64, i, "#7df7ff") for i in range(5)])
    save_sheet("vfx/laser-impact.png", 32, 32, [burst(32, 32, i, "#ffef5f") for i in range(4)])
    save_sheet("vfx/enemy-destroy-burst.png", 48, 48, [burst(48, 48, i, "#ff65a8") for i in range(5)])
    save_art("vfx/ball-trail.png", 16, 16, [circle(8, 8, 7, "#4aa3ff", opacity="0.25"), circle(8, 8, 4, "#ffffff", opacity="0.35")])
    save_sheet("vfx/doh-defeat-explosion.png", 160, 160, [burst(160, 160, i, "#ff8a2a") for i in range(6)])

    save_art("ui/title-screen.png", 600, 800, ui_screen("ARKANOID", "PRESS START", "#ff5f70"))
    save_art("ui/round-ready-banner.png", 320, 64, [rect(0, 0, 320, 64, "#101827"), rect(4, 4, 312, 56, "#24405f"), text(160, 32, "ROUND 01 READY", "#ffffff", 33)])
    save_art("ui/intro-story-screen.png", 600, 800, ui_screen("VAUS ESCAPES", "A DIMENSIONAL FORTRESS", "#72d6ff"))
    save_art("ui/victory-screen.png", 600, 800, ui_screen("VICTORY", "DOH IS DEFEATED", "#fff070"))
    save_art(
        "ui/high-score-table.png",
        480,
        400,
        [
            rect(0, 0, 480, 400, "#07111f"),
            rect(10, 10, 460, 380, "#1c2f4a"),
            text(240, 52, "HIGH SCORES", "#fff070", 42),
            *[rect(70, 98 + i * 44, 340, 2, "#6aa0c8") for i in range(6)],
            *[text(142, 122 + i * 44, f"{i + 1:02d}", "#ffffff", 24) for i in range(5)],
            *[text(280, 122 + i * 44, f"{50000 - i * 5000}", "#72d6ff", 24) for i in range(5)],
        ],
    )


SFX = {
    "wall-bounce": ("aevalsrc=0.28*sin(2*PI*820*t)*exp(-34*t):s=44100:d=0.11", 0.11),
    "paddle-bounce": ("aevalsrc=0.32*sin(2*PI*(520+900*t)*t)*exp(-20*t):s=44100:d=0.16", 0.16),
    "brick-break": ("aevalsrc=(0.22*sin(2*PI*300*t)+0.18*(2*random(0)-1))*exp(-16*t):s=44100:d=0.22", 0.22),
    "ball-lost": ("aevalsrc=0.35*sin(2*PI*(460-260*t)*t)*exp(-3.8*t):s=44100:d=0.8", 0.8),
    "hard-brick-clink": ("aevalsrc=(0.28*sin(2*PI*1250*t)+0.18*sin(2*PI*1850*t))*exp(-38*t):s=44100:d=0.14", 0.14),
    "ball-speedup": ("aevalsrc=0.22*sin(2*PI*(500+1700*t)*t)*exp(-4*t):s=44100:d=0.42", 0.42),
    "capsule-catch": ("aevalsrc=(0.24*sin(2*PI*620*t)+0.18*sin(2*PI*930*t))*exp(-8*t):s=44100:d=0.32", 0.32),
    "laser-fire": ("aevalsrc=0.22*sin(2*PI*(1300+2600*t)*t)*exp(-30*t):s=44100:d=0.12", 0.12),
    "expand": ("aevalsrc=0.24*sin(2*PI*(260+520*t)*t)*exp(-3.6*t):s=44100:d=0.55", 0.55),
    "multiball": ("aevalsrc=(0.18*sin(2*PI*440*t)+0.18*sin(2*PI*660*t)+0.16*sin(2*PI*880*t))*exp(-4*t):s=44100:d=0.55", 0.55),
    "slow": ("aevalsrc=0.25*sin(2*PI*(640-420*t)*t)*exp(-5*t):s=44100:d=0.48", 0.48),
    "extra-life": ("aevalsrc=(0.18*sin(2*PI*660*t)+0.18*sin(2*PI*880*t)+0.18*sin(2*PI*1320*t))*exp(-2.8*t):s=44100:d=0.72", 0.72),
    "warp-gate-open": ("aevalsrc=(0.18*sin(2*PI*(180+620*t)*t)+0.12*(2*random(0)-1))*exp(-2.2*t):s=44100:d=0.85", 0.85),
    "enemy-spawn": ("aevalsrc=(0.20*sin(2*PI*(900-300*t)*t)+0.10*(2*random(0)-1))*exp(-8*t):s=44100:d=0.34", 0.34),
    "enemy-destroyed": ("aevalsrc=(0.18*sin(2*PI*220*t)+0.24*(2*random(0)-1))*exp(-12*t):s=44100:d=0.28", 0.28),
}


def generate_sfx() -> None:
    sfx_dir = ASSETS / "audio" / "sfx"
    wav_dir = sfx_dir / "source_wav"
    wav_dir.mkdir(parents=True, exist_ok=True)
    manifest = {}
    for name, (source, duration) in SFX.items():
        wav_path = wav_dir / f"{name}.wav"
        ogg_path = sfx_dir / f"{name}.ogg"
        run([FFMPEG, "-y", "-f", "lavfi", "-i", source, "-t", str(duration), "-ar", "44100", "-ac", "1", str(wav_path)])
        run([FFMPEG, "-y", "-i", str(wav_path), "-c:a", "libvorbis", "-q:a", "4", str(ogg_path)])
        manifest[f"{name}.ogg"] = {"source": source, "duration_seconds": duration}
    (sfx_dir / "sfx_sources.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")


SAMPLE_RATE = 44100

# Short jingles as note lists: (frequency_hz, start_s, duration_s, amplitude).
# Each is kept well under the 15s music cap.
MUSIC = {
    # Ascending fanfare on a round clear.
    "round-clear": (
        [
            (523.25, 0.00, 0.16, 0.85),
            (659.25, 0.16, 0.16, 0.85),
            (783.99, 0.32, 0.16, 0.85),
            (1046.50, 0.48, 0.16, 0.85),
            (1318.51, 0.64, 0.16, 0.85),
            (1567.98, 0.80, 0.16, 0.85),
            (1046.50, 0.96, 0.60, 0.50),
            (1318.51, 0.96, 0.60, 0.50),
            (1567.98, 0.96, 0.60, 0.50),
        ],
        1.7,
    ),
    # Descending, minor-key sting on game over, settling on a low chord.
    "game-over": (
        [
            (440.00, 0.00, 0.30, 0.85),
            (349.23, 0.35, 0.30, 0.85),
            (293.66, 0.70, 0.30, 0.85),
            (220.00, 1.05, 0.50, 0.85),
            (220.00, 1.65, 1.40, 0.50),
            (164.81, 1.65, 1.40, 0.45),
            (130.81, 1.65, 1.40, 0.45),
        ],
        3.2,
    ),
}


def synth(notes: list[tuple[float, float, float, float]], total: float) -> array.array:
    """Renders a note list to mono 16-bit PCM with a soft attack and exponential decay."""
    n = int(total * SAMPLE_RATE)
    buf = array.array("h", [0] * n)
    for freq, start, dur, amp in notes:
        s = int(start * SAMPLE_RATE)
        e = min(n, int((start + dur) * SAMPLE_RATE))
        for i in range(s, e):
            t = (i - s) / SAMPLE_RATE
            env = min(1.0, t / 0.008) * math.exp(-3.2 * t)
            val = amp * env * math.sin(2 * math.pi * freq * t)
            mixed = buf[i] + int(val * 32767 * 0.5)
            buf[i] = max(-32768, min(32767, mixed))
    return buf


def generate_music() -> None:
    music_dir = ASSETS / "audio" / "music"
    wav_dir = music_dir / "source_wav"
    wav_dir.mkdir(parents=True, exist_ok=True)
    for name, (notes, total) in MUSIC.items():
        assert total <= 15, f"{name} exceeds the 15s music cap"
        wav_path = wav_dir / f"{name}.wav"
        ogg_path = music_dir / f"{name}.ogg"
        with wave.open(str(wav_path), "w") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SAMPLE_RATE)
            w.writeframes(synth(notes, total).tobytes())
        run([FFMPEG, "-y", "-i", str(wav_path), "-c:a", "libvorbis", "-q:a", "4", str(ogg_path)])


def main() -> None:
    print(f"Using Python: {PYTHON}", flush=True)
    generate_visuals()
    generate_sfx()
    generate_music()
    print("Generated sprites, VFX, UI PNG art, SFX, and music.", flush=True)


if __name__ == "__main__":
    main()
