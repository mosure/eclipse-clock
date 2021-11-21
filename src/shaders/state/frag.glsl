#version 300 es
precision highp float;

#define PI                  3.14159265359
#define RESET_TIME_SECONDS  0.5
#define PIXELS_PER_CELL     1
#define FPS                 60.0
#define SATURATION          0.7
#define LIGHTNESS           0.9
#define BACKGROUND          vec3(0.0, 0.0, 0.0)
#define GLITCH_THRESH       0.000005

uniform int         u_frame;
uniform vec2        u_resolution;
uniform sampler2D   u_state;
uniform float       u_time;

out vec4 outColor;


// https://stackoverflow.com/questions/4200224/random-noise-functions-for-glsl
uint hash( uint x ) {
    x += ( x << 10u );
    x ^= ( x >>  6u );
    x += ( x <<  3u );
    x ^= ( x >> 11u );
    x += ( x << 15u );
    return x;
}

uint hash( uvec2 v ) { return hash( v.x ^ hash(v.y)                         ); }
uint hash( uvec3 v ) { return hash( v.x ^ hash(v.y) ^ hash(v.z)             ); }
uint hash( uvec4 v ) { return hash( v.x ^ hash(v.y) ^ hash(v.z) ^ hash(v.w) ); }

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
float floatConstruct( uint m ) {
    const uint ieeeMantissa = 0x007FFFFFu; // binary32 mantissa bitmask
    const uint ieeeOne      = 0x3F800000u; // 1.0 in IEEE binary32

    m &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
    m |= ieeeOne;                          // Add fractional part to 1.0

    float  f = uintBitsToFloat( m );       // Range [1:2]
    return f - 1.0;                        // Range [0:1]
}



// Pseudo-random value in half-open range [0:1].
float random( float x ) { return floatConstruct(hash(floatBitsToUint(x))); }
float random( vec2  v ) { return floatConstruct(hash(floatBitsToUint(v))); }
float random( vec3  v ) { return floatConstruct(hash(floatBitsToUint(v))); }
float random( vec4  v ) { return floatConstruct(hash(floatBitsToUint(v))); }

// 2D Noise based on Morgan McGuire @morgan3d
// https://www.shadertoy.com/view/4dS3Wd
float noise(in vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);

    // Four corners in 2D of a tile
    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));

    // Smooth Interpolation

    // Cubic Hermine Curve.  Same as SmoothStep()
    vec2 u = f*f*(3.0-2.0*f);
    // u = smoothstep(0.,1.,f);

    // Mix 4 coorners percentages
    return mix(a, b, u.x) +
            (c - a)* u.y * (1.0 - u.x) +
            (d - b) * u.x * u.y;
}

vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec4 get_cell_at(ivec2 coords) {
    vec2 lookup_coords = mod(vec2(coords) * vec2(PIXELS_PER_CELL), u_resolution);

    return texture(u_state, lookup_coords / u_resolution + 1.0 / (u_resolution * 10.0));
}

bool is_coord_alive(ivec2 coords) {
    return get_cell_at(coords).a == 1.0;
}

float get_coord_hue(ivec2 coords) {
    if (!is_coord_alive(coords)) {
        return 0.0;
    }

    return rgb2hsv(get_cell_at(coords).rgb).x;
}

ivec2 get_mapped_coords(vec2 coords) {
    return ivec2(coords) / ivec2(PIXELS_PER_CELL);
}

int get_neighbor_count(ivec2 coords) {
    int count = 0;

    count += int(is_coord_alive(coords + ivec2(-1, -1)));
    count += int(is_coord_alive(coords + ivec2(0, -1)));
    count += int(is_coord_alive(coords + ivec2(1, -1)));
    count += int(is_coord_alive(coords + ivec2(-1, 0)));
    count += int(is_coord_alive(coords + ivec2(1, 0)));
    count += int(is_coord_alive(coords + ivec2(-1, 1)));
    count += int(is_coord_alive(coords + ivec2(0, 1)));
    count += int(is_coord_alive(coords + ivec2(1, 1)));

    return count;
}

float neighbor_hue_sum(ivec2 coords) {
    float hue_sum = 0.0;

    hue_sum += get_coord_hue(coords + ivec2(-1, -1));
    hue_sum += get_coord_hue(coords + ivec2(0, -1));
    hue_sum += get_coord_hue(coords + ivec2(1, -1));
    hue_sum += get_coord_hue(coords + ivec2(-1, 0));
    hue_sum += get_coord_hue(coords + ivec2(1, 0));
    hue_sum += get_coord_hue(coords + ivec2(-1, 1));
    hue_sum += get_coord_hue(coords + ivec2(0, 1));
    hue_sum += get_coord_hue(coords + ivec2(1, 1));
    
    return hue_sum;
}

vec4 should_live() {
    vec2 st = gl_FragCoord.xy / u_resolution.xy;
    ivec2 mapped_coords = get_mapped_coords(gl_FragCoord.xy);

    if (u_frame % int(60.0 / FPS) != 0) {
        return get_cell_at(mapped_coords);
    }

    int neighbor_count = get_neighbor_count(mapped_coords);

    bool self_alive = is_coord_alive(mapped_coords);

    bool ns = abs(random(vec2(st.x + sin(float(u_time)) / 10.0, st.y + sin(float(u_time))) / 10.0)) < GLITCH_THRESH;

    bool alive = neighbor_count == 3 || (
        neighbor_count == 2 && self_alive
    )  || ns;

    float hue_sum = neighbor_hue_sum(mapped_coords) / float(neighbor_count);
    if (self_alive) {
        hue_sum = get_coord_hue(mapped_coords) + random(
            vec2(st.x + float(u_time), st.y + float(u_time))
        ) / 500.0;
    }

    return vec4(hsv2rgb(vec3(hue_sum, SATURATION, LIGHTNESS)), float(alive));
}

bool try_reset() {
    if (u_time < RESET_TIME_SECONDS) {
        vec2 st = gl_FragCoord.xy / u_resolution.xy;

        float rnd = noise(st * 1000.0) + 0.5;

        outColor = vec4(hsv2rgb(vec3(fract(rnd), 1.0, 1.0)), floor(rnd));
        return true;
    }

    return false;
}

void main() {
    if (try_reset()) {
        return;
    } else {
        vec2 st = gl_FragCoord.xy / u_resolution;

        vec4 state = should_live();
        if (bool(state.a)) {
            outColor = vec4(state.rgb, 1.0);
        } else {
            outColor = vec4(BACKGROUND, 0.98);
        }
    }
}
