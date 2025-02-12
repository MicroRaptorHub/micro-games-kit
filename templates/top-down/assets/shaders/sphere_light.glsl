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

in vec3 v_uv;
out vec4 o_color;
uniform vec2 u_intensity;
uniform float u_attenuation;

void main() {
    float factor = clamp(1.0 - length(v_uv.xy * 2.0 - 1.0), 0.0, 1.0);
    o_color = vec4(mix(u_intensity.x, u_intensity.y, pow(factor, u_attenuation)));
}