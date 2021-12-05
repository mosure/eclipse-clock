import datetime
import time

from effect import eclipse_clock, FragmentShader


num_pixels = 144


def do_frame(frag_shader: FragmentShader):
    now = datetime.datetime.now()

    for i in range(num_pixels):
        frag_shader(
            x=i,
            resolution=num_pixels,
            now=now,
        )


start_time = time.time()

ITERATIONS = 144 * 30
for i in range(ITERATIONS):
    do_frame(eclipse_clock)

end_time = time.time()

delta = end_time - start_time

print('Seconds per frame:')
print(delta / ITERATIONS)

print('FPS:')
print(ITERATIONS / delta)
