#version 450

layout(location = 0) in vec3 v_direction;

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform PushConstants {
    mat4 view;
    mat4 projection;
    vec4 sun_direction;
    vec4 sky_zenith;      // rgb = zenith color
    vec4 sky_horizon;     // rgb = horizon color
    vec4 sun_params;      // x = sun size, y = sun glow, z = time_of_day
} pc;

void main() {
    vec3 dir = normalize(v_direction);

    // Gradient based on vertical direction
    float horizon_factor = 1.0 - abs(dir.y);
    horizon_factor = pow(horizon_factor, 0.8);

    vec3 sky_color = mix(pc.sky_zenith.rgb, pc.sky_horizon.rgb, horizon_factor);

    // Sun glow
    vec3 sun_dir = normalize(pc.sun_direction.xyz);
    float sun_dot = dot(dir, sun_dir);

    // Sun disk
    float sun_size = pc.sun_params.x;
    float sun_disk = smoothstep(1.0 - sun_size, 1.0 - sun_size * 0.5, sun_dot);

    // Sun glow halo
    float sun_glow = pc.sun_params.y;
    float glow = pow(max(sun_dot, 0.0), 4.0) * sun_glow;

    // Sun color (warmer near horizon)
    vec3 sun_color = vec3(1.0, 0.95, 0.85);
    if (sun_dir.y < 0.2) {
        float sunset_factor = 1.0 - sun_dir.y / 0.2;
        sun_color = mix(sun_color, vec3(1.0, 0.5, 0.2), sunset_factor * 0.7);
    }

    vec3 final_color = sky_color + glow * sun_color * 0.3;
    final_color = mix(final_color, sun_color, sun_disk);

    // Stars at night (when sun is below horizon)
    if (pc.sun_direction.w < 0.1) {
        // Simple pseudo-random stars based on direction
        float star_seed = fract(sin(dot(floor(dir * 500.0), vec3(12.9898, 78.233, 45.164))) * 43758.5453);
        float star = step(0.998, star_seed) * (1.0 - pc.sun_direction.w * 10.0);
        final_color += vec3(star);
    }

    f_color = vec4(final_color, 1.0);
}
