#!/usr/bin/env python3
"""Wheel spy: shows exactly what the terminal sends when you scroll.

Usage:  python3 tools/wheel_spy.py
Run it TWICE:
  1) directly in your terminal (outside thunderhead)
  2) inside thunderhead (after launching thunderhead, run it in the shell)

In each run, scroll up and down a few times during the two 5-second
windows. Copy the full output back.
"""
import os
import select
import sys
import termios
import time
import tty


def decode(b: bytes) -> str:
    parts = []
    i = 0
    while i < len(b):
        c = b[i]
        if c == 0x1B:
            if i + 1 < len(b) and b[i + 1] == ord("["):
                # CSI: ESC [ <params> <final byte 0x40..0x7E>
                j = i + 2
                while j < len(b) and j - i < 48:
                    if 0x40 <= b[j] <= 0x7E:
                        break
                    j += 1
                if j < len(b):
                    final = chr(b[j])
                    body = b[i + 2 : j].decode("latin1")
                    if body.startswith("<") and final in "Mm":
                        m = body[1:].split(";")
                        if len(m) == 3 and m[0] in ("64", "65"):
                            what = "WHEEL_UP" if m[0] == "64" else "WHEEL_DOWN"
                            parts.append(f"{what} at x={m[1]} y={m[2]} ({'press' if final == 'M' else 'release'})")
                        else:
                            parts.append(f"mouse b={m[0]} x={m[1]} y={m[2]}")
                    else:
                        parts.append(f"CSI {body!r}{final}")
                    i = j + 1
                    continue
            else:
                # two-byte ESC X
                if i + 1 < len(b):
                    parts.append(f"ESC {chr(b[i + 1])}")
                    i += 2
                    continue
        if c < 0x20 or c == 0x7F:
            parts.append(f"<{c:02x}>")
        else:
            parts.append(chr(c))
        i += 1
    return " ".join(parts)


def window(fd: int, seconds: float) -> bytes:
    buf = b""
    t0 = time.time()
    while time.time() - t0 < seconds:
        r, _, _ = select.select([fd], [], [], 0.1)
        if r:
            try:
                data = os.read(fd, 4096)
            except OSError:
                break
            if data:
                buf += data
    return buf


def main() -> int:
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        out = sys.stdout.write
        flush = sys.stdout.flush

        out("\x1b[2J\x1b[H\x1b[?25l")
        out("=== WHEEL SPY ===\n")
        out("terminal size: %d x %d\n" % (os.get_terminal_size().columns, os.get_terminal_size().lines))
        out("pid: %d\n\n" % os.getpid())

        out("PHASE 1: no mouse mode (5s). SCROLL UP AND DOWN NOW.\n")
        flush()
        b1 = window(fd, 5.0)
        out("--- bytes received without mouse mode: %d ---\n" % len(b1))
        if b1:
            out(decode(b1) + "\n")
        else:
            out("(nothing — the terminal scrolled its own scrollback, not the app)\n")
        out("\n")

        out("PHASE 2: mouse reporting ON (1000/1002/1003/1006) (5s). SCROLL NOW.\n")
        out("\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h")
        flush()
        b2 = window(fd, 5.0)
        out("--- bytes received with mouse mode: %d ---\n" % len(b2))
        if b2:
            out(decode(b2) + "\n")
        else:
            out("(nothing — the terminal ignored the mouse-mode request)\n")

        out("\x1b[?1006l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?25h")
        out("\n=== done — copy this whole output ===\n")
        flush()
        time.sleep(0.3)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)
    return 0


if __name__ == "__main__":
    sys.exit(main())
