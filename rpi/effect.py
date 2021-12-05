import datetime
import math
import time
from typing import Protocol

from util import convert_K_to_RGB, get_hour, get_minute, get_second, region


"""
import colorsys
>>> colorsys.rgb_to_hsv(0.2, 0.4, 0.4)
(0.5, 0.5, 0.4)
>>> colorsys.hsv_to_rgb(0.5, 0.5, 0.4)
(0.2, 0.4, 0.4)
"""


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


def eclipse_clock(origin = 0.0, dimming = 1.0, static_color = convert_K_to_RGB(3200)):
    def _eclipse_clock(x: int, resolution: int, now: datetime) -> Color:
        st = (x / resolution + origin) % 1.0

        intensity = hour_intensity(st, now) * minute_intensity(st, now) * second_intensity(st, now) * dimming

        return tuple([channel * intensity for channel in static_color])

    return _eclipse_clock


def dev_effect():
    boot_time = time.time()

    def _dev_effect(x: int, resolution: int, now: datetime) -> Color:
        run_time = time.time() - boot_time()

        st = (x / resolution + run_time) % 1.0

        return tuple([math.sin(st * math.pi), 0, 0])

    return _dev_effect
