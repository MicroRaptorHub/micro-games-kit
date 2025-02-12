/// [vertex]
#version 300 es

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec3 a_uv;
layout(location = 2) in vec4 a_color;
out vec4 v_color;
out vec3 v_uv;
uniform mat4 u_projection_view;

void main() {
    gl_Position = u_projection_view * vec4(a_position, 0.0, 1.0);
    v_color = a_color;
    v_uv = a_uv;
}

/// [fragment]
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