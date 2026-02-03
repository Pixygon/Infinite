#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec4 v_color;
layout(location = 2) out vec3 v_world_pos;

layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec4 sun_direction;   // xyz = direction, w = intensity
    vec4 sun_color;       // xyz = color, w = ambient intensity
} pc;

void main() {
    vec4 world_pos = pc.model * vec4(position, 1.0);
    v_world_pos = world_pos.xyz;
    v_normal = mat3(pc.model) * normal;
    v_color = color;
    gl_Position = pc.projection * pc.view * world_pos;
}
