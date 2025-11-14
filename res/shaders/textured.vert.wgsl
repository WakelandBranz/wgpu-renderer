struct ScreenSize {
    screen_size: vec2<f32>,
}

struct VertexOutput {
    @location(0) vColor: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(0) 
var<uniform> global: ScreenSize;
var<private> aPosition_1: vec2<f32>;
var<private> aColor_1: vec4<f32>;
var<private> vColor: vec4<f32>;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var ndc_x: f32;
    var ndc_y: f32;

    let _e5: vec2<f32> = aPosition_1;
    let _e7: vec2<f32> = global.screen_size;
    ndc_x = (((_e5.x / _e7.x) * 2f) - 1f);
    let _e16: vec2<f32> = aPosition_1;
    let _e18: vec2<f32> = global.screen_size;
    ndc_y = (1f - ((_e16.y / _e18.y) * 2f));
    let _e26: f32 = ndc_x;
    let _e27: f32 = ndc_y;
    gl_Position = vec4<f32>(_e26, _e27, 0f, 1f);
    let _e31: vec4<f32> = aColor_1;
    vColor = _e31;
    return;
}

@vertex 
fn main(@location(0) aPosition: vec2<f32>, @location(1) aColor: vec4<f32>) -> VertexOutput {
    aPosition_1 = aPosition;
    aColor_1 = aColor;
    main_1();
    let _e13: vec4<f32> = vColor;
    let _e15: vec4<f32> = gl_Position;
    return VertexOutput(_e13, _e15);
}
