struct Params {
    zoom: f32,
    pitch: f32,
    yaw: f32,
    focus_x: f32,
    focus_y: f32,
    perspective: f32,
}

const PI: f32 = 3.141592653589793;

fn in_bounds(uv: vec2<f32>) -> bool {
    return uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0;
}

fn radians(deg: f32) -> f32 {
    return deg * (PI / 180.0);
}

fn rotate_x(p: vec3<f32>, angle: f32) -> vec3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec3<f32>(p.x, p.y * c - p.z * s, p.y * s + p.z * c);
}

fn rotate_y(p: vec3<f32>, angle: f32) -> vec3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec3<f32>(p.x * c + p.z * s, p.y, -p.x * s + p.z * c);
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let aspect = frame.size.x / max(frame.size.y, 1.0);
    let focus = vec2<f32>(params.focus_x, params.focus_y) / 100.0;
    let zoom_factor = max(params.zoom / 100.0, 0.01);
    
    // Normalized centered screen point (relative to focus)
    let p_screen = vec2<f32>((uv.x - focus.x) * aspect, uv.y - focus.y);
    
    // Camera distance (d) in normalized units
    let d = max(params.perspective, 100.0) / 1000.0;
    
    let pitch_rad = radians(params.pitch);
    let yaw_rad = radians(params.yaw);
    
    // The normal of the tilted plane: N = Ry(yaw) * Rx(pitch) * (0, 0, 1)
    let normal = rotate_y(rotate_x(vec3<f32>(0.0, 0.0, 1.0), pitch_rad), yaw_rad);
    
    // Ray from camera position (0, 0, d) towards screen point (p_screen.x, p_screen.y, 0)
    let ray_dir = vec3<f32>(p_screen.x, p_screen.y, -d);
    
    let denom = dot(normal, ray_dir);
    if (abs(denom) < 1e-6) {
        return vec4<f32>(0.0);
    }
    
    // Solve plane intersection N . (P - 0) = 0 => N . (Camera + t * ray_dir) = 0
    let t = -(normal.z * d) / denom;
    if (t <= 0.0) {
        return vec4<f32>(0.0);
    }
    
    let p_hit = vec3<f32>(0.0, 0.0, d) + t * ray_dir;
    
    // Rotate back to un-tilted coordinate system: P_orig = Rx(-pitch) * Ry(-yaw) * p_hit
    let p_orig = rotate_x(rotate_y(p_hit, -yaw_rad), -pitch_rad);
    
    // Apply zoom and convert back to texture UV coordinates
    let uv_sample = focus + vec2<f32>(p_orig.x / (aspect * zoom_factor), p_orig.y / zoom_factor);
    
    if (!in_bounds(uv_sample)) {
        return vec4<f32>(0.0);
    }
    
    return sample(uv_sample);
}
