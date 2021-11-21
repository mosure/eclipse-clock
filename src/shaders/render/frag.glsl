#version 300 es
precision highp float;

#define PI                  3.14159265359
#define ORIGIN              2.0 * PI + PI / 2.0
#define RADIUS              0.2
#define DIFFUSION           0.6
#define PLATE_COLOR         vec3(0.0, 0.0, 0.0) / 255.0
#define LIGHT_COLOR         vec3(255.0, 234.0, 210.0) / 255.0
#define HOUR_INTENSITY      0.85
#define HOUR_THETA          1.0 / 12.0
#define MINUTE_INTENSITY    0.6
#define MINUTE_THETA        1.0 / 12.0
#define SECOND_INTENSITY    0.48
#define SECOND_THETA        1.0 / 12.0

uniform vec2        u_resolution;
uniform float       u_time;

uniform float       u_hour;
uniform float       u_minute;
uniform float       u_second;
uniform int         u_millisecond;

out vec4 outColor;


// https://stackoverflow.com/questions/26070410/robust-atany-x-on-glsl-for-converting-xy-coordinate-to-angle
float atan2(in float y, in float x) {
    return x == 0.0 ? sign(y) * PI / 2.0 : atan(y, x);
}

float invert(in float x) {
    return -min(x, 1.0) + 1.0;
}

float region(float target, float theta, float bound) {
    if (target + bound > 1.0 && theta < (target + bound) - 1.0) {
        // |-----------b--tg-|-b----------------|
        return smoothstep(target - bound, target, theta + 1.0) *
               smoothstep(target + bound, target, theta + 1.0);
    } else if (target - bound < 0.0 && theta > 1.0 + (target - bound)) {
        // |-----------b-|-tg--b----------------|
        return smoothstep(target - bound, target, theta - 1.0) *
               smoothstep(target + bound, target, theta - 1.0);
    }

    return smoothstep(target - bound, target, theta) *
           smoothstep(target + bound, target, theta);
}

float circle(in vec2 _st, in float _radius){
    vec2 dist = _st-vec2(0.5);
    return 1.-smoothstep(_radius-(_radius*0.01),
                            _radius+(_radius*0.01),
                            dot(dist,dist)*4.0);
}

float hour_intensity(in float theta) {
    float hour_pos = u_hour / 12.0;

    return 1.0 - region(hour_pos, theta, HOUR_THETA) * HOUR_INTENSITY;
}

float minute_intensity(in float theta) {
    float minute_pos = u_minute / 60.0;

    return 1.0 - region(minute_pos, theta, MINUTE_THETA) * MINUTE_INTENSITY;
}

float second_intensity(in float theta) {
    float second_pos = u_second / 60.0;

    return 1.0 - region(second_pos, theta, SECOND_THETA) * SECOND_INTENSITY;
}

void main() {
    vec2 st = gl_FragCoord.xy / u_resolution.xy;
    vec2 center = vec2(0.5);
    vec2 delta = st - center;
    float dist = distance(st, center);

    float theta = fract((ORIGIN - atan2(delta.y, delta.x)) / (2.0 * PI));

    float tan_dist = dist - (RADIUS - DIFFUSION * RADIUS);

    float grad = invert(tan_dist / DIFFUSION);
    float intensity = grad * grad * grad * grad * hour_intensity(theta) * minute_intensity(theta) * second_intensity(theta);

    float mask = circle(st, RADIUS);
    float inv_mask = 1.0 - mask;

    vec3 plate = mask * PLATE_COLOR + intensity * inv_mask * LIGHT_COLOR;

    outColor = vec4(plate, 1.0);
}
