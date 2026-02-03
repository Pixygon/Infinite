#version 450

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 v_direction;

layout(push_constant) uniform PushConstants {
    mat4 view;
    mat4 projection;
    vec4 sun_direction;
    vec4 sky_zenith;      // rgb = zenith color, a = unused
    vec4 sky_horizon;     // rgb = horizon color, a = unused
    vec4 sun_params;      // x = sun size, y = sun glow, z = time_of_day, w = unused
} pc;

void main() {
    v_direction = position;

    // Remove translation from view matrix for infinite sky
    mat4 view_no_translate = pc.view;
    view_no_translate[3] = vec4(0.0, 0.0, 0.0, 1.0);

    vec4 pos = pc.projection * view_no_translate * vec4(position * 1000.0, 1.0);

    // Set z to w so depth is always at far plane
    gl_Position = pos.xyww;
}
