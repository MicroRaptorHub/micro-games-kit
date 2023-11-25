use fontdue::Font;
use image::{GenericImage, GenericImageView, RgbaImage};
use spitfire_draw::{context::DrawContext, utils::Vertex};
use spitfire_glow::{graphics::Graphics, renderer::GlowTextureFormat};
use std::{borrow::Cow, path::Path};

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
    path: impl AsRef<Path>,
    pages: u32,
    pages_per_row: u32,
) {
    let path = path.as_ref();
    let mut image = image::open(path)
        .unwrap_or_else(|_| panic!("Failed to load texture: {:?}", path))
        .into_rgba8();
    image = if pages > 1 {
        if pages_per_row > 1 {
            let width = image.width() / pages_per_row;
            let height = image.height();
            let mut result = RgbaImage::new(width, height * pages);
            for index in 0..pages {
                let view = image.view(index * width, 0, width, height);
                result.copy_from(&*view, 0, index * height).unwrap();
            }
            result
        } else {
            image
        }
    } else {
        image
    };
    draw.textures.insert(
        name.into(),
        graphics
            .texture(
                image.width(),
                image.height() / pages,
                pages,
                GlowTextureFormat::Rgba,
                image.as_raw(),
            )
            .unwrap(),
    );
}

pub fn load_textures<const N: usize>(
    draw: &mut DrawContext,
    graphics: &Graphics<Vertex>,
    // [id, path, pages count, pages per row count]
    items: [(&'static str, &str, u32, u32); N],
) {
    for (name, path, pages, pages_per_row) in items {
        load_texture(draw, graphics, name, path, pages, pages_per_row);
    }
}

pub fn load_font(
    draw: &mut DrawContext,
    name: impl Into<Cow<'static, str>>,
    path: impl AsRef<Path>,
) {
    let path = path.as_ref();
    let bytes = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to load font: {:?}", path));
    draw.fonts
        .insert(name, Font::from_bytes(bytes, Default::default()).unwrap());
}

pub fn load_fonts<const N: usize>(
    draw: &mut DrawContext,
    // [id, path]
    items: [(&'static str, &str); N],
) {
    for (name, path) in items {
        load_font(draw, name, path);
    }
}
