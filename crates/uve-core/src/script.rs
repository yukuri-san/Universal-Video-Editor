use crate::keyframe::{Interpolation, Keyframe, KeyframedValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptValue {
    Number(f32),
    Text(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptCommand {
    pub name: String,
    pub arguments: Vec<ScriptValue>,
}

#[derive(Debug, Error, PartialEq)]
pub enum ScriptError {
    #[error("script line {line}: missing command name")]
    MissingCommand { line: usize },
    #[error("script line {line}: invalid number `{value}`")]
    InvalidNumber { line: usize, value: String },
}

pub fn parse_script(source: &str) -> Result<Vec<ScriptCommand>, ScriptError> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let line = raw.split("--").next()?.trim();
            (!line.is_empty()).then_some((index + 1, line))
        })
        .map(|(line_number, line)| {
            let mut tokens = line.split_whitespace();
            let name = tokens
                .next()
                .ok_or(ScriptError::MissingCommand { line: line_number })?
                .to_string();
            let arguments = tokens
                .map(|token| {
                    if let Ok(number) = token.parse::<f32>() {
                        return Ok(ScriptValue::Number(number));
                    }
                    match token {
                        "true" => Ok(ScriptValue::Boolean(true)),
                        "false" => Ok(ScriptValue::Boolean(false)),
                        value if value.starts_with('"') && value.ends_with('"') => {
                            Ok(ScriptValue::Text(value[1..value.len() - 1].to_string()))
                        }
                        value => Err(ScriptError::InvalidNumber {
                            line: line_number,
                            value: value.to_string(),
                        }),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ScriptCommand { name, arguments })
        })
        .collect()
}

pub fn linear_motion(
    start_frame: u64,
    end_frame: u64,
    start: f32,
    end: f32,
) -> KeyframedValue<f32> {
    KeyframedValue {
        default: start,
        keyframes: vec![
            Keyframe {
                frame: start_frame,
                value: start,
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                frame: end_frame,
                value: end,
                interpolation: Interpolation::Linear,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands_and_comments() {
        let commands = parse_script("移動 0 100 -- comment\n透明度 0.5").unwrap();
        assert_eq!(commands[0].name, "移動");
        assert_eq!(commands[0].arguments[1], ScriptValue::Number(100.0));
        assert_eq!(commands.len(), 2);
    }
}
