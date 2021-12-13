import colorsys
import datetime
import math
import time
from typing import Protocol

from noise import pnoise1, pnoise3, snoise3

from util import clamp, convert_K_to_RGB, get_hour, get_minute, get_second, region


Color = tuple[float, float, float]


class FragmentShader(Protocol):
    def __call__(
        self,
        x: int,
        resolution: int,
        now: datetime,
    ) -> Color: ...


def hour_intensity(st: float, now: datetime):
    COUNT = 12
    SIZE = 1 / 12
    INTENSITY = 0.85

    pos = (get_hour(now) % COUNT) / COUNT

    return 1.0 - region(pos, st, SIZE) * INTENSITY


def minute_intensity(st: float, now: datetime):
    COUNT = 60
    SIZE = 1 / 12
    INTENSITY = 0.6

    pos = (get_minute(now) % COUNT) / COUNT

    return 1.0 - region(pos, st, SIZE) * INTENSITY


def second_intensity(st: float, now: datetime):
    COUNT = 60
    SIZE = 1 / 12
    INTENSITY = 0.4

    pos = (get_second(now) % COUNT) / COUNT

    return 1.0 - region(pos, st, SIZE) * INTENSITY


def eclipse_clock(origin = 11/24, dimming = 0.6, static_color = convert_K_to_RGB(3200), night_mode=False):
    def _eclipse_clock(x: int, resolution: int, now: datetime) -> Color:
        st = ((-x + resolution) / resolution + origin) % 1.0

        if night_mode:
            intensity = (1 - hour_intensity(st, now)) * dimming
        else:
            intensity = hour_intensity(st, now) * minute_intensity(st, now) * second_intensity(st, now) * dimming

        return tuple([channel * intensity for channel in static_color])

    return _eclipse_clock


def color_noise(segment = True):
    boot_time = time.time()

    def _color_noise(x: int, resolution: int, now: datetime) -> Color:
        run_time = time.time() - boot_time
        st = (x / resolution + 10 * pnoise1(run_time / 24, octaves=1) + 4 * pnoise1(run_time / 10, octaves=2)) % 1.0

        s = math.cos(st * 2 * math.pi)
        t = math.sin(st * 2 * math.pi)

        #hue = ((snoise3(s, t, run_time / 3, octaves=8) + 1) / 2 + run_time / 10) % 1
        #hue = (run_time / 6) % 1
        hue = (pnoise1(run_time / 15, octaves=2) + 1.0) % 1.0
        sat = (pnoise1(run_time / 24, octaves=2) + 1.0) % 1.0

        #ct = (run_time / 4) % 1.0

        if segment:
            #intensity = math.sin(st * 14 * math.pi + run_time * 7 * math.pi)
            #intensity = abs(math.sin(run_time * 4 * math.pi)) * 0.5
            intensity = clamp(snoise3(s, t, run_time / 12, octaves=1), 0, 1)
        else:
            intensity = 0.4

        r,g,b = colorsys.hsv_to_rgb(hue, clamp(sat, 0.2, 0.8), intensity)

        return tuple([
            r * 255,
            g * 255,
            b * 255,
        ])

    return _color_noise
