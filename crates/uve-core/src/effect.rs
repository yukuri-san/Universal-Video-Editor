use crate::keyframe::KeyframedValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    Number(KeyframedValue<f32>),
    Boolean(KeyframedValue<bool>),
    Text(String),
    Color([u8; 4]),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    pub id: String,
    pub name: String,
    pub parameters: Vec<EffectParameter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectParameter {
    pub id: String,
    pub name: String,
    pub default: ParameterValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectInstance {
    pub effect_id: String,
    pub enabled: bool,
    pub parameters: BTreeMap<String, ParameterValue>,
}

impl EffectInstance {
    pub fn new(effect: &Effect) -> Self {
        let parameters = effect
            .parameters
            .iter()
            .map(|parameter| (parameter.id.clone(), parameter.default.clone()))
            .collect();
        Self {
            effect_id: effect.id.clone(),
            enabled: true,
            parameters,
        }
    }
}
