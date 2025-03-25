use anput::world::World;
use keket::{database::handle::AssetHandle, protocol::AssetProtocol};
use std::{collections::HashMap, error::Error};
use vek::Rect;

#[derive(Debug, Default, Clone)]
pub struct AtlasAsset {
    pub image: String,
    pub width: usize,
    pub height: usize,
    pub regions: HashMap<String, AtlasAssetRegion>,
}

impl AtlasAsset {
    pub fn parse(content: &str) -> Self {
        enum Mode {
            AtlasPath,
            AtlasParams,
            Regions,
        }
        let mut result = Self::default();
        let mut mode = Mode::AtlasPath;
        let mut region = String::default();
        for line in content.lines() {
            if line.starts_with(char::is_whitespace) {
                if let Some((key, values)) = Self::parse_key_values(line) {
                    match mode {
                        Mode::AtlasParams => {
                            if key == "size" && values.len() == 2 {
                                result.width = values[0].parse().unwrap_or_default();
                                result.height = values[1].parse().unwrap_or_default();
                            }
                        }
                        Mode::Regions => {
                            if let Some(region) = result.regions.get_mut(&region) {
                                if key == "bounds" && values.len() == 4 {
                                    region.x = values[0].parse().unwrap_or_default();
                                    region.y = values[1].parse().unwrap_or_default();
                                    region.width = values[2].parse().unwrap_or_default();
                                    region.height = values[3].parse().unwrap_or_default();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                match mode {
                    Mode::AtlasPath => {
                        mode = Mode::AtlasParams;
                        result.image = line.trim().into();
                    }
                    Mode::AtlasParams | Mode::Regions => {
                        mode = Mode::Regions;
                        region = line.trim().to_owned();
                        result.regions.insert(region.to_owned(), Default::default());
                    }
                }
            }
        }
        result
    }

    fn parse_key_values(line: &str) -> Option<(String, Vec<String>)> {
        let key_values = line.trim().split(':').collect::<Vec<_>>();
        if key_values.len() != 2 {
            return None;
        }
        let key = key_values[0].trim().to_owned();
        let values = key_values[1]
            .split(',')
            .map(|part| part.trim().to_owned())
            .collect::<Vec<_>>();
        Some((key, values))
    }

    pub fn uvs(&self, region: &str) -> Option<Rect<f32, f32>> {
        let region = self.regions.get(region)?;
        Some(Rect::new(
            region.x as f32 / self.width as f32,
            region.y as f32 / self.height as f32,
            region.width as f32 / self.width as f32,
            region.height as f32 / self.height as f32,
        ))
    }
}

#[derive(Debug, Default, Clone)]
pub struct AtlasAssetRegion {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct AtlasAssetProtocol;

impl AssetProtocol for AtlasAssetProtocol {
    fn name(&self) -> &str {
        "atlas"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let atlas = AtlasAsset::parse(std::str::from_utf8(&bytes)?);

        storage.insert(handle.entity(), (atlas,))?;

        Ok(())
    }
}
