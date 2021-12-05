import datetime

import adafruit_dotstar
import board

from effect import Color, eclipse_clock, FragmentShader


GAMMA = bytearray(256)
for i in range(256):
    GAMMA[i] = int(pow(float(i) / 255.0, 2.7) * 255.0 + 0.5)


num_pixels = 144
pixels = adafruit_dotstar.DotStar(
    board.SCK,
    board.MOSI,
    num_pixels,
    auto_write=False
)


def write(index: int, color: Color):
    pixels[index] = tuple(map(lambda c: GAMMA[int(c)], color))

def do_frame(frag_shader: FragmentShader):
    now = datetime.datetime.now()

    for i in range(num_pixels):
        color = frag_shader(
            x=i,
            resolution=num_pixels,
            now=now,
        )

        write(i, color)

    pixels.show()


while True:
    do_frame(eclipse_clock)
    break
