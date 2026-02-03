#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec4 v_color;
layout(location = 2) in vec3 v_world_pos;

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec4 sun_direction;   // xyz = direction, w = intensity
    vec4 sun_color;       // xyz = color, w = ambient intensity
} pc;

void main() {
    vec3 N = normalize(v_normal);
    vec3 L = normalize(pc.sun_direction.xyz);

    // Diffuse lighting
    float NdotL = max(dot(N, L), 0.0);
    float sun_intensity = pc.sun_direction.w;
    float ambient_intensity = pc.sun_color.w;

    vec3 diffuse = v_color.rgb * NdotL * pc.sun_color.rgb * sun_intensity;
    vec3 ambient = v_color.rgb * ambient_intensity;

    vec3 final_color = ambient + diffuse;

    // Simple fog for distance
    float dist = length(v_world_pos);
    float fog = exp(-dist * 0.01);
    vec3 fog_color = vec3(0.02, 0.02, 0.05);
    final_color = mix(fog_color, final_color, fog);

    f_color = vec4(final_color, v_color.a);
}
