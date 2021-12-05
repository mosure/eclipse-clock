import datetime
import math


def convert_K_to_RGB(colour_temperature):
    """
    https://gist.github.com/petrklus/b1f427accdf7438606a6

    Converts from K to RGB, algorithm courtesy of 
    http://www.tannerhelland.com/4435/convert-temperature-rgb-algorithm-code/
    """
    #range check
    if colour_temperature < 1000: 
        colour_temperature = 1000
    elif colour_temperature > 40000:
        colour_temperature = 40000
    
    tmp_internal = colour_temperature / 100.0
    
    # red 
    if tmp_internal <= 66:
        red = 255
    else:
        tmp_red = 329.698727446 * math.pow(tmp_internal - 60, -0.1332047592)
        if tmp_red < 0:
            red = 0
        elif tmp_red > 255:
            red = 255
        else:
            red = tmp_red
    
    # green
    if tmp_internal <=66:
        tmp_green = 99.4708025861 * math.log(tmp_internal) - 161.1195681661
        if tmp_green < 0:
            green = 0
        elif tmp_green > 255:
            green = 255
        else:
            green = tmp_green
    else:
        tmp_green = 288.1221695283 * math.pow(tmp_internal - 60, -0.0755148492)
        if tmp_green < 0:
            green = 0
        elif tmp_green > 255:
            green = 255
        else:
            green = tmp_green
    
    # blue
    if tmp_internal >=66:
        blue = 255
    elif tmp_internal <= 19:
        blue = 0
    else:
        tmp_blue = 138.5177312231 * math.log(tmp_internal - 10) - 305.0447927307
        if tmp_blue < 0:
            blue = 0
        elif tmp_blue > 255:
            blue = 255
        else:
            blue = tmp_blue
    
    return red, green, blue


def get_hour(now: datetime):
    return now.hour + now.minute / 60 + now.second / 60 / 60 + now.microsecond / 1e6 / 60 / 60


def get_minute(now: datetime):
    return now.minute + now.second / 60 + now.microsecond / 1e6 / 60


def get_second(now: datetime):
    return now.second + now.microsecond / 1e6


def clamp(x: float, min: float, max: float) -> float:
    if x > max:
        return max

    if x < min:
        return min

    return x


def smoothstep(edge0: float, edge1: float, x: float) -> float:
    t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0)

    return t * t * (3.0 - 2.0 * t)


def region(target: float, st: float, bound: float):
    if target + bound > 1.0 and st < (target + bound) - 1.0:
        return smoothstep(target - bound, target, st + 1.0) * smoothstep(target + bound, target, st + 1.0)
    elif target - bound < 0.0 and st > 1.0 + (target - bound):
        return smoothstep(target - bound, target, st - 1.0) * smoothstep(target + bound, target, st - 1.0)

    return smoothstep(target - bound, target, st) * smoothstep(target + bound, target, st)
