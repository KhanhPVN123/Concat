struct Params {
    padding: f32,
    radius: f32,
    shadow: f32,
    header: f32,
    bg_style: f32,
}

fn get_bg(uv: vec2<f32>, style: f32) -> vec3<f32> {
    let t = (uv.x + uv.y) * 0.5;
    if (style < 0.5) {
        // Deep Indigo (#1e1b4b -> #0f172a)
        let c1 = vec3<f32>(0.08, 0.06, 0.22);
        let c2 = vec3<f32>(0.02, 0.04, 0.10);
        return mix(c1, c2, t);
    } else if (style < 1.5) {
        // Sunset Glow (#e11d48 -> #7c3aed)
        let c1 = vec3<f32>(0.75, 0.08, 0.22);
        let c2 = vec3<f32>(0.35, 0.10, 0.70);
        return mix(c1, c2, t);
    } else if (style < 2.5) {
        // Dark Minimalist (#18181b -> #09090b)
        let c1 = vec3<f32>(0.06, 0.06, 0.07);
        let c2 = vec3<f32>(0.02, 0.02, 0.02);
        return mix(c1, c2, t);
    } else {
        // Ocean Wave (#0284c7 -> #1e1b4b)
        let c1 = vec3<f32>(0.01, 0.40, 0.70);
        let c2 = vec3<f32>(0.05, 0.05, 0.25);
        return mix(c1, c2, t);
    }
}

fn rounded_box_dist(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let pad = params.padding / 100.0;
    if (pad <= 0.001) {
        return sample(uv);
    }
    
    let aspect = frame.size.x / max(frame.size.y, 1.0);
    let bg_color = get_bg(uv, params.bg_style);
    
    let has_header = params.header >= 0.5;
    let header_h = select(0.0, 0.045, has_header);
    
    let pad_x = pad;
    let pad_y = pad;
    
    // Overall window bounds
    let win_min = vec2<f32>(pad_x, pad_y);
    let win_max = vec2<f32>(1.0 - pad_x, 1.0 - pad_y);
    let win_size = win_max - win_min;
    let win_center = (win_min + win_max) * 0.5;
    
    // Distance from window in aspect-corrected coordinates for accurate corners & shadow
    let p = (uv - win_center) * vec2<f32>(aspect, 1.0);
    let b = (win_size * 0.5) * vec2<f32>(aspect, 1.0);
    let r = (params.radius / 100.0) * 0.1 * min(aspect, 1.0);
    let dist = rounded_box_dist(p, b, r);
    
    // Pixel feather for smooth anti-aliasing
    let pixel_w = 1.5 / max(frame.size.y, 1.0);
    let window_alpha = 1.0 - smoothstep(-pixel_w, pixel_w, dist);
    
    // Outside the window: draw shadow over background
    if (window_alpha <= 0.001) {
        let shadow_radius = 0.05 * (params.shadow / 50.0);
        let shadow_factor = (1.0 - smoothstep(0.0, shadow_radius, max(dist, 0.0))) * (params.shadow / 100.0);
        let final_bg = bg_color * (1.0 - shadow_factor * 0.7);
        return vec4<f32>(final_bg, 1.0);
    }
    
    // Inside the window: check if in header
    let content_min = vec2<f32>(win_min.x, win_min.y + header_h);
    let content_size = vec2<f32>(win_size.x, win_size.y - header_h);
    
    var win_col = vec4<f32>(0.0);
    
    if (has_header && uv.y < content_min.y) {
        // Window Titlebar Header (Dark Slate #1e293b)
        win_col = vec4<f32>(0.10, 0.12, 0.16, 1.0);
        
        // Traffic light control buttons
        let btn_y = win_min.y + header_h * 0.5;
        let btn_r = 0.006;
        
        let red_pos = vec2<f32>(win_min.x + 0.02, btn_y);
        let yellow_pos = vec2<f32>(win_min.x + 0.035, btn_y);
        let green_pos = vec2<f32>(win_min.x + 0.05, btn_y);
        
        let d_red = length((uv - red_pos) * vec2<f32>(aspect, 1.0));
        let d_yel = length((uv - yellow_pos) * vec2<f32>(aspect, 1.0));
        let d_grn = length((uv - green_pos) * vec2<f32>(aspect, 1.0));
        
        if (d_red < btn_r) {
            win_col = vec4<f32>(0.95, 0.25, 0.25, 1.0); // Close button red
        } else if (d_yel < btn_r) {
            win_col = vec4<f32>(0.95, 0.75, 0.20, 1.0); // Minimize yellow
        } else if (d_grn < btn_r) {
            win_col = vec4<f32>(0.20, 0.80, 0.35, 1.0); // Maximize green
        }
    } else {
        // Video screen content
        let content_uv = (uv - content_min) / max(content_size, vec2<f32>(0.001));
        let clamped_uv = clamp(content_uv, vec2<f32>(0.0), vec2<f32>(1.0));
        win_col = sample(clamped_uv);
    }
    
    // Blend window with background for anti-aliased rounded corners
    return mix(vec4<f32>(bg_color, 1.0), win_col, window_alpha);
}
