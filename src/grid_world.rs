use crate::pcg::Grid;
use spitfire_draw::{
    context::DrawContext,
    tiles::{TileInstance, TileMap, TileSet, TilesEmitter},
    utils::{Drawable, Vertex},
};
use spitfire_glow::graphics::Graphics;
use std::ops::Range;
use vek::{Rect, Vec2};

#[derive(Clone)]
pub struct GridWorld {
    pub position: Vec2<f32>,
    pub pivot: Vec2<f32>,
    pub tile_size: Vec2<f32>,
    pub tileset: TileSet,
    pub visible_layers: Range<usize>,
    map_layers: Vec<TileMap>,
    tile_instances: Vec<TileInstance>,
    colliders: Grid<bool>,
}

impl GridWorld {
    pub fn new(tile_size: Vec2<f32>, tileset: TileSet, terrain_layer: TileMap) -> Self {
        let size = terrain_layer.size();
        Self {
            position: Default::default(),
            pivot: Default::default(),
            tile_size,
            tileset,
            visible_layers: 0..1,
            map_layers: vec![terrain_layer],
            tile_instances: Default::default(),
            colliders: Grid::new(size, false),
        }
    }

    pub fn with_position(mut self, value: Vec2<f32>) -> Self {
        self.position = value;
        self
    }

    pub fn with_pivot(mut self, value: Vec2<f32>) -> Self {
        self.pivot = value;
        self
    }

    pub fn with_visible_layers(mut self, value: Range<usize>) -> Self {
        self.visible_layers = value;
        self
    }

    pub fn with_map_layer(mut self, tilemap: TileMap) -> Self {
        if tilemap.size() == self.map_layers[0].size() {
            self.map_layers.push(tilemap);
        }
        self
    }

    pub fn with_visible_map_layer(mut self, tilemap: TileMap) -> Self {
        self = self.with_map_layer(tilemap);
        self.visible_layers.end = self.map_layers.len();
        self
    }

    pub fn with_tile_instance(mut self, instance: TileInstance) -> Self {
        self.insert_tile_instance(instance);
        self
    }

    pub fn with_collider(mut self, location: Vec2<usize>) -> Self {
        self.set_collider(location, true);
        self
    }

    pub fn insert_tile_instance(&mut self, instance: TileInstance) {
        let index = self
            .tile_instances
            .binary_search_by(|item| item.location.yx().cmp(&instance.location.yx()))
            .map_or_else(|index| index, |index| index);
        self.tile_instances.insert(index, instance);
    }

    pub fn remove_tile_instances(&mut self, instance: &TileInstance) {
        while let Some(index) = self.tile_instances.iter().position(|item| item == instance) {
            self.tile_instances.remove(index);
        }
    }

    pub fn remove_tile_instances_at_location(&mut self, location: Vec2<usize>) {
        while let Some(index) = self
            .tile_instances
            .iter()
            .position(|item| item.location == location)
        {
            self.tile_instances.remove(index);
        }
    }

    pub fn collider(&self, location: Vec2<usize>) -> bool {
        self.colliders.get(location).unwrap_or_default()
    }

    pub fn set_collider(&mut self, location: Vec2<usize>, value: bool) {
        self.colliders.set(location, value);
    }
}

impl Drawable for GridWorld {
    fn draw(&self, context: &mut DrawContext, graphics: &mut Graphics<Vertex>) {
        let size = self.map_layers[0].size();
        let rectangle = graphics.main_camera.world_rectangle();
        let offset = (rectangle.position() - self.position) / self.tile_size;
        let extent = rectangle.extent() / self.tile_size;
        let region = Rect {
            x: (offset.x as usize).clamp(0, size.x),
            y: (offset.y as usize).clamp(0, size.y),
            w: (extent.w.ceil() as usize).clamp(0, size.x),
            h: (extent.h.ceil() as usize).clamp(0, size.y),
        };
        let offset = Vec2::new(size.x as f32, size.y as f32) * self.tile_size * self.pivot;

        TilesEmitter::default()
            .position(self.position - offset)
            .tile_size(self.tile_size)
            .emit(
                &self.tileset,
                self.visible_layers
                    .clone()
                    .filter_map(|index| self.map_layers.get(index))
                    .flat_map(|layer| layer.emit_region(region))
                    .chain(
                        self.tile_instances
                            .iter()
                            .filter(|instance| {
                                self.tileset
                                    .mappings
                                    .get(&instance.id)
                                    .map(|item| {
                                        region.collides_with_rect(Rect {
                                            x: instance.location.x,
                                            y: instance.location.y,
                                            w: item.size.x,
                                            h: item.size.y,
                                        })
                                    })
                                    .unwrap_or_default()
                            })
                            .cloned(),
                    ),
            )
            .draw(context, graphics);
    }
}
