use anput::world::World;
use keket::{database::handle::AssetHandle, protocol::AssetProtocol};
use serde::{de::Visitor, Deserialize, Deserializer};
use std::{collections::HashMap, error::Error};

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineDocument {
    pub skeleton: SpineSkeleton,
    #[serde(default)]
    pub bones: Vec<SpineBone>,
    #[serde(default)]
    pub slots: Vec<SpineSlot>,
    #[serde(default)]
    pub skins: Vec<SpineSkin>,
    #[serde(default)]
    pub events: HashMap<String, SpineEvent>,
    #[serde(default)]
    pub animations: HashMap<String, SpineAnimation>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineSkeleton {
    pub hash: String,
    pub spine: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpineTransformMode {
    #[default]
    Normal,
    OnlyTranslation,
    NoRotationOrReflection,
    NoScale,
    NoScaleOrReflection,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineBone {
    pub name: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub length: f32,
    #[serde(default)]
    pub transform: SpineTransformMode,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default = "default_scale")]
    pub scale_x: f32,
    #[serde(default = "default_scale")]
    pub scale_y: f32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpineSlotBlendMode {
    #[default]
    Normal,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineSlot {
    pub name: String,
    pub bone: String,
    #[serde(default)]
    pub attachment: Option<String>,
    #[serde(default)]
    pub blend: SpineSlotBlendMode,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineSkinAttachmentRegion {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_scale")]
    pub scale_x: f32,
    #[serde(default = "default_scale")]
    pub scale_y: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub width: usize,
    #[serde(default)]
    pub height: usize,
}

pub type SpineSkinAttachmentSlot = HashMap<String, SpineSkinAttachmentRegion>;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineSkin {
    pub name: String,
    #[serde(default)]
    pub attachments: HashMap<String, SpineSkinAttachmentSlot>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineEvent {
    #[serde(rename = "int")]
    #[serde(default)]
    pub int_value: isize,
    #[serde(rename = "float")]
    #[serde(default)]
    pub float_value: f32,
    #[serde(rename = "string")]
    #[serde(default)]
    pub string_value: Option<String>,
}

#[derive(Debug, Default, Clone)]
// #[serde(rename_all = "camelCase", untagged)]
pub enum SpineAnimationCurve {
    #[default]
    Linear,
    Stepped,
    BezierControlPoints(Vec<f32>),
}

impl<'de> Deserialize<'de> for SpineAnimationCurve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CurveVisitor;

        impl<'de> Visitor<'de> for CurveVisitor {
            type Value = SpineAnimationCurve;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("stepped, an array of floats, or nothing (Linear)")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == "stepped" {
                    Ok(SpineAnimationCurve::Stepped)
                } else {
                    Err(serde::de::Error::unknown_variant(value, &["stepped"]))
                }
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let vec: Vec<f32> =
                    Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;
                Ok(SpineAnimationCurve::BezierControlPoints(vec))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SpineAnimationCurve::Linear)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SpineAnimationCurve::Linear)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }
        }

        deserializer.deserialize_any(CurveVisitor)
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineAnimationBoneRotate {
    #[serde(default)]
    pub time: f32,
    #[serde(default)]
    #[serde(alias = "angle")]
    pub value: f32,
    #[serde(default)]
    pub curve: SpineAnimationCurve,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineAnimationBoneTranslateOrScale {
    #[serde(default)]
    pub time: f32,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub curve: SpineAnimationCurve,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineAnimationBone {
    #[serde(default)]
    pub rotate: Vec<SpineAnimationBoneRotate>,
    #[serde(default)]
    pub translate: Vec<SpineAnimationBoneTranslateOrScale>,
    #[serde(default)]
    pub scale: Vec<SpineAnimationBoneTranslateOrScale>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineAnimationEvent {
    #[serde(default)]
    pub time: f32,
    pub name: String,
    #[serde(rename = "int")]
    #[serde(default)]
    pub int_value: Option<isize>,
    #[serde(rename = "float")]
    #[serde(default)]
    pub float_value: Option<f32>,
    #[serde(rename = "string")]
    #[serde(default)]
    pub string_value: Option<String>,
    #[serde(default)]
    pub volume: Option<f32>,
    #[serde(default)]
    pub balance: Option<f32>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineAnimation {
    #[serde(default)]
    pub bones: HashMap<String, SpineAnimationBone>,
    #[serde(default)]
    pub events: Vec<SpineAnimationEvent>,
}

pub struct SpineAssetProtocol;

impl AssetProtocol for SpineAssetProtocol {
    fn name(&self) -> &str {
        "spine"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let document = serde_json::from_slice::<SpineDocument>(&bytes)?;

        storage.insert(handle.entity(), (document,))?;

        Ok(())
    }
}
