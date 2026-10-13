#!/usr/bin/env python3
"""Virtual Xbox 360 gamepad for session-keepalive input pulses.

Creates a uinput gamepad that SDL enumerates; ControllerInput forwards its
snapshots to the remote session without needing Qt item focus. Used to keep
GFN sessions alive during scored experiment arms (anti-AFK equivalent).

Usage:
  virtual-gamepad.py --once            # create pad, pulse L3 once, stay alive
  virtual-gamepad.py --period 150      # pulse every N seconds until killed
  virtual-gamepad.py --press <name>    # pulse a named button once then exit
Buttons: south east north west lb rb back start guide lstick rstick
         dup ddown dleft dright
Axes:    lx ly rx ry lt rt  (use --axis lx=12000 style moves with --move)
"""
import argparse, sys, time
from evdev import UInput, ecodes as e

CAP = {
    e.EV_KEY: [
        e.BTN_SOUTH, e.BTN_EAST, e.BTN_NORTH, e.BTN_WEST,
        e.BTN_TL, e.BTN_TR, e.BTN_SELECT, e.BTN_START, e.BTN_MODE,
        e.BTN_THUMBL, e.BTN_THUMBR,
    ],
    e.EV_ABS: [
        (e.ABS_X,  (0, -32768, 32767, 16, 128)),
        (e.ABS_Y,  (0, -32768, 32767, 16, 128)),
        (e.ABS_RX, (0, -32768, 32767, 16, 128)),
        (e.ABS_RY, (0, -32768, 32767, 16, 128)),
        (e.ABS_Z,  (0, 0, 255, 0, 0)),
        (e.ABS_RZ, (0, 0, 255, 0, 0)),
        (e.ABS_HAT0X, (0, -1, 1, 0, 0)),
        (e.ABS_HAT0Y, (0, -1, 1, 0, 0)),
    ],
}

BUTTONS = {
    "south": e.BTN_SOUTH, "east": e.BTN_EAST, "north": e.BTN_NORTH,
    "west": e.BTN_WEST, "lb": e.BTN_TL, "rb": e.BTN_TR,
    "back": e.BTN_SELECT, "start": e.BTN_START, "guide": e.BTN_MODE,
    "lstick": e.BTN_THUMBL, "rstick": e.BTN_THUMBR,
}
HATS = {
    "dup":    (e.ABS_HAT0Y, -1), "ddown":  (e.ABS_HAT0Y, 1),
    "dleft":  (e.ABS_HAT0X, -1), "dright": (e.ABS_HAT0X, 1),
}
AXES = {
    "lx": e.ABS_X, "ly": e.ABS_Y, "rx": e.ABS_RX, "ry": e.ABS_RY,
    "lt": e.ABS_Z, "rt": e.ABS_RZ,
}


def make_pad():
    return UInput(
        events=CAP,
        name="Microsoft X-Box 360 pad",
        vendor=0x045E, product=0x028E, version=0x0110,
        bustype=e.BUS_USB,
    )


def pulse(ui, name, hold=0.06):
    if name in BUTTONS:
        code = BUTTONS[name]
        ui.write(e.EV_KEY, code, 1); ui.syn()
        time.sleep(hold)
        ui.write(e.EV_KEY, code, 0); ui.syn()
    elif name in HATS:
        code, val = HATS[name]
        ui.write(e.EV_ABS, code, val); ui.syn()
        time.sleep(hold)
        ui.write(e.EV_ABS, code, 0); ui.syn()
    else:
        raise SystemExit(f"unknown button {name!r}")


def move(ui, axis, value, hold=0.08):
    code = AXES[axis]
    ui.write(e.EV_ABS, code, value); ui.syn()
    time.sleep(hold)
    ui.write(e.EV_ABS, code, 0); ui.syn()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--period", type=float, default=0,
                   help="pulse every N seconds (0 = single pulse)")
    p.add_argument("--button", default="lstick", help="pulse button name")
    p.add_argument("--move", default="", help="axis move like lx=12000")
    args = p.parse_args()

    ui = make_pad()
    print(f"virtual pad created at {ui.device.path}", flush=True)
    time.sleep(1.5)  # let SDL enumerate before first event

    def do_pulse():
        if args.move:
            axis, _, v = args.move.partition("=")
            move(ui, axis, int(v))
        else:
            pulse(ui, args.button)
        print(f"pulse {time.strftime('%H:%M:%S')} {args.move or args.button}", flush=True)

    do_pulse()
    if args.period > 0:
        try:
            while True:
                time.sleep(args.period)
                do_pulse()
        except KeyboardInterrupt:
            pass
    else:
        time.sleep(1.0)  # keep device alive briefly so SDL sees the events
    ui.close()


if __name__ == "__main__":
    main()
