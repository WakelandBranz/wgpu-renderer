struct VertexOutput {
    @location(0) vColor: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

var<private> aPosition_1: vec2<f32>;
var<private> aColor_1: vec4<f32>;
var<private> vColor: vec4<f32>;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var ndc_x: f32;
    var ndc_y: f32;

    let _e3: vec2<f32> = aPosition_1;
    ndc_x = (((_e3.x / 800f) * 2f) - 1f);
    let _e13: vec2<f32> = aPosition_1;
    ndc_y = (1f - ((_e13.y / 600f) * 2f));
    let _e22: f32 = ndc_x;
    let _e23: f32 = ndc_y;
    gl_Position = vec4<f32>(_e22, _e23, 0f, 1f);
    let _e27: vec4<f32> = aColor_1;
    vColor = _e27;
    return;
}

@vertex 
fn main(@location(0) aPosition: vec2<f32>, @location(1) aColor: vec4<f32>) -> VertexOutput {
    aPosition_1 = aPosition;
    aColor_1 = aColor;
    main_1();
    let _e11: vec4<f32> = vColor;
    let _e13: vec4<f32> = gl_Position;
    return VertexOutput(_e11, _e13);
}
