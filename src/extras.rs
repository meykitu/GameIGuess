use image::GenericImageView;

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub(crate) pos: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
}

pub fn load_image_bytes(path: &str) -> (Vec<u8>, u32, u32) {
    let img = image::open(path).expect("Failed to load image");
    let (width, height) = img.dimensions();
    let raw_bytes = img.to_rgba8().into_raw();
    (raw_bytes, width, height)
}