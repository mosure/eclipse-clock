import datetime
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


STATIC_COLOR = convert_K_to_RGB(3200)


def eclipse_clock(x: int, resolution: int, now: datetime) -> Color:
    ORIGIN = 0

    st = (x / resolution + ORIGIN) % 1.0

    intensity = hour_intensity(st, now) * minute_intensity(st, now) * second_intensity(st, now)

    return tuple([channel * intensity for channel in STATIC_COLOR])
