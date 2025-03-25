use crate::assets::{
    atlas::AtlasAsset,
    spine::{SpineAnimationCurve, SpineDocument, SpineSlotBlendMode},
};
use anim8::{
    curve::{Curved, CurvedChange},
    phase::Phase,
    spline::{SplinePoint, SplinePointDirection},
};
use spitfire_draw::{
    context::DrawContext,
    sprite::{Sprite, SpriteTexture},
    utils::{transform_to_matrix, Drawable, TextureRef, Vertex},
};
use spitfire_glow::{graphics::Graphics, renderer::GlowBlending};
use std::{cmp::Ordering, collections::HashMap, error::Error, ops::Range};
use vek::{Mat4, Quaternion, Transform, Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub transform: Transform<f32, f32, f32>,
    pub bones: Vec<Bone>,
    pub slots: Vec<Slot>,
}

impl Skeleton {
    pub fn new(
        document: &SpineDocument,
        atlas: &AtlasAsset,
        skin: &str,
        slot_texture_sampler: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let skin = document
            .skins
            .iter()
            .find(|s| s.name == skin)
            .ok_or_else(|| format!("Skin not found: {}", skin))?;
        let mut bones = Vec::<Bone>::new();
        let mut slots = Vec::<Slot>::new();
        for bone in document.bones.iter() {
            let parent = bone
                .parent
                .as_ref()
                .and_then(|parent| bones.iter().position(|b| &b.name == parent));
            bones.push(Bone {
                name: bone.name.to_owned(),
                parent,
                transform: Transform {
                    position: Vec3::new(bone.x, -bone.y, 0.0),
                    orientation: Quaternion::rotation_z(-bone.rotation.to_radians()),
                    scale: Vec3::new(bone.scale_x, bone.scale_y, 1.0),
                },
            });
        }
        for slot in document.slots.iter() {
            let Some(attachment_name) = slot.attachment.as_deref() else {
                continue;
            };
            let slot_attachments = skin.attachments.get(&slot.name).ok_or_else(|| {
                format!(
                    "Slot attachments not found: {} in skin: {}",
                    slot.name, skin.name
                )
            })?;
            let attachment_region = slot_attachments.get(attachment_name).ok_or_else(|| {
                format!(
                    "Slot attachment: {} region not found: {} in skin: {}",
                    slot.name, attachment_name, skin.name
                )
            })?;
            let uvs = atlas
                .uvs(attachment_name)
                .ok_or_else(|| format!("UVs not found: {} in atlas", attachment_name))?;
            let transform = Transform {
                position: Vec3::new(attachment_region.x, -attachment_region.y, 0.0),
                orientation: Quaternion::rotation_z(-attachment_region.rotation.to_radians()),
                scale: Vec3::new(attachment_region.scale_x, attachment_region.scale_y, 1.0),
            };
            slots.push(Slot {
                name: slot.name.clone(),
                bone: bones
                    .iter()
                    .position(|b| b.name == slot.bone)
                    .ok_or_else(|| {
                        format!("Bone not found: {} of slot: {}", slot.bone, slot.name)
                    })?,
                sprite: Sprite::single(SpriteTexture::new(
                    slot_texture_sampler.to_owned().into(),
                    TextureRef::name(atlas.image.to_owned()),
                ))
                .size(Vec2::new(
                    attachment_region.width as f32,
                    attachment_region.height as f32,
                ))
                .region_page(uvs, 0.0)
                .pivot(0.5.into())
                .transform(transform)
                .blending(match slot.blend {
                    SpineSlotBlendMode::Normal => GlowBlending::Alpha,
                    SpineSlotBlendMode::Additive => GlowBlending::Additive,
                    SpineSlotBlendMode::Multiply => GlowBlending::Multiply,
                    SpineSlotBlendMode::Screen => GlowBlending::Additive,
                }),
                show: true,
            });
        }
        Ok(Self {
            transform: Transform::default(),
            bones,
            slots,
        })
    }

    fn calculate_transform(&self, bone: usize, transform: &mut Mat4<f32>) {
        let bone = &self.bones[bone];
        if let Some(parent) = bone.parent {
            self.calculate_transform(parent, transform);
        }
        *transform *= transform_to_matrix(bone.transform);
    }
}

impl Drawable for Skeleton {
    fn draw(&self, context: &mut DrawContext, graphics: &mut Graphics<Vertex>) {
        for slot in self.slots.iter() {
            if slot.show {
                let mut transform = transform_to_matrix(self.transform);
                self.calculate_transform(slot.bone, &mut transform);
                context.push_transform_relative(transform);
                slot.sprite.draw(context, graphics);
                context.pop_transform();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub transform: Transform<f32, f32, f32>,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub bone: usize,
    pub sprite: Sprite,
    pub show: bool,
}

#[derive(Debug)]
struct PhaseExtractMeta {
    time: f32,
    value: f32,
    #[allow(dead_code)]
    controls: Option<[f32; 4]>,
}

fn extract_phase(data: Vec<PhaseExtractMeta>, base_value: f32) -> Phase {
    if data.len() == 1 {
        return Phase::point(data[0].value + base_value).unwrap();
    }
    let mut points = data
        .iter()
        .map(|item| {
            SplinePoint::new(
                (item.time, item.value + base_value),
                SplinePointDirection::InOut((0.0, 0.0), (0.0, 0.0)),
            )
        })
        .collect::<Vec<_>>();
    for (index, pair) in data.windows(2).enumerate() {
        let (item_prev, item_next) = (&pair[0], &pair[1]);
        let (left, right) = points.split_at_mut(index + 1);
        let prev = left.last_mut().unwrap();
        let next = right.first_mut().unwrap();
        let prev_out = if let SplinePointDirection::InOut(_, value) = &mut prev.direction {
            value
        } else {
            continue;
        };
        let next_in = if let SplinePointDirection::InOut(value, _) = &mut next.direction {
            value
        } else {
            continue;
        };
        if let Some(controls) = item_prev.controls.as_ref() {
            prev_out.0 = -(controls[0] - item_prev.time);
            prev_out.1 = -(controls[1] - item_prev.value);
            next_in.0 = -(item_next.time - controls[2]);
            next_in.1 = -(item_next.value - controls[3]);
        } else {
            let offset = next.point.delta(&prev.point).scale(1.0 / 3.0);
            *prev_out = offset;
            *next_in = offset;
        }
    }
    Phase::new(points).unwrap()
}

macro_rules! extract_bone_phase {
    ( $component_list: expr, $value: ident, $offset: expr) => {
        if $component_list.is_empty() {
            vec![PhaseExtractMeta {
                time: 0.0,
                value: 0.0,
                controls: None,
            }]
        } else {
            $component_list
                .iter()
                .map(|item| PhaseExtractMeta {
                    time: item.time,
                    value: item.$value,
                    controls: match &item.curve {
                        SpineAnimationCurve::Linear => None,
                        SpineAnimationCurve::Stepped => None,
                        SpineAnimationCurve::BezierControlPoints(points) => Some([
                            points[$offset],
                            points[$offset + 1],
                            points[$offset + 2],
                            points[$offset + 3],
                        ]),
                    },
                })
                .collect::<Vec<_>>()
        }
    };
}

#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    pub current: Option<String>,
    pub time: f32,
    pub speed: f32,
    pub looped: bool,
    pub paused: bool,
    pub animation_set: AnimationSet,
}

impl AnimationPlayer {
    pub fn new(document: &SpineDocument) -> Self {
        Self {
            current: None,
            time: 0.0,
            speed: 1.0,
            looped: true,
            paused: false,
            animation_set: document
                .animations
                .iter()
                .map(|(name, animation)| {
                    let bones = animation
                        .bones
                        .iter()
                        .filter_map(|(name, timeline)| {
                            let bone = document.bones.iter().find(|bone| &bone.name == name)?;
                            let rotate = extract_bone_phase!(timeline.rotate, value, 0);
                            let rotate = extract_phase(rotate, bone.rotation);
                            let translate_x = extract_bone_phase!(timeline.translate, x, 0);
                            let translate_x = extract_phase(translate_x, bone.x);
                            let translate_y = extract_bone_phase!(timeline.translate, y, 4);
                            let translate_y = extract_phase(translate_y, bone.y);
                            let scale_x = extract_bone_phase!(timeline.scale, x, 0);
                            let scale_x = extract_phase(scale_x, bone.scale_x);
                            let scale_y = extract_bone_phase!(timeline.scale, y, 4);
                            let scale_y = extract_phase(scale_y, bone.scale_y);
                            Some(AnimationTimelineBone {
                                name: name.to_owned(),
                                rotate,
                                translate_x,
                                translate_y,
                                scale_x,
                                scale_y,
                            })
                        })
                        .collect::<Vec<_>>();
                    let duration_start = bones
                        .iter()
                        .map(|timeline| {
                            timeline
                                .rotate
                                .time_frame()
                                .start
                                .min(timeline.translate_x.time_frame().start)
                                .min(timeline.translate_y.time_frame().start)
                                .min(timeline.scale_x.time_frame().start)
                                .min(timeline.scale_y.time_frame().start)
                        })
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                        .unwrap_or_default();
                    let duration_end = bones
                        .iter()
                        .map(|timeline| {
                            timeline
                                .rotate
                                .time_frame()
                                .end
                                .max(timeline.translate_x.time_frame().end)
                                .max(timeline.translate_y.time_frame().end)
                                .max(timeline.scale_x.time_frame().end)
                                .max(timeline.scale_y.time_frame().end)
                        })
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                        .unwrap_or_default();
                    let animation = Animation {
                        bones,
                        events: animation
                            .events
                            .iter()
                            .map(|event| AnimationTimelineEvent {
                                time: event.time,
                                name: event.name.to_owned(),
                                int_value: event.int_value,
                                float_value: event.float_value,
                                string_value: event.string_value.clone(),
                            })
                            .collect(),
                        duration: duration_start..duration_end,
                    };
                    (name.to_owned(), animation)
                })
                .collect(),
        }
    }

    pub fn looped(mut self) -> Self {
        self.looped = true;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn paused(mut self) -> Self {
        self.paused = true;
        self
    }

    pub fn playing(mut self, animation: &str) -> Self {
        self.play(animation);
        self
    }

    pub fn play(&mut self, animation: &str) {
        self.current = Some(animation.to_owned());
        self.time = 0.0;
    }

    pub fn stop(&mut self) {
        self.current = None;
        self.time = 0.0;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.paused {
            return;
        }
        let Some(animation) = self
            .current
            .as_ref()
            .and_then(|name| self.animation_set.get(name))
        else {
            return;
        };
        self.time += delta_time * self.speed;
        if self.looped {
            if self.time < animation.duration.start {
                self.time = animation.duration.end;
            }
            if self.time > animation.duration.end {
                self.time = animation.duration.start;
            }
        } else {
            self.time = self
                .time
                .clamp(animation.duration.start, animation.duration.end);
        }
    }

    pub fn apply_to_skeleton(&self, skeleton: &mut Skeleton) {
        let Some(animation) = self
            .current
            .as_ref()
            .and_then(|name| self.animation_set.get(name))
        else {
            return;
        };
        for timeline in animation.bones.iter() {
            let Some(bone) = skeleton
                .bones
                .iter()
                .position(|bone| bone.name == timeline.name)
            else {
                continue;
            };
            let rotate = timeline.rotate.sample(self.time);
            let translate_x = timeline.translate_x.sample(self.time);
            let translate_y = timeline.translate_y.sample(self.time);
            let scale_x = timeline.scale_x.sample(self.time);
            let scale_y = timeline.scale_y.sample(self.time);
            let transform = Transform {
                position: Vec3::new(translate_x, -translate_y, 0.0),
                orientation: Quaternion::rotation_z(-rotate.to_radians()),
                scale: Vec3::new(scale_x, scale_y, 1.0),
            };
            skeleton.bones[bone].transform = transform;
        }
    }
}

pub type AnimationSet = HashMap<String, Animation>;

#[derive(Debug, Clone)]
pub struct Animation {
    pub bones: Vec<AnimationTimelineBone>,
    pub events: Vec<AnimationTimelineEvent>,
    pub duration: Range<f32>,
}

#[derive(Debug, Clone)]
pub struct AnimationTimelineBone {
    pub name: String,
    pub rotate: Phase,
    pub translate_x: Phase,
    pub translate_y: Phase,
    pub scale_x: Phase,
    pub scale_y: Phase,
}

#[derive(Debug, Clone)]
pub struct AnimationTimelineEvent {
    pub time: f32,
    pub name: String,
    pub int_value: Option<isize>,
    pub float_value: Option<f32>,
    pub string_value: Option<String>,
}
