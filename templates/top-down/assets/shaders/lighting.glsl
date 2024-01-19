#version 300 es

precision highp float;
precision highp int;
precision highp sampler2DArray;

in vec3 v_uv;
out vec4 o_color;
uniform sampler2DArray u_image;
uniform vec4 u_dark_color;
uniform vec4 u_light_color;

void main() {
    float factor = clamp(texture(u_image, v_uv).x, 0.0, 1.0);
    o_color = mix(u_dark_color, u_light_color, factor);
}