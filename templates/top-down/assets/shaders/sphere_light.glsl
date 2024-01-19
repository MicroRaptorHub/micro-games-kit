#version 300 es

precision highp float;
precision highp int;

in vec3 v_uv;
out vec4 o_color;
uniform vec2 u_intensity;
uniform float u_attenuation;

void main() {
    float factor = clamp(1.0 - length(v_uv.xy * 2.0 - 1.0), 0.0, 1.0);
    o_color = vec4(mix(u_intensity.x, u_intensity.y, pow(factor, u_attenuation)));
}