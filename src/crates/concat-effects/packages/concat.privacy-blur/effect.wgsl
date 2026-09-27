struct Params {
    amount: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    feather: f32,
}

const TAPS: i32 = 8;

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let center = vec2<f32>(params.x, params.y) / 100.0;
    let half_size = vec2<f32>(params.w, params.h) / 200.0;
    let d = abs(uv - center) - half_size;
    let dist = max(d.x, d.y);
    
    let f = max(params.feather / 100.0, 0.001) * 0.1;
    let mask = 1.0 - smoothstep(-f, 0.0, dist);
    
    if (mask <= 0.001) {
        return sample(uv);
    }
    
    let radius = (params.amount / 100.0) * 0.04;
    let aspect = frame.size.x / max(frame.size.y, 1.0);
    
    var sum = sample(uv);
    var count = 1.0;
    
    // 8-direction box/disc sampling
    let step_x = radius / aspect;
    let step_y = radius;
    
    sum += sample(uv + vec2<f32>(step_x, 0.0));
    sum += sample(uv - vec2<f32>(step_x, 0.0));
    sum += sample(uv + vec2<f32>(0.0, step_y));
    sum += sample(uv - vec2<f32>(0.0, step_y));
    sum += sample(uv + vec2<f32>(step_x * 0.7, step_y * 0.7));
    sum += sample(uv - vec2<f32>(step_x * 0.7, step_y * 0.7));
    sum += sample(uv + vec2<f32>(-step_x * 0.7, step_y * 0.7));
    sum += sample(uv + vec2<f32>(step_x * 0.7, -step_y * 0.7));
    count += 8.0;
    
    let blurred = sum / count;
    let original = sample(uv);
    
    return mix(original, blurred, mask);
}
