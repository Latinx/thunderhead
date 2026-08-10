#!/usr/bin/env python3
"""Record a Thunderhead demo and render it to an animated GIF.

Spawns thunderhead in a PTY and drives a scripted scene:
  1. paints a fullscreen ASCII storm (figlet title + generated rain/bolt/ground)
  2. lets the live storm rage over it
  3. transitions into nvim (colorscheme with a colored background, so the
     storm-over-painted-background behavior shows)
  4. quits nvim and keeps capturing

The ANSI stream is rasterized to a live grid at FPS frames per second and each
frame is rendered to pixels with PIL, then saved as a looping GIF.

Usage: python3 tools/make_demo_gif.py [out.gif]
"""

import pty
import os
import sys
import time
import select
import struct
import termios as T
import fcntl
import copy

from PIL import Image, ImageDraw, ImageFont

ROWS, COLS = 40, 120
FPS = 60
CAPTURE_S = 18
BIN = os.path.expanduser("~/.local/bin/thunderhead")
FONT_PATH = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
CELL_W, CELL_H = 10, 20
FONT_PX = 16

ART_PATH = "/tmp/thunderhead_art.txt"
NVIM_FILE = "/tmp/thunderhead_demo.txt"
ART_ASSET = os.path.join(os.path.dirname(os.path.abspath(__file__)), "storm_art.txt")

# --- xterm 256-color approximation (16 base + 6x6x6 cube + grayscale) --------
BASE16 = [
    (0, 0, 0), (205, 0, 0), (0, 205, 0), (205, 205, 0),
    (0, 0, 238), (205, 0, 205), (0, 205, 205), (229, 229, 229),
    (127, 127, 127), (255, 0, 0), (0, 255, 0), (255, 255, 0),
    (92, 92, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
]


def xterm_rgb(n):
    if n < 16:
        return BASE16[n]
    if n < 232:
        n -= 16
        return tuple(round(v * 255 / 5) for v in ((n // 36) % 6, (n // 6) % 6, n % 6))
    g = round((n - 232) * 255 / 23)
    return (g, g, g)


# --- fullscreen ASCII art ----------------------------------------------------
# A redraw of "Starry Night by Vincent van Gogh in ASCII" by Veni, Vidi, ASCII
# (see README credits). Kept as a repo asset so the demo shows the same piece
# every run.


def write_scene_files():
    with open(ART_ASSET) as src, open(ART_PATH, "w") as dst:
        dst.write(src.read())
    with open(NVIM_FILE, "w") as f:
        f.write(
            "// thunderhead.rs — the storm lives in your terminal\n"
            "fn main() {\n"
            "    let mut storm = Storm::new();\n"
            "    loop {\n"
            "        storm.tick(dt);\n"
            "        render(&storm);\n"
            "    }\n"
            "}\n"
        )


# --- ANSI stream -> grid rasterizer ------------------------------------------
class Raster:
    def __init__(self):
        self.grid = [[(" ", (229, 229, 229), (0, 0, 0), False)] * COLS for _ in range(ROWS)]
        self.r = 0
        self.c = 0
        self.fg = (229, 229, 229)
        self.bg = (0, 0, 0)
        self.bold = False
        self.osc = False
        self.esc = 0  # 0=ground, 1=esc, 2=csi
        self.params = b""

    def _clear_cell(self, r, c):
        self.grid[r][c] = (" ", self.fg, self.bg, self.bold)

    def _move(self, r, c):
        self.r = max(0, min(ROWS - 1, r))
        self.c = max(0, min(COLS - 1, c))

    def feed(self, data: bytes):
        for b in data:
            if self.osc:
                if b == 0x07:
                    self.osc = False
                continue
            if self.esc == 0:
                if b == 0x1B:
                    self.esc = 1
                elif b == 0x0D:
                    self.c = 0
                elif b == 0x0A:
                    self.r = min(ROWS - 1, self.r + 1)
                elif b == 0x09:
                    self.c = min(COLS - 1, ((self.c // 8) + 1) * 8)
                elif b == 0x08:
                    self.c = max(0, self.c - 1)
                elif 0x20 <= b < 0x7F:
                    self.grid[self.r][self.c] = (chr(b), self.fg, self.bg, self.bold)
                    self.c = min(COLS - 1, self.c + 1)
                continue
            if self.esc == 1:
                if b == ord("["):
                    self.esc = 2
                    self.params = b""
                elif b == ord("]"):
                    self.osc = True
                    self.esc = 0
                else:
                    self.esc = 0
                continue
            # CSI
            if b == ord("?"):
                self.params += b"?"
                continue
            if 0x30 <= b <= 0x3F:
                self.params += bytes([b])
                continue
            self._csi(b)
            self.esc = 0

    def _csi(self, final):
        s = self.params.decode("ascii", "replace")
        if s.startswith("?"):
            return  # private modes (?25, ?1049, ...) don't affect pixels
        nums = [int(x) if x else 0 for x in s.split(";")]
        if final == ord("m"):
            i = 0
            while i < len(nums):
                n = nums[i]
                if n == 0:
                    self.fg, self.bg, self.bold = (229, 229, 229), (0, 0, 0), False
                elif n == 1:
                    self.bold = True
                elif n == 22:
                    self.bold = False
                elif 30 <= n <= 37:
                    self.fg = BASE16[n - 30]
                elif 90 <= n <= 97:
                    self.fg = BASE16[n - 90 + 8]
                elif n == 39:
                    self.fg = (229, 229, 229)
                elif 40 <= n <= 47:
                    self.bg = BASE16[n - 40]
                elif 100 <= n <= 107:
                    self.bg = BASE16[n - 100 + 8]
                elif n == 49:
                    self.bg = (0, 0, 0)
                elif n == 38 or n == 48:
                    if i + 1 < len(nums) and nums[i + 1] == 5 and i + 2 < len(nums):
                        rgb = xterm_rgb(nums[i + 2])
                        if n == 38:
                            self.fg = rgb
                        else:
                            self.bg = rgb
                        i += 2
                    elif i + 1 < len(nums) and nums[i + 1] == 2 and i + 4 < len(nums):
                        rgb = (nums[i + 2], nums[i + 3], nums[i + 4])
                        if n == 38:
                            self.fg = rgb
                        else:
                            self.bg = rgb
                        i += 4
                i += 1
        elif final == ord("H") or final == ord("f"):
            r = nums[0] if nums else 1
            c = nums[1] if len(nums) > 1 else 1
            self._move(r - 1, c - 1)
        elif final == ord("A"):
            self._move(self.r - (nums[0] or 1), self.c)
        elif final == ord("B"):
            self._move(self.r + (nums[0] or 1), self.c)
        elif final == ord("C"):
            self._move(self.r, self.c + (nums[0] or 1))
        elif final == ord("D"):
            self._move(self.r, self.c - (nums[0] or 1))
        elif final == ord("G"):
            self._move(self.r, (nums[0] or 1) - 1)
        elif final == ord("J"):
            mode = nums[0] if nums else 0
            if mode == 2:
                for r in range(ROWS):
                    for c in range(COLS):
                        self._clear_cell(r, c)
            elif mode == 0:
                for c in range(self.c, COLS):
                    self._clear_cell(self.r, c)
                for r in range(self.r + 1, ROWS):
                    for c in range(COLS):
                        self._clear_cell(r, c)
            else:
                for c in range(0, self.c + 1):
                    self._clear_cell(self.r, c)
                for r in range(0, self.r):
                    for c in range(COLS):
                        self._clear_cell(r, c)
        elif final == ord("K"):
            mode = nums[0] if nums else 0
            if mode == 2:
                for c in range(COLS):
                    self._clear_cell(self.r, c)
            elif mode == 0:
                for c in range(self.c, COLS):
                    self._clear_cell(self.r, c)
            else:
                for c in range(0, self.c + 1):
                    self._clear_cell(self.r, c)
        # everything else is ignored — the diff renderer re-positions every
        # run, so skipped sequences self-heal.


def capture_frames():
    write_scene_files()
    pid, fd = pty.fork()
    if pid == 0:
        os.execv(BIN, [BIN])
    fcntl.ioctl(fd, T.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))

    raster = Raster()
    frames = []
    start = time.time()
    next_snap = start + 1.0 / FPS
    end = start + CAPTURE_S
    sent = {}  # action -> time it was sent

    def send(t, text):
        if t not in sent and time.time() - start >= t:
            os.write(fd, text.encode() + b"\r")
            sent[t] = True

    while time.time() < end:
        send(1.2, "cat " + ART_PATH)                       # fullscreen art
        send(4.5, "nvim " + NVIM_FILE)                       # their real nvim
        send(12.0, "\x1b:q")                                # quit nvim
        r, _, _ = select.select([fd], [], [], 0.02)
        if r:
            try:
                data = os.read(fd, 65536)
            except OSError:
                break
            if not data:
                break
            raster.feed(data)
        now = time.time()
        if now >= next_snap:
            frames.append(copy.deepcopy(raster.grid))
            next_snap = now + 1.0 / FPS
    os.close(fd)
    return frames


def render_gif(frames, out_path):
    font = ImageFont.truetype(FONT_PATH, FONT_PX)
    glyph_cache = {}
    w, h = COLS * CELL_W, ROWS * CELL_H
    gif = []
    for frame in frames:
        img = Image.new("RGB", (w, h), (0, 0, 0))
        d = ImageDraw.Draw(img)
        for r, row in enumerate(frame):
            for c, (ch, fg, bg, bold) in enumerate(row):
                x, y = c * CELL_W, r * CELL_H
                if bg != (0, 0, 0):
                    d.rectangle([x, y, x + CELL_W - 1, y + CELL_H - 1], fill=bg)
                if ch == " ":
                    continue
                key = (ch, fg, bold)
                tile = glyph_cache.get(key)
                if tile is None:
                    tile = Image.new("RGBA", (CELL_W, CELL_H), (0, 0, 0, 0))
                    td = ImageDraw.Draw(tile)
                    td.text((0, -2), ch, font=font, fill=(*fg, 255))
                    glyph_cache[key] = tile
                img.paste(tile, (x, y), tile)
        gif.append(img)
    gif[0].save(
        out_path,
        save_all=True,
        append_images=gif[1:],
        duration=1000 // FPS,
        loop=0,
        optimize=True,
    )


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "demo.gif"
    print("capturing thunderhead demo...")
    frames = capture_frames()
    print(f"captured {len(frames)} frames")
    print("rendering gif...")
    render_gif(frames, out)
    print(f"wrote {out} ({os.path.getsize(out) // 1024} KB)")


if __name__ == "__main__":
    main()
