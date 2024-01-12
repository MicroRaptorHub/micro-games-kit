#version 300 es

precision highp float;
precision highp int;
precision highp sampler2DArray;

in vec4 v_color;
in vec3 v_uv;
out vec4 o_color;
uniform sampler2DArray u_image;
uniform vec4 u_fill_color;

void main() {
    vec4 color = texture(u_image, v_uv) * v_color;
    o_color = vec4(mix(color.rgb, u_fill_color.rgb, u_fill_color.a), color.a);
}