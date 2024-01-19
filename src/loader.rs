use fontdue::Font;
use image::{GenericImage, GenericImageView, RgbaImage};
use spitfire_draw::{context::DrawContext, utils::Vertex};
use spitfire_glow::{graphics::Graphics, renderer::GlowTextureFormat};
use std::borrow::Cow;

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! load_asset {
    ($path:literal) => {
        std::fs::read($path)
            .unwrap_or_else(|_| panic!("Failed to load binary asset: {}", $path))
            .as_slice()
    };
    (str $path:literal) => {
        std::fs::read_to_string($path)
            .unwrap_or_else(|_| panic!("Failed to load string asset: {}", $path))
            .as_str()
    };
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! load_asset {
    ($path:literal) => {
        include_bytes!($path)
    };
    (str $path:literal) => {
        include_str!($path)
    };
}

pub fn load_shader(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    name: impl Into<Cow<'static, str>>,
    vertex: &str,
    fragment: &str,
) {
    draw.shaders
        .insert(name.into(), graphics.shader(vertex, fragment).unwrap());
}

pub fn load_shaders<const N: usize>(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    // [id, vertex, fragment]
    items: [(&'static str, &str, &str); N],
) {
    for (name, vertex, fragment) in items {
        load_shader(draw, graphics, name, vertex, fragment);
    }
}

pub fn load_texture(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    name: impl Into<Cow<'static, str>>,
    bytes: &[u8],
    cols: u32,
    rows: u32,
) {
    let name = name.into();
    let image = image::load_from_memory(bytes)
        .unwrap_or_else(|_| panic!("Failed to load texture: {:?}", name))
        .into_rgba8();
    build_texture(draw, graphics, name, image, cols, rows)
}

pub fn load_textures<const N: usize>(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    // [id, bytes, columns, rows]
    items: [(&'static str, &[u8], u32, u32); N],
) {
    for (name, bytes, cols, rows) in items {
        load_texture(draw, graphics, name, bytes, cols, rows);
    }
}

pub fn build_texture(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    name: impl Into<Cow<'static, str>>,
    mut image: RgbaImage,
    cols: u32,
    rows: u32,
) {
    let name = name.into();
    let pages = cols * rows;
    image = if cols > 1 || rows > 1 {
        let width = image.width() / cols;
        let height = image.height() / rows;
        let mut result = RgbaImage::new(width, height * pages);
        for row in 0..rows {
            for col in 0..cols {
                let view = image.view(col * width, row * height, width, height);
                result
                    .copy_from(&*view, 0, (row * cols + col) * height)
                    .unwrap();
            }
        }
        result
    } else {
        image
    };
    draw.textures.insert(
        name,
        graphics
            .texture(
                image.width(),
                image.height() / pages,
                pages,
                GlowTextureFormat::Rgba,
                Some(image.as_raw()),
            )
            .unwrap(),
    );
}

pub fn build_textures<const N: usize>(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    // [id, image, columns, rows]
    items: [(&'static str, RgbaImage, u32, u32); N],
) {
    for (name, image, cols, rows) in items {
        build_texture(draw, graphics, name, image, cols, rows);
    }
}

pub fn load_font(draw: &mut DrawContext, name: impl Into<Cow<'static, str>>, bytes: &[u8]) {
    let name = name.into();
    let font = Font::from_bytes(bytes, Default::default())
        .unwrap_or_else(|_| panic!("Failed to load font: {:?}", name));
    draw.fonts.insert(name, font);
}

pub fn load_fonts<const N: usize>(
    draw: &mut DrawContext,
    // [id, bytes]
    items: [(&'static str, &[u8]); N],
) {
    for (name, bytes) in items {
        load_font(draw, name, bytes);
    }
}
