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
    #[error("script line {line}: unterminated quoted string")]
    UnterminatedString { line: usize },
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
            let tokens = tokenize(line, line_number)?;
            let mut tokens = tokens.into_iter();
            let name = tokens
                .next()
                .ok_or(ScriptError::MissingCommand { line: line_number })?
                .to_owned();
            let arguments = tokens
                .map(|token| {
                    if let Ok(number) = token.parse::<f32>() {
                        return Ok(ScriptValue::Number(number));
                    }
                    match token.as_str() {
                        "true" => Ok(ScriptValue::Boolean(true)),
                        "false" => Ok(ScriptValue::Boolean(false)),
                        _ => Ok(ScriptValue::Text(token)),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ScriptCommand { name, arguments })
        })
        .collect()
}

fn tokenize(line: &str, line_number: usize) -> Result<Vec<String>, ScriptError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quoted = false;

    for character in line.chars() {
        match character {
            '"' => quoted = !quoted,
            character if character.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            character => current.push(character),
        }
    }

    if quoted {
        return Err(ScriptError::UnterminatedString { line: line_number });
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Ok(tokens)
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
        let commands = parse_script("テキスト \"こんにちは 世界\" -- comment\n透明度 0.5").unwrap();
        assert_eq!(commands[0].name, "テキスト");
        assert_eq!(
            commands[0].arguments[0],
            ScriptValue::Text("こんにちは 世界".into())
        );
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn rejects_unterminated_strings() {
        assert!(matches!(
            parse_script("テキスト \"未完了"),
            Err(ScriptError::UnterminatedString { line: 1 })
        ));
    }
}
