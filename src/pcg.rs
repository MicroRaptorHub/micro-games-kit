use noise::NoiseFn;
use std::ops::{Add, Div, Mul, Range, Sub};
use vek::{Mat4, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Clone)]
pub struct Grid<T: Copy> {
    size: Vec2<usize>,
    buffer: Vec<T>,
}

impl<T: Copy> Grid<T> {
    pub fn new(size: Vec2<usize>, fill_value: T) -> Self {
        Self {
            size,
            buffer: vec![fill_value; size.x * size.y],
        }
    }

    pub fn with_buffer(size: Vec2<usize>, buffer: Vec<T>) -> Option<Self> {
        if buffer.len() == size.x * size.y {
            Some(Self { size, buffer })
        } else {
            None
        }
    }

    pub fn generate(size: Vec2<usize>, generator: impl GridGenetator<T>) -> Self
    where
        T: Default,
    {
        let mut result = Self::new(size, Default::default());
        result.apply_all(generator);
        result
    }

    pub fn fork(&self, fill_value: T) -> Self {
        Self {
            size: self.size,
            buffer: vec![fill_value; self.size.x * self.size.y],
        }
    }

    pub fn fork_generate(&self, generator: impl GridGenetator<T>) -> Self {
        let mut result = self.clone();
        result.apply_all(generator);
        result
    }

    pub fn apply(
        &mut self,
        from: impl Into<Vec2<usize>>,
        to: impl Into<Vec2<usize>>,
        mut generator: impl GridGenetator<T>,
    ) {
        if self.buffer.is_empty() {
            return;
        }
        let from = from.into();
        let to = to.into();
        for y in from.y..to.y {
            for x in from.x..to.x {
                let location = Vec2::new(x, y);
                let index = self.index(location);
                self.buffer[index] =
                    generator.generate(location, self.size, self.buffer[index], self);
            }
        }
    }

    pub fn apply_all(&mut self, generator: impl GridGenetator<T>) {
        self.apply(0, self.size, generator);
    }

    pub fn map<U: Copy>(&self, mut f: impl FnMut(Vec2<usize>, Vec2<usize>, T) -> U) -> Grid<U> {
        Grid {
            size: self.size,
            buffer: self
                .buffer
                .iter()
                .enumerate()
                .map(|(index, value)| f(self.location(index), self.size, *value))
                .collect(),
        }
    }

    pub fn into_inner(self) -> (Vec2<usize>, Vec<T>) {
        (self.size, self.buffer)
    }

    pub fn size(&self) -> Vec2<usize> {
        self.size
    }

    pub fn buffer(&self) -> &[T] {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [T] {
        &mut self.buffer
    }

    pub fn iter(&self) -> impl Iterator<Item = (Vec2<usize>, usize, T)> + '_ {
        self.buffer
            .iter()
            .copied()
            .enumerate()
            .map(|(index, value)| (self.location(index), index, value))
    }

    pub fn index(&self, location: impl Into<Vec2<usize>>) -> usize {
        let location = location.into();
        (location.y % self.size.y) * self.size.x + (location.x % self.size.x)
    }

    pub fn location(&self, index: usize) -> Vec2<usize> {
        Vec2 {
            x: index % self.size.x,
            y: (index / self.size.y) % self.size.y,
        }
    }

    pub fn location_offset(
        &self,
        mut location: Vec2<usize>,
        direction: GridDirection,
        distance: usize,
    ) -> Option<Vec2<usize>> {
        if distance == 0 {
            return None;
        }
        match direction {
            GridDirection::North => {
                if let Some(y) = location.y.checked_sub(distance) {
                    location.y = y;
                } else {
                    return None;
                }
            }
            GridDirection::NorthEast => {
                if location.x + distance < self.size.x {
                    location.x += distance;
                } else {
                    return None;
                }
                if let Some(y) = location.y.checked_sub(distance) {
                    location.y = y;
                } else {
                    return None;
                }
            }
            GridDirection::East => {
                if location.x + distance < self.size.x {
                    location.x += distance;
                } else {
                    return None;
                }
            }
            GridDirection::SouthEast => {
                if location.x + distance < self.size.x {
                    location.x += distance;
                } else {
                    return None;
                }
                if location.y + distance < self.size.y {
                    location.y += distance;
                } else {
                    return None;
                }
            }
            GridDirection::South => {
                if location.y + distance < self.size.y {
                    location.y += distance;
                } else {
                    return None;
                }
            }
            GridDirection::SouthWest => {
                if let Some(x) = location.x.checked_sub(distance) {
                    location.x = x;
                } else {
                    return None;
                }
                if location.y + distance < self.size.y {
                    location.y += distance;
                } else {
                    return None;
                }
            }
            GridDirection::West => {
                if let Some(x) = location.x.checked_sub(distance) {
                    location.x = x;
                } else {
                    return None;
                }
            }
            GridDirection::NorthWest => {
                if let Some(x) = location.x.checked_sub(distance) {
                    location.x = x;
                } else {
                    return None;
                }
                if let Some(y) = location.y.checked_sub(distance) {
                    location.y = y;
                } else {
                    return None;
                }
            }
        }
        Some(location)
    }

    pub fn neighbors(
        &self,
        location: impl Into<Vec2<usize>>,
        range: Range<usize>,
    ) -> impl Iterator<Item = (GridDirection, Vec2<usize>, T)> + '_ {
        let location = location.into();
        range.flat_map(move |distance| {
            [
                GridDirection::North,
                GridDirection::NorthEast,
                GridDirection::East,
                GridDirection::SouthEast,
                GridDirection::South,
                GridDirection::SouthWest,
                GridDirection::West,
                GridDirection::NorthWest,
            ]
            .into_iter()
            .filter_map(move |direction| {
                let location = self.location_offset(location, direction, distance)?;
                Some((direction, location, self.get(location)?))
            })
        })
    }

    pub fn get(&self, location: impl Into<Vec2<usize>>) -> Option<T> {
        let index = self.index(location);
        self.buffer.get(index).copied()
    }

    pub fn set(&mut self, location: impl Into<Vec2<usize>>, value: T) {
        let index = self.index(location);
        if let Some(item) = self.buffer.get_mut(index) {
            *item = value;
        }
    }
}

pub trait GridGenetator<T: Copy> {
    fn generate(
        &mut self,
        location: Vec2<usize>,
        size: Vec2<usize>,
        current: T,
        grid: &Grid<T>,
    ) -> T;
}

impl<T: Copy, F: FnMut(Vec2<usize>, Vec2<usize>, T) -> T> GridGenetator<T> for F {
    fn generate(&mut self, location: Vec2<usize>, size: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        self(location, size, current)
    }
}

pub struct ConstGenerator<T: Copy>(pub T);

impl<T: Copy> GridGenetator<T> for ConstGenerator<T> {
    fn generate(&mut self, _: Vec2<usize>, _: Vec2<usize>, _: T, _: &Grid<T>) -> T {
        self.0
    }
}

pub struct OffsetLocationGenerator<'a, T: Copy> {
    pub generator: &'a mut dyn GridGenetator<T>,
    pub offsets: &'a Grid<Vec2<isize>>,
}

impl<T: Copy> GridGenetator<T> for OffsetLocationGenerator<'_, T> {
    fn generate(
        &mut self,
        mut location: Vec2<usize>,
        size: Vec2<usize>,
        current: T,
        grid: &Grid<T>,
    ) -> T {
        let offset = self.offsets.get(location).unwrap_or_default();
        if offset.x >= 0 {
            location.x = (location.x + offset.x as usize) % size.x;
        } else {
            location.x = (location.x + size.x - offset.x.unsigned_abs() % size.x) % size.x;
        }
        if offset.y >= 0 {
            location.y = (location.y + offset.y as usize) % size.y;
        } else {
            location.y = (location.y + size.y - offset.y.unsigned_abs() % size.y) % size.y;
        }
        self.generator.generate(location, size, current, grid)
    }
}

pub struct NoiseGenerator<T: NoiseFn<f64, 2>> {
    pub noise: T,
    pub transform: Mat4<f64>,
}

impl<T: NoiseFn<f64, 2>> NoiseGenerator<T> {
    pub fn new(noise: T) -> Self {
        Self {
            noise,
            transform: Mat4::identity(),
        }
    }

    pub fn transform(mut self, transform: Mat4<f64>) -> Self {
        self.transform = transform;
        self
    }
}

impl<T: NoiseFn<f64, 2>> GridGenetator<f64> for NoiseGenerator<T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, _: f64, _: &Grid<f64>) -> f64 {
        let point = self.transform.mul_point(Vec2 {
            x: location.x as f64,
            y: location.y as f64,
        });
        self.noise.get(point.into_array())
    }
}

pub struct CopyGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Add<Output = T> + Default> GridGenetator<T> for CopyGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, _: T, _: &Grid<T>) -> T {
        self.other.get(location).unwrap_or_default()
    }
}

pub struct AddGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Add<Output = T> + Default> GridGenetator<T> for AddGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current + self.other.get(location).unwrap_or_default()
    }
}

pub struct SubGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Sub<Output = T> + Default> GridGenetator<T> for SubGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current - self.other.get(location).unwrap_or_default()
    }
}

pub struct MulGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Mul<Output = T> + Default> GridGenetator<T> for MulGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current * self.other.get(location).unwrap_or_default()
    }
}

pub struct DivGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Div<Output = T> + Default> GridGenetator<T> for DivGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current / self.other.get(location).unwrap_or_default()
    }
}

pub struct MinGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Div<Output = T> + Ord + Default> GridGenetator<T> for MinGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current.min(self.other.get(location).unwrap_or_default())
    }
}

pub struct MaxGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<T: Copy + Div<Output = T> + Ord + Default> GridGenetator<T> for MaxGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current.max(self.other.get(location).unwrap_or_default())
    }
}

pub struct ClampGenerator<T: Copy> {
    pub min: T,
    pub max: T,
}

impl<T: Copy + Ord + Default> GridGenetator<T> for ClampGenerator<T> {
    fn generate(&mut self, _: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        current.clamp(self.min, self.max)
    }
}

pub struct RemapGenerator<T: Copy> {
    pub from: Range<T>,
    pub to: Range<T>,
}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>>
    GridGenetator<T> for RemapGenerator<T>
{
    fn generate(&mut self, _: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        let factor = (current - self.from.start) / (self.from.end - self.from.start);
        (self.to.end - self.to.start) * factor + self.to.start
    }
}

pub enum ThresholdGenerator<'a, T: Copy> {
    Constant {
        threshold: T,
        value_upper: T,
        value_lower: T,
    },
    Samples {
        thresholds: &'a Grid<T>,
        value_upper: T,
        value_lower: T,
    },
}

impl<T: Copy + PartialOrd + Default> GridGenetator<T> for ThresholdGenerator<'_, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T, _: &Grid<T>) -> T {
        match self {
            Self::Constant {
                threshold,
                value_upper,
                value_lower,
            } => {
                if current > *threshold {
                    *value_upper
                } else {
                    *value_lower
                }
            }
            Self::Samples {
                thresholds,
                value_upper,
                value_lower,
            } => {
                if current > thresholds.get(location).unwrap_or_default() {
                    *value_upper
                } else {
                    *value_lower
                }
            }
        }
    }
}

pub struct Kernel33Generator<'a, T: Copy> {
    pub other: &'a Grid<T>,
    pub kernel: [T; 9],
}

impl<'a> Kernel33Generator<'a, f64> {
    pub fn identity(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn ridge(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0],
        }
    }

    pub fn edge_detection(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [-1.0, -1.0, -1.0, -1.0, 8.0, -1.0, -1.0, -1.0, -1.0],
        }
    }

    pub fn sharpen(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0],
        }
    }

    pub fn emboss(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [-2.0, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0],
        }
    }

    pub fn box_blur(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
                1.0 / 9.0,
            ],
        }
    }

    pub fn gaussian_blur(other: &'a Grid<f64>) -> Self {
        Self {
            other,
            kernel: [
                1.0 / 16.0,
                2.0 / 16.0,
                1.0 / 16.0,
                2.0 / 16.0,
                4.0 / 16.0,
                2.0 / 16.0,
                1.0 / 16.0,
                2.0 / 16.0,
                1.0 / 16.0,
            ],
        }
    }
}

impl<T: Copy + Add<Output = T> + Mul<Output = T> + Default> GridGenetator<T>
    for Kernel33Generator<'_, T>
{
    fn generate(&mut self, location: Vec2<usize>, size: Vec2<usize>, _: T, _: &Grid<T>) -> T {
        let region = [
            self.other
                .get(location + Vec2::new(size.x - 1, size.y - 1))
                .unwrap_or_default(),
            self.other
                .get(location + Vec2::new(0, size.y - 1))
                .unwrap_or_default(),
            self.other
                .get(location + Vec2::new(1, size.y - 1))
                .unwrap_or_default(),
            self.other
                .get(location + Vec2::new(size.x - 1, 0))
                .unwrap_or_default(),
            self.other.get(location).unwrap_or_default(),
            self.other
                .get(location + Vec2::new(1, 0))
                .unwrap_or_default(),
            self.other
                .get(location + Vec2::new(size.x - 1, 1))
                .unwrap_or_default(),
            self.other
                .get(location + Vec2::new(0, 1))
                .unwrap_or_default(),
            self.other
                .get(location + Vec2::new(1, 1))
                .unwrap_or_default(),
        ];
        region
            .into_iter()
            .zip(self.kernel)
            .fold(Default::default(), |accumulator, (value, kernel)| {
                value * kernel + accumulator
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Grid, GridDirection, GridGenetator, Kernel33Generator, NoiseGenerator,
        OffsetLocationGenerator, RemapGenerator, SubGenerator, ThresholdGenerator,
    };
    use image::{GrayImage, RgbImage};
    use noise::{Fbm, MultiFractal, ScalePoint, SuperSimplex, Worley};
    use vek::Vec2;

    const SIZE: usize = 512;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Terrain {
        Water,
        Sand,
        Grass,
        Mountain,
    }

    fn gradient_generator(location: Vec2<usize>, size: Vec2<usize>, _: f64) -> f64 {
        let center = size / 2;
        let x = if location.x >= center.x {
            location.x - center.x
        } else {
            center.x - location.x
        } as f64;
        let y = if location.y >= center.y {
            location.y - center.y
        } else {
            center.y - location.y
        } as f64;
        let result = (x / center.x as f64).max(y / center.y as f64);
        result * result
    }

    struct OffsetsGenerator<'a> {
        pub source: &'a Grid<f64>,
        pub scale: f64,
    }

    impl GridGenetator<Vec2<isize>> for OffsetsGenerator<'_> {
        fn generate(
            &mut self,
            location: Vec2<usize>,
            _: Vec2<usize>,
            _: Vec2<isize>,
            _: &Grid<Vec2<isize>>,
        ) -> Vec2<isize> {
            let left = self
                .source
                .get(
                    self.source
                        .location_offset(location, GridDirection::West, 1)
                        .unwrap_or(location),
                )
                .unwrap_or_default();
            let right = self
                .source
                .get(
                    self.source
                        .location_offset(location, GridDirection::East, 1)
                        .unwrap_or(location),
                )
                .unwrap_or_default();
            let top = self
                .source
                .get(
                    self.source
                        .location_offset(location, GridDirection::North, 1)
                        .unwrap_or(location),
                )
                .unwrap_or_default();
            let bottom = self
                .source
                .get(
                    self.source
                        .location_offset(location, GridDirection::South, 1)
                        .unwrap_or(location),
                )
                .unwrap_or_default();
            Vec2 {
                x: ((right - left) * self.scale) as isize,
                y: ((bottom - top) * self.scale) as isize,
            }
        }
    }

    fn generate_terrain(size: Vec2<usize>) -> Grid<Terrain> {
        let mut grid = Grid::<f64>::generate(
            size,
            NoiseGenerator::new(
                Fbm::<SuperSimplex>::default()
                    .set_octaves(9)
                    .set_frequency(0.008),
            ),
        );
        grid.apply_all(RemapGenerator {
            from: -1.0..1.0,
            to: 0.0..1.0,
        });
        let gradient = grid.fork_generate(gradient_generator);
        grid.apply_all(SubGenerator { other: &gradient });
        grid.map(|_, _, value| {
            if value > 0.5 {
                Terrain::Mountain
            } else if value > 0.2 {
                Terrain::Grass
            } else if value > 0.15 {
                Terrain::Sand
            } else {
                Terrain::Water
            }
        })
    }

    fn generate_tunnels(size: Vec2<usize>) -> Grid<bool> {
        let offsets = Grid::<f64>::generate(
            size,
            NoiseGenerator::new(ScalePoint::new(SuperSimplex::default()).set_scale(0.04)),
        );
        let offsets = Grid::<Vec2<isize>>::generate(
            offsets.size(),
            OffsetsGenerator {
                source: &offsets,
                scale: 20.0,
            },
        );
        let mut thresholds = Grid::<f64>::generate(
            size,
            NoiseGenerator::new(ScalePoint::new(SuperSimplex::default()).set_scale(0.02)),
        );
        thresholds.apply_all(RemapGenerator {
            from: -1.0..1.0,
            to: 0.0..0.4,
        });
        let mut grid = Grid::<f64>::generate(
            size,
            OffsetLocationGenerator {
                generator: &mut NoiseGenerator::new(Worley::default().set_frequency(0.04)),
                offsets: &offsets,
            },
        );
        grid.apply_all(RemapGenerator {
            from: -1.0..1.0,
            to: 0.0..1.0,
        });
        grid.apply_all(Kernel33Generator::edge_detection(&grid.clone()));
        grid.apply_all(ThresholdGenerator::Constant {
            threshold: 1.0e-4,
            value_upper: 1.0,
            value_lower: 0.0,
        });
        for _ in 0..1 {
            grid.apply_all(Kernel33Generator::gaussian_blur(&grid.clone()));
        }
        grid.apply_all(ThresholdGenerator::Samples {
            thresholds: &thresholds,
            value_upper: 1.0,
            value_lower: 0.0,
        });
        grid.map(|_, _, value| value >= 0.5)
    }

    #[test]
    fn test_pcg_island() {
        let terrain = generate_terrain(SIZE.into());
        let tunnels = generate_tunnels(SIZE.into());

        let (size, buffer) = terrain.into_inner();
        let buffer = buffer
            .into_iter()
            .enumerate()
            .flat_map(|(index, value)| match value {
                Terrain::Mountain => {
                    let location = tunnels.location(index);
                    if tunnels.get(location).unwrap() {
                        [64, 64, 64]
                    } else {
                        [128, 128, 128]
                    }
                }
                Terrain::Grass => [0, 128, 0],
                Terrain::Sand => [192, 192, 128],
                Terrain::Water => [0, 0, 128],
            })
            .collect();
        let image = RgbImage::from_vec(size.x as _, size.y as _, buffer).unwrap();
        image.save("./resources/island.png").unwrap();
    }

    #[test]
    fn test_pcg_tunnels() {
        let tunnels = generate_tunnels(SIZE.into());

        let (size, buffer) = tunnels.into_inner();
        let buffer = buffer
            .into_iter()
            .map(|value| if value { 255 } else { 0 })
            .collect();
        let image = GrayImage::from_vec(size.x as _, size.y as _, buffer).unwrap();
        image.save("./resources/caves.png").unwrap();
    }
}
