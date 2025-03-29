use crate::assets::spine::SpineAsset;
use rusty_spine::{
    controller::SkeletonController, AnimationEvent, AnimationStateData, BlendMode, Physics,
};
use spitfire_core::Triangle;
use spitfire_draw::{
    context::DrawContext,
    sprite::SpriteTexture,
    utils::{Drawable, ShaderRef, TextureRef, Vertex},
};
use spitfire_glow::{
    graphics::Graphics,
    graphics::GraphicsBatch,
    renderer::{GlowBlending, GlowUniformValue},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{channel, Receiver},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

pub enum SpineEvent {
    Start,
    Interrupt,
    End,
    Complete,
    Dispose,
    Event {
        /// The name of the event, which is unique across all events in the skeleton.
        name: String,
        /// The animation time this event was keyed.
        time: f32,
        /// The event's int value.
        int: i32,
        /// The event's float value.
        float: f32,
        /// The event's string value or an empty string.
        string: String,
        /// The event's audio path or an empty string.
        audio_path: String,
        /// The event's audio volume.
        volume: f32,
        /// The event's audio balance.
        balance: f32,
    },
}

#[derive(Debug)]
pub struct SpineSkeleton {
    pub shader: Option<ShaderRef>,
    pub uniforms: HashMap<Cow<'static, str>, GlowUniformValue>,
    textures: HashMap<String, SpriteTexture>,
    controller: RwLock<SkeletonController>,
    animation_events: Receiver<SpineEvent>,
}

impl SpineSkeleton {
    pub fn new(asset: &SpineAsset) -> Self {
        let (sender, receiver) = channel::<SpineEvent>();
        let mut controller = SkeletonController::new(
            asset.skeleton_data.clone(),
            Arc::new(AnimationStateData::new(asset.skeleton_data.clone())),
        );
        controller.animation_state.set_listener(move |_, event| {
            let _ = sender.send(match event {
                AnimationEvent::Start { .. } => SpineEvent::Start,
                AnimationEvent::Interrupt { .. } => SpineEvent::Interrupt,
                AnimationEvent::End { .. } => SpineEvent::End,
                AnimationEvent::Complete { .. } => SpineEvent::Complete,
                AnimationEvent::Dispose { .. } => SpineEvent::Dispose,
                AnimationEvent::Event {
                    name,
                    time,
                    int,
                    float,
                    string,
                    audio_path,
                    volume,
                    balance,
                    ..
                } => SpineEvent::Event {
                    name: name.to_owned(),
                    time,
                    int,
                    float,
                    string: string.to_owned(),
                    audio_path: audio_path.to_owned(),
                    volume,
                    balance,
                },
            });
        });
        let textures = asset
            .atlas
            .pages()
            .filter_map(|page| {
                let name = page.name().to_owned();
                let sampler = name
                    .strip_suffix(".png")
                    .unwrap_or(name.as_str())
                    .replace(['-', '.', '/', '\\'], "_")
                    .to_lowercase();
                let sampler = format!("u_{sampler}");
                let path = asset.textures.get(&name)?.path().to_owned();
                let texture = SpriteTexture::new(sampler.into(), TextureRef::name(path));
                Some((name, texture))
            })
            .collect::<HashMap<_, _>>();
        Self {
            shader: None,
            uniforms: Default::default(),
            textures,
            controller: RwLock::new(controller),
            animation_events: receiver,
        }
    }

    pub fn shader(mut self, value: ShaderRef) -> Self {
        self.shader = Some(value);
        self
    }

    pub fn uniform(mut self, key: Cow<'static, str>, value: GlowUniformValue) -> Self {
        self.uniforms.insert(key, value);
        self
    }

    pub fn read(&self) -> Option<RwLockReadGuard<SkeletonController>> {
        self.controller.try_read().ok()
    }

    pub fn write(&self) -> Option<RwLockWriteGuard<SkeletonController>> {
        self.controller.try_write().ok()
    }

    pub fn poll_event(&self) -> Option<SpineEvent> {
        self.animation_events.try_recv().ok()
    }

    pub fn play_animation(
        &self,
        name: &str,
        track_index: usize,
        timescale: f32,
        looping: bool,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(mut controller) = self.controller.try_write() {
            let mut track =
                controller
                    .animation_state
                    .set_animation_by_name(track_index, name, looping)?;
            track.set_timescale(timescale);
        }
        Ok(())
    }

    pub fn stop_animation(&self, track_index: usize) {
        if let Ok(mut controller) = self.controller.try_write() {
            controller.animation_state.clear_track(track_index);
        }
    }

    pub fn update(&self, delta_time: f32) {
        if let Ok(mut controller) = self.controller.try_write() {
            controller.update(delta_time, Physics::Update);
        }
    }
}

impl Drawable for SpineSkeleton {
    fn draw(&self, context: &mut DrawContext, graphics: &mut Graphics<Vertex>) {
        if let Ok(mut controller) = self.controller.try_write() {
            let renderables = controller.combined_renderables();
            for renderable in renderables {
                let batch = GraphicsBatch {
                    shader: context.shader(self.shader.as_ref()),
                    uniforms: self
                        .uniforms
                        .iter()
                        .map(|(k, v)| (k.clone(), v.to_owned()))
                        .chain(std::iter::once((
                            "u_projection_view".into(),
                            GlowUniformValue::M4(
                                graphics.main_camera.world_matrix().into_col_array(),
                            ),
                        )))
                        .chain(
                            self.textures
                                .iter()
                                .enumerate()
                                .map(|(index, (_, texture))| {
                                    (texture.sampler.clone(), GlowUniformValue::I1(index as _))
                                }),
                        )
                        .collect(),
                    textures: self
                        .textures
                        .iter()
                        .filter_map(|(_, texture)| {
                            Some((context.texture(Some(&texture.texture))?, texture.filtering))
                        })
                        .collect(),
                    blending: match renderable.blend_mode {
                        BlendMode::Normal => GlowBlending::Alpha,
                        BlendMode::Additive => GlowBlending::Additive,
                        BlendMode::Multiply => GlowBlending::Multiply,
                        BlendMode::Screen => GlowBlending::Additive,
                    },
                    scissor: None,
                };
                graphics.stream.batch_optimized(batch);
                graphics.stream.extend(
                    renderable
                        .vertices
                        .iter()
                        .copied()
                        .zip(renderable.uvs.iter().copied())
                        .zip(renderable.colors.iter().copied())
                        .map(|((position, uv), color)| Vertex {
                            position: [position[0], -position[1]],
                            uv: [uv[0], uv[1], 0.0],
                            color,
                        }),
                    renderable.indices.chunks(3).map(|chunk| Triangle {
                        a: chunk[0] as _,
                        b: chunk[1] as _,
                        c: chunk[2] as _,
                    }),
                );
            }
        }
    }
}
