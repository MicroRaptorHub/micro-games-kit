use spitfire_draw::utils::TextureRef;
use std::{collections::HashMap, ops::Range};
use vek::Rect;

#[derive(Debug, Clone, PartialEq)]
pub struct FrameAnimation {
    frames: Range<usize>,
    current: Option<usize>,
    accumulator: f32,
    pub fps: f32,
    pub is_playing: bool,
    pub looping: bool,
}

impl FrameAnimation {
    pub fn new(frames: Range<usize>) -> Self {
        Self {
            frames,
            current: None,
            accumulator: 0.0,
            fps: 30.0,
            is_playing: false,
            looping: false,
        }
    }

    pub fn fps(mut self, value: f32) -> Self {
        self.fps = value;
        self
    }

    pub fn playing(mut self) -> Self {
        self.play();
        self
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn play(&mut self) {
        if self.frames.is_empty() {
            return;
        }
        self.is_playing = true;
        self.accumulator = 0.0;
        self.current = Some(self.frames.start);
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.accumulator = 0.0;
        self.current = None;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.frames.is_empty() || !self.is_playing {
            return;
        }
        let Some(mut current) = self.current else {
            return;
        };
        self.accumulator += (delta_time * self.fps).max(0.0);
        while self.accumulator >= 1.0 {
            self.accumulator -= 1.0;
            current += 1;
            if current >= self.frames.end {
                if self.looping {
                    current = self.frames.start;
                } else {
                    self.stop();
                    return;
                }
            }
        }
        self.current = Some(current);
    }

    pub fn current_frame(&self) -> Option<usize> {
        self.current
    }
}

#[derive(Debug, Clone)]
pub struct NamedAnimation {
    pub animation: FrameAnimation,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct SpriteAnimationFrame {
    pub texture: TextureRef,
    pub region: Rect<f32, f32>,
    pub page: f32,
}

#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    pub animation: FrameAnimation,
    pub frames: HashMap<usize, SpriteAnimationFrame>,
}

impl SpriteAnimation {
    pub fn current_frame(&self) -> Option<&SpriteAnimationFrame> {
        self.frames.get(&self.animation.current_frame()?)
    }
}
