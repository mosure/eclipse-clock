#version 300 es
precision highp float;

#define PI                  3.14159265359
#define ORIGIN              2.0 * PI + PI / 2.0
#define RADIUS              0.25
#define DIFFUSION           0.5
#define PLATE_COLOR         vec3(0.0, 0.0, 0.0) / 255.0
#define HOUR_INTENSITY      0.9
#define HOUR_THETA          1.0 / 6.0 / 2.0
#define MINUTE_INTENSITY    0.7
#define MINUTE_THETA        1.0 / 6.0 / 2.0
#define SECOND_INTENSITY    0.5
#define SECOND_THETA        1.0 / 6.0 / 2.0

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

float circle(in vec2 _st, in float _radius){
    vec2 dist = _st-vec2(0.5);
    return 1.-smoothstep(_radius-(_radius*0.01),
                            _radius+(_radius*0.01),
                            dot(dist,dist)*4.0);
}

float hour_intensity(in float theta) {
    float hour_pos = u_hour / 12.0;

    float region = smoothstep(hour_pos - HOUR_THETA, hour_pos, theta) *
                   smoothstep(hour_pos + HOUR_THETA, hour_pos, theta);

    return 1.0 - region * HOUR_INTENSITY;
}

float minute_intensity(in float theta) {
    float minute_pos = u_minute / 60.0;

    float region = smoothstep(minute_pos - MINUTE_THETA, minute_pos, theta) *
                   smoothstep(minute_pos + MINUTE_THETA, minute_pos, theta);

    return 1.0 - region * MINUTE_INTENSITY;
}

float second_intensity(in float theta) {
    float second_pos = u_second / 60.0;

    float region = smoothstep(second_pos - SECOND_THETA, second_pos, theta) *
                   smoothstep(second_pos + SECOND_THETA, second_pos, theta);

    return 1.0 - region * SECOND_INTENSITY;
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

    vec3 light_color = vec3(1.0);

    vec3 plate = mask * PLATE_COLOR + intensity * inv_mask;

    // outColor = vec4(st.x, st.y, sin(float(u_millisecond) / 1000.0), 1.0);
    outColor = vec4(plate, 1.0);
}
