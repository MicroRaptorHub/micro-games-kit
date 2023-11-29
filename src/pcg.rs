use noise::NoiseFn;
use std::ops::{Add, Div, Mul, Range, Sub};
use vek::{Mat4, Vec2};

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
                self.buffer[index] = generator.generate(location, self.size, self.buffer[index]);
            }
        }
    }

    pub fn apply_all(&mut self, generator: impl GridGenetator<T>) {
        self.apply(0, self.size, generator);
    }

    pub fn map<U: Copy>(&self, mut f: impl FnMut(T) -> U) -> Grid<U> {
        Grid {
            size: self.size,
            buffer: self.buffer.iter().map(|value| f(*value)).collect(),
        }
    }
}

pub trait GridGenetator<T> {
    fn generate(&mut self, location: Vec2<usize>, size: Vec2<usize>, current: T) -> T;
}

impl<T, F: FnMut(Vec2<usize>, Vec2<usize>, T) -> T> GridGenetator<T> for F {
    fn generate(&mut self, location: Vec2<usize>, size: Vec2<usize>, current: T) -> T {
        self(location, size, current)
    }
}

pub struct ConstGenerator<T: Copy>(pub T);

impl<T: Copy> GridGenetator<T> for ConstGenerator<T> {
    fn generate(&mut self, _: Vec2<usize>, _: Vec2<usize>, _: T) -> T {
        self.0
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
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, _: f64) -> f64 {
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

impl<'a, T: Copy + Add<Output = T> + Default> GridGenetator<T> for CopyGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, _: T) -> T {
        self.other.get(location).unwrap_or_default()
    }
}

pub struct AddGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<'a, T: Copy + Add<Output = T> + Default> GridGenetator<T> for AddGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        current + self.other.get(location).unwrap_or_default()
    }
}

pub struct SubGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<'a, T: Copy + Sub<Output = T> + Default> GridGenetator<T> for SubGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        current - self.other.get(location).unwrap_or_default()
    }
}

pub struct MulGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<'a, T: Copy + Mul<Output = T> + Default> GridGenetator<T> for MulGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        current * self.other.get(location).unwrap_or_default()
    }
}

pub struct DivGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<'a, T: Copy + Div<Output = T> + Default> GridGenetator<T> for DivGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        current / self.other.get(location).unwrap_or_default()
    }
}

pub struct MinGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<'a, T: Copy + Div<Output = T> + Ord + Default> GridGenetator<T> for MinGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        current.min(self.other.get(location).unwrap_or_default())
    }
}

pub struct MaxGenerator<'a, T: Copy> {
    pub other: &'a Grid<T>,
}

impl<'a, T: Copy + Div<Output = T> + Ord + Default> GridGenetator<T> for MaxGenerator<'a, T> {
    fn generate(&mut self, location: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        current.max(self.other.get(location).unwrap_or_default())
    }
}

pub struct RemapGenerator<T: Copy> {
    pub from: Range<T>,
    pub to: Range<T>,
}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>>
    GridGenetator<T> for RemapGenerator<T>
{
    fn generate(&mut self, _: Vec2<usize>, _: Vec2<usize>, current: T) -> T {
        let factor = (current - self.from.start) / (self.from.end - self.from.start);
        (self.to.end - self.to.start) * factor + self.to.start
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

impl<'a, T: Copy + Add<Output = T> + Mul<Output = T> + Default> GridGenetator<T>
    for Kernel33Generator<'a, T>
{
    fn generate(&mut self, location: Vec2<usize>, size: Vec2<usize>, _: T) -> T {
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
    use super::{Grid, NoiseGenerator, RemapGenerator, SubGenerator};
    use image::RgbImage;
    use noise::{Fbm, MultiFractal, SuperSimplex};
    use vek::Vec2;

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

    #[test]
    fn test_pcg() {
        let mut grid = Grid::<f64>::generate(
            512.into(),
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

        let (size, buffer) = grid.into_inner();
        let buffer = buffer
            .into_iter()
            .flat_map(|value| {
                if value > 0.7 {
                    [255, 255, 255]
                } else if value > 0.5 {
                    [128, 128, 128]
                } else if value > 0.2 {
                    [0, 128, 0]
                } else if value > 0.15 {
                    [192, 192, 128]
                } else {
                    [0, 0, 128]
                }
            })
            .collect();
        let image = RgbImage::from_vec(size.x as _, size.y as _, buffer).unwrap();
        image.save("./resources/island.png").unwrap();
    }
}
