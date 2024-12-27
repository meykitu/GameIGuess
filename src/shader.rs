use miniquad::*;

#[repr(C)]
pub struct UniformsDefault {
    pub mvp: [[f32; 4]; 4],
}

#[repr(C)]
pub struct UniformsShadow {
    pub mvp: [[f32; 4]; 4],
    pub light_mvp: [[f32; 4]; 4],
}