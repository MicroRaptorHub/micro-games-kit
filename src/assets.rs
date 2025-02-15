use crate::game::GameSubsystem;
use anput::world::World;
use core::str;
use fontdue::Font;
use image::{GenericImage, GenericImageView, RgbaImage};
use keket::{
    database::{
        handle::AssetHandle,
        path::{AssetPath, AssetPathStatic},
        AssetDatabase,
    },
    fetch::{
        container::{ContainerAssetFetch, ContainerPartialFetch},
        AssetFetch,
    },
    protocol::{
        bytes::BytesAssetProtocol, group::GroupAssetProtocol, text::TextAssetProtocol,
        AssetProtocol,
    },
};
use kira::sound::static_sound::StaticSoundData;
use serde::{Deserialize, Serialize};
use spitfire_glow::renderer::GlowTextureFormat;
use std::{
    borrow::Cow,
    collections::HashMap,
    error::Error,
    io::{Cursor, Read, Write},
    ops::Range,
    path::Path,
};

pub fn name_from_path<'a>(path: &'a AssetPath<'a>) -> &'a str {
    path.meta_items()
        .find(|(key, _)| *key == "as")
        .map(|(_, value)| value)
        .unwrap_or(path.path())
}

pub fn make_database(fetch: impl AssetFetch) -> AssetDatabase {
    AssetDatabase::default()
        .with_protocol(BytesAssetProtocol)
        .with_protocol(TextAssetProtocol)
        .with_protocol(GroupAssetProtocol)
        .with_protocol(ShaderAssetProtocol)
        .with_protocol(TextureAssetProtocol)
        .with_protocol(FontAssetProtocol)
        .with_protocol(SoundAssetProtocol)
        .with_fetch(fetch)
}

pub fn make_memory_database(package: &[u8]) -> Result<AssetDatabase, Box<dyn Error>> {
    Ok(make_database(ContainerAssetFetch::new(
        AssetPackage::decode(package)?,
    )))
}

pub fn make_directory_database(
    directory: impl AsRef<Path>,
) -> Result<AssetDatabase, Box<dyn Error>> {
    Ok(make_database(ContainerAssetFetch::new(
        AssetPackage::from_directory(directory)?,
    )))
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AssetPackageRegistry {
    mappings: HashMap<String, Range<usize>>,
}

#[derive(Default)]
pub struct AssetPackage {
    registry: AssetPackageRegistry,
    content: Vec<u8>,
}

impl AssetPackage {
    pub fn from_directory(directory: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        fn visit_dirs(
            dir: &Path,
            root: &str,
            registry: &mut AssetPackageRegistry,
            content: &mut Cursor<Vec<u8>>,
        ) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_str().unwrap();
                    let name = if root.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{}/{}", root, name)
                    };
                    if path.is_dir() {
                        visit_dirs(&path, &name, registry, content)?;
                    } else {
                        let bytes = std::fs::read(path)?;
                        let start = content.position() as usize;
                        content.write_all(&bytes)?;
                        let end = content.position() as usize;
                        registry.mappings.insert(name, start..end);
                    }
                }
            }
            Ok(())
        }

        let directory = directory.as_ref();
        let mut registry = AssetPackageRegistry::default();
        let mut content = Cursor::new(Vec::default());
        visit_dirs(directory, "", &mut registry, &mut content)?;
        Ok(AssetPackage {
            registry,
            content: content.into_inner(),
        })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut stream = Cursor::new(bytes);
        let mut size = 0u32.to_be_bytes();
        stream.read_exact(&mut size)?;
        let size = u32::from_be_bytes(size) as usize;
        let mut registry = vec![0u8; size];
        stream.read_exact(&mut registry)?;
        let registry = toml::from_str(str::from_utf8(&registry)?)?;
        let mut content = Vec::default();
        stream.read_to_end(&mut content)?;
        Ok(Self { registry, content })
    }

    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut stream = Cursor::new(Vec::default());
        let registry = toml::to_string(&self.registry)?;
        let registry = registry.as_bytes();
        stream.write_all(&(registry.len() as u32).to_be_bytes())?;
        stream.write_all(registry)?;
        stream.write_all(&self.content)?;
        Ok(stream.into_inner())
    }

    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.registry.mappings.keys().map(|key| key.as_str())
    }
}

impl ContainerPartialFetch for AssetPackage {
    fn load_bytes(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        if let Some(range) = self.registry.mappings.get(path.path()).cloned() {
            if range.end <= self.content.len() {
                Ok(self.content[range].to_owned())
            } else {
                Err(format!(
                    "Asset: `{}` out of content bounds! Bytes range: {:?}, content byte size: {}",
                    path,
                    range,
                    self.content.len()
                )
                .into())
            }
        } else {
            Err(format!("Asset: `{}` not present in package!", path).into())
        }
    }
}

impl std::fmt::Debug for AssetPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetPackage")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

pub struct ShaderAsset {
    pub vertex: Cow<'static, str>,
    pub fragment: Cow<'static, str>,
}

impl ShaderAsset {
    pub fn new(vertex: &'static str, fragment: &'static str) -> Self {
        Self {
            vertex: vertex.into(),
            fragment: fragment.into(),
        }
    }
}

pub struct ShaderAssetSubsystem;

impl GameSubsystem for ShaderAssetSubsystem {
    fn run(&mut self, context: crate::context::GameContext, _: f32) {
        for entity in context.assets.storage.added().iter_of::<ShaderAsset>() {
            if let Some((path, asset)) = context
                .assets
                .storage
                .lookup_one::<true, (&AssetPathStatic, &ShaderAsset)>(entity)
            {
                context.draw.shaders.insert(
                    name_from_path(&path).to_owned().into(),
                    context
                        .graphics
                        .shader(asset.vertex.trim(), asset.fragment.trim())
                        .unwrap(),
                );
            }
        }
        for entity in context.assets.storage.removed().iter_of::<ShaderAsset>() {
            if let Some(path) = context
                .assets
                .storage
                .lookup_one::<true, &AssetPathStatic>(entity)
            {
                context.draw.shaders.remove(name_from_path(&path));
            }
        }
    }
}

pub struct ShaderAssetProtocol;

impl AssetProtocol for ShaderAssetProtocol {
    fn name(&self) -> &str {
        "shader"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        enum Mode {
            Vertex,
            Fragment,
        }

        let mut vertex = String::default();
        let mut fragment = String::default();
        let mut mode = Mode::Vertex;
        for line in std::str::from_utf8(&bytes)?.lines() {
            let trimmed = line.trim();
            if let Some(comment) = trimmed.strip_prefix("///") {
                let comment = comment.trim().to_lowercase();
                if comment == "[vertex]" {
                    mode = Mode::Vertex;
                    continue;
                }
                if comment == "[fragment]" {
                    mode = Mode::Fragment;
                    continue;
                }
            }
            match mode {
                Mode::Vertex => {
                    vertex.push_str(line);
                    vertex.push('\n');
                }
                Mode::Fragment => {
                    fragment.push_str(line);
                    fragment.push('\n');
                }
            }
        }

        storage.insert(
            handle.entity(),
            (ShaderAsset {
                vertex: vertex.into(),
                fragment: fragment.into(),
            },),
        )?;

        Ok(())
    }
}

pub struct TextureAsset {
    pub image: RgbaImage,
    pub cols: u32,
    pub rows: u32,
}

pub struct TextureAssetSubsystem;

impl GameSubsystem for TextureAssetSubsystem {
    fn run(&mut self, context: crate::context::GameContext, _: f32) {
        for entity in context.assets.storage.added().iter_of::<TextureAsset>() {
            if let Some((path, asset)) = context
                .assets
                .storage
                .lookup_one::<true, (&AssetPathStatic, &TextureAsset)>(entity)
            {
                let pages = asset.cols * asset.rows;
                context.draw.textures.insert(
                    name_from_path(&path).to_owned().into(),
                    context
                        .graphics
                        .texture(
                            asset.image.width(),
                            asset.image.height() / pages,
                            pages,
                            GlowTextureFormat::Rgba,
                            Some(asset.image.as_raw()),
                        )
                        .unwrap(),
                );
            }
        }
        for entity in context.assets.storage.removed().iter_of::<TextureAsset>() {
            if let Some(path) = context
                .assets
                .storage
                .lookup_one::<true, &AssetPathStatic>(entity)
            {
                context.draw.textures.remove(name_from_path(&path));
            }
        }
    }
}

pub struct TextureAssetProtocol;

impl AssetProtocol for TextureAssetProtocol {
    fn name(&self) -> &str {
        "texture"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let path = storage.component::<true, AssetPathStatic>(handle.entity())?;
        let mut cols = 1;
        let mut rows = 1;
        for (key, value) in path.meta_items() {
            if key == "cols" || key == "c" {
                cols = value.parse().unwrap_or(1);
            } else if key == "rows" || key == "r" {
                rows = value.parse().unwrap_or(1);
            }
        }
        let mut image = image::load_from_memory(&bytes)
            .map_err(|_| format!("Failed to load texture: {:?}", path.path()))?
            .into_rgba8();
        drop(path);
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

        storage.insert(handle.entity(), (TextureAsset { image, cols, rows },))?;

        Ok(())
    }
}

pub struct FontAsset {
    pub font: Font,
}

pub struct FontAssetSubsystem;

impl GameSubsystem for FontAssetSubsystem {
    fn run(&mut self, context: crate::context::GameContext, _: f32) {
        for entity in context.assets.storage.added().iter_of::<FontAsset>() {
            if let Some((path, asset)) = context
                .assets
                .storage
                .lookup_one::<true, (&AssetPathStatic, &FontAsset)>(entity)
            {
                context
                    .draw
                    .fonts
                    .insert(name_from_path(&path).to_owned(), asset.font.clone());
            }
        }
        for entity in context.assets.storage.removed().iter_of::<FontAsset>() {
            if let Some(path) = context
                .assets
                .storage
                .lookup_one::<true, &AssetPathStatic>(entity)
            {
                context.draw.fonts.remove(name_from_path(&path));
            }
        }
    }
}

pub struct FontAssetProtocol;

impl AssetProtocol for FontAssetProtocol {
    fn name(&self) -> &str {
        "font"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let path = storage.component::<true, AssetPathStatic>(handle.entity())?;
        let font = Font::from_bytes(bytes, Default::default())
            .map_err(|_| format!("Failed to load font: {:?}", path.path()))?;
        drop(path);

        storage.insert(handle.entity(), (FontAsset { font },))?;

        Ok(())
    }
}

pub struct SoundAsset {
    pub data: StaticSoundData,
}

pub struct SoundAssetSubsystem;

impl GameSubsystem for SoundAssetSubsystem {
    fn run(&mut self, context: crate::context::GameContext, _: f32) {
        for entity in context.assets.storage.added().iter_of::<SoundAsset>() {
            if let Some((path, asset)) = context
                .assets
                .storage
                .lookup_one::<true, (&AssetPathStatic, &SoundAsset)>(entity)
            {
                context
                    .audio
                    .sounds
                    .insert(name_from_path(&path).to_owned(), asset.data.clone());
            }
        }
        for entity in context.assets.storage.removed().iter_of::<SoundAsset>() {
            if let Some(path) = context
                .assets
                .storage
                .lookup_one::<true, &AssetPathStatic>(entity)
            {
                context.audio.sounds.remove(name_from_path(&path));
            }
        }
    }
}

pub struct SoundAssetProtocol;

impl AssetProtocol for SoundAssetProtocol {
    fn name(&self) -> &str {
        "sound"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let path = storage.component::<true, AssetPathStatic>(handle.entity())?;
        let data = StaticSoundData::from_cursor(Cursor::new(bytes))
            .map_err(|_| format!("Failed to load sound: {:?}", path.path()))?;
        drop(path);

        storage.insert(handle.entity(), (SoundAsset { data },))?;

        Ok(())
    }
}
