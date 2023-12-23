use crate::pcg::Grid;
use spitfire_draw::{
    context::DrawContext,
    tiles::{TileInstance, TileMap, TileSet, TilesEmitter},
    utils::{Drawable, Vertex},
};
use spitfire_glow::graphics::Graphics;
use std::{
    any::{Any, TypeId},
    ops::Range,
};
use vek::{Rect, Vec2};

pub trait GridWorldEmitterFilter: Any {
    fn filter(&self, tile: &TileInstance) -> bool;
}

impl GridWorldEmitterFilter for () {
    fn filter(&self, _: &TileInstance) -> bool {
        true
    }
}

#[derive(Debug, Default)]
pub struct InRangeFilter {
    pub location: Vec2<usize>,
    pub range: usize,
    pub clear_outside: bool,
}

impl GridWorldEmitterFilter for InRangeFilter {
    fn filter(&self, tile: &TileInstance) -> bool {
        let status = tile.location.distance_squared(self.location) > self.range * self.range;
        status != self.clear_outside
    }
}

pub struct GridWorldLayer {
    pub tilemap: TileMap,
    filter: Box<dyn GridWorldEmitterFilter>,
    filter_type: TypeId,
}

impl GridWorldLayer {
    pub fn new(tilemap: TileMap) -> Self {
        Self {
            tilemap,
            filter: Box::new(()),
            filter_type: TypeId::of::<()>(),
        }
    }

    pub fn new_filtered<F: GridWorldEmitterFilter + 'static>(tilemap: TileMap, filter: F) -> Self {
        Self {
            tilemap,
            filter: Box::new(filter),
            filter_type: TypeId::of::<F>(),
        }
    }

    pub fn access_filter<F: GridWorldEmitterFilter + 'static>(&mut self) -> Option<&mut F> {
        if self.filter_type == TypeId::of::<F>() {
            let result = &mut *self.filter as *mut dyn GridWorldEmitterFilter as *mut F;
            unsafe { Some(&mut *result) }
        } else {
            None
        }
    }
}

pub struct GridWorld {
    pub position: Vec2<f32>,
    pub pivot: Vec2<f32>,
    pub tile_size: Vec2<f32>,
    pub tileset: TileSet,
    pub visible_layers: Range<usize>,
    map_layers: Vec<GridWorldLayer>,
    tile_instances: Vec<TileInstance>,
    colliders: Grid<bool>,
}

impl GridWorld {
    pub fn new(tile_size: Vec2<f32>, tileset: TileSet, terrain_layer: GridWorldLayer) -> Self {
        let size = terrain_layer.tilemap.size();
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

    pub fn with_layer(mut self, layer: GridWorldLayer) -> Self {
        if layer.tilemap.size() == self.map_layers[0].tilemap.size() {
            self.map_layers.push(layer);
        }
        self
    }

    pub fn with_visible_layer(mut self, layer: GridWorldLayer) -> Self {
        self = self.with_layer(layer);
        self.visible_layers.end = self.map_layers.len();
        self
    }

    pub fn with_tile_instance(mut self, instance: TileInstance) -> Self {
        self.insert_tile_instance(instance);
        self
    }

    pub fn with_tile_instances(
        mut self,
        instances: impl IntoIterator<Item = TileInstance>,
    ) -> Self {
        for instance in instances {
            self.insert_tile_instance(instance);
        }
        self
    }

    pub fn with_colliders(mut self, grid: Grid<bool>) -> Self {
        if self.colliders.size() == grid.size() {
            self.colliders = grid;
        }
        self
    }

    pub fn with_collider(mut self, location: Vec2<usize>) -> Self {
        self.set_collider(location, true);
        self
    }

    pub fn insert_tile_instance(&mut self, instance: TileInstance) {
        if self.tile_instances.len() == self.tile_instances.capacity() {
            self.tile_instances.reserve(self.tile_instances.capacity());
        }
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

    pub fn layers(&self) -> &[GridWorldLayer] {
        &self.map_layers
    }

    pub fn layers_mut(&mut self) -> &mut [GridWorldLayer] {
        &mut self.map_layers
    }

    pub fn locations_iter(&self) -> impl Iterator<Item = Vec2<usize>> {
        let size = self.map_layers[0].tilemap.size();
        (0..size.y).flat_map(move |y| (0..size.x).map(move |x| Vec2 { x, y }))
    }

    pub fn world_to_local(&self, location: Vec2<f32>) -> Option<Vec2<usize>> {
        let size = self.map_layers[0].tilemap.size();
        let result = location - self.position
            + Vec2::new(size.x as f32, size.y as f32) * self.tile_size * self.pivot;
        let result = result / self.tile_size;
        if result.x >= 0.0 && result.y >= 0.0 {
            let result = Vec2::new(result.x as usize, result.y as usize);
            if result.x < size.x && result.y < size.y {
                return Some(result);
            }
        }
        None
    }

    pub fn local_to_world(&self, location: Vec2<usize>) -> Vec2<f32> {
        let size = self.map_layers[0].tilemap.size();
        Vec2::new(location.x as f32, location.y as f32) * self.tile_size + self.position
            - Vec2::new(size.x as f32, size.y as f32) * self.tile_size * self.pivot
    }
}

impl Drawable for GridWorld {
    fn draw(&self, context: &mut DrawContext, graphics: &mut Graphics<Vertex>) {
        let size = self.map_layers[0].tilemap.size();
        let rectangle = graphics.main_camera.world_rectangle();
        let offset = (rectangle.position() - self.position) / self.tile_size;
        let extent = rectangle.extent() / self.tile_size;
        let region = Rect {
            x: offset.x as usize,
            y: offset.y as usize,
            w: extent.w.ceil() as usize + 1,
            h: extent.h.ceil() as usize + 1,
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
                    .flat_map(|layer| {
                        layer
                            .tilemap
                            .emit_region(region, false)
                            .filter(|tile| layer.filter.filter(tile))
                    })
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
