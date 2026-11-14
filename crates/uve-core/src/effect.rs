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

pub fn builtin_effects() -> Vec<Effect> {
    vec![
        Effect {
            id: "transform".into(),
            name: "移動・変形".into(),
            parameters: vec![
                number_parameter("x", "X", 0.0),
                number_parameter("y", "Y", 0.0),
                number_parameter("scale_x", "幅", 1.0),
                number_parameter("scale_y", "高さ", 1.0),
                number_parameter("rotation", "回転", 0.0),
            ],
        },
        Effect {
            id: "opacity".into(),
            name: "透明度".into(),
            parameters: vec![number_parameter("amount", "透明度", 1.0)],
        },
        Effect {
            id: "blur".into(),
            name: "ぼかし".into(),
            parameters: vec![number_parameter("radius", "強さ", 0.0)],
        },
        Effect {
            id: "color-key".into(),
            name: "クロマキー".into(),
            parameters: vec![
                EffectParameter {
                    id: "color".into(),
                    name: "キー色".into(),
                    default: ParameterValue::Color([0, 255, 0, 255]),
                },
                number_parameter("threshold", "許容範囲", 0.1),
            ],
        },
    ]
}

fn number_parameter(id: &str, name: &str, value: f32) -> EffectParameter {
    EffectParameter {
        id: id.into(),
        name: name.into(),
        default: ParameterValue::Number(KeyframedValue::constant(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_instances_from_builtin_effects() {
        let blur = builtin_effects()
            .into_iter()
            .find(|effect| effect.id == "blur")
            .unwrap();
        let instance = EffectInstance::new(&blur);
        assert!(instance.enabled);
        assert!(instance.parameters.contains_key("radius"));
    }
}
