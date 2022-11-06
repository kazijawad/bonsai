@group(0) @binding(0)
var texture_out: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let screen_size = textureDimensions(texture_out);
    let screen_position = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let color = vec4<f32>(f32(global_id.x) / f32(screen_size.x), f32(global_id.y) / f32(screen_size.y), 0.0, 1.0);
    textureStore(texture_out, screen_position, color);
}
