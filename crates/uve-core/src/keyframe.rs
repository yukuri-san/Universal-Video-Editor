use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interpolation {
    Step,
    Linear,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub frame: u64,
    pub value: T,
    pub interpolation: Interpolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframedValue<T> {
    pub default: T,
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone> KeyframedValue<T> {
    pub fn constant(value: T) -> Self {
        Self {
            default: value,
            keyframes: Vec::new(),
        }
    }

    pub fn value_at(&self, frame: u64) -> T
    where
        T: Lerp,
    {
        let Some(first) = self.keyframes.first() else {
            return self.default.clone();
        };

        if frame <= first.frame {
            return first.value.clone();
        }

        for pair in self.keyframes.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            if frame <= right.frame {
                if left.interpolation == Interpolation::Step || left.frame == right.frame {
                    return left.value.clone();
                }
                let progress = (frame - left.frame) as f32 / (right.frame - left.frame) as f32;
                return left.value.lerp(&right.value, progress);
            }
        }

        self.keyframes
            .last()
            .map(|keyframe| keyframe.value.clone())
            .unwrap_or_else(|| self.default.clone())
    }
}

pub trait Lerp {
    fn lerp(&self, other: &Self, progress: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, progress: f32) -> Self {
        self + (other - self) * progress.clamp(0.0, 1.0)
    }
}

impl Lerp for bool {
    fn lerp(&self, other: &Self, progress: f32) -> Self {
        if progress < 1.0 {
            *self
        } else {
            *other
        }
    }
}

impl Lerp for [f32; 2] {
    fn lerp(&self, other: &Self, progress: f32) -> Self {
        [
            self[0].lerp(&other[0], progress),
            self[1].lerp(&other[1], progress),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_between_keyframes() {
        let value = KeyframedValue {
            default: 0.0,
            keyframes: vec![
                Keyframe {
                    frame: 10,
                    value: 10.0,
                    interpolation: Interpolation::Linear,
                },
                Keyframe {
                    frame: 20,
                    value: 30.0,
                    interpolation: Interpolation::Linear,
                },
            ],
        };

        assert_eq!(value.value_at(15), 20.0);
    }

    #[test]
    fn step_interpolation_keeps_the_previous_value() {
        let value = KeyframedValue {
            default: 0.0,
            keyframes: vec![
                Keyframe {
                    frame: 0,
                    value: 10.0,
                    interpolation: Interpolation::Step,
                },
                Keyframe {
                    frame: 10,
                    value: 30.0,
                    interpolation: Interpolation::Linear,
                },
            ],
        };

        assert_eq!(value.value_at(5), 10.0);
    }
}
