struct Params {
    dim: f32,
    x: f32,
    y: f32,
    radius: f32,
    feather: f32,
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let aspect = frame.size.x / max(frame.size.y, 1.0);
    let center = vec2<f32>(params.x, params.y) / 100.0;
    
    let delta = (uv - center) * vec2<f32>(aspect, 1.0);
    let dist = length(delta);
    
    let r = max(params.radius / 100.0, 0.01);
    let f = max(params.feather / 100.0, 0.01) * r;
    
    let spot = 1.0 - smoothstep(r - f, r + f, dist);
    let dim_factor = 1.0 - (params.dim / 100.0) * (1.0 - spot);
    
    let col = sample(uv);
    return vec4<f32>(col.rgb * dim_factor, col.a);
}
