#version 300 es
precision highp float;

#define PI                  3.14159265359
#define RESET_TIME_SECONDS  2.0

uniform vec2        u_resolution;
uniform sampler2D   u_state;
uniform float       u_time;

out vec4 outColor;


void main() {
    outColor = texture(u_state, gl_FragCoord.xy / u_resolution);
}
