struct FragmentOutput {
    @location(0) fColor: vec4<f32>,
}

var<private> vColor_1: vec4<f32>;
var<private> fColor: vec4<f32>;

fn main_1() {
    let _e2: vec4<f32> = vColor_1;
    fColor = _e2;
    return;
}

@fragment 
fn main(@location(0) vColor: vec4<f32>) -> FragmentOutput {
    vColor_1 = vColor;
    main_1();
    let _e7: vec4<f32> = fColor;
    return FragmentOutput(_e7);
}
