use crate::effect::EffectInstance;
use crate::keyframe::KeyframedValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub name: String,
    pub frame_rate: f32,
    pub width: u32,
    pub height: u32,
    pub timeline: Timeline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Timeline {
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub muted: bool,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub start_frame: u64,
    pub duration_frames: u64,
    pub source: ClipSource,
    pub transform: Transform,
    pub effects: Vec<EffectInstance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClipSource {
    Video { path: String },
    Image { path: String },
    Audio { path: String },
    Text { content: String },
    PsdLayer { document: String, layer_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub x: KeyframedValue<f32>,
    pub y: KeyframedValue<f32>,
    pub scale_x: KeyframedValue<f32>,
    pub scale_y: KeyframedValue<f32>,
    pub rotation: KeyframedValue<f32>,
    pub opacity: KeyframedValue<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedTransform {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderItem<'a> {
    pub clip: &'a Clip,
    pub local_frame: u64,
    pub transform: EvaluatedTransform,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: KeyframedValue::constant(0.0),
            y: KeyframedValue::constant(0.0),
            scale_x: KeyframedValue::constant(1.0),
            scale_y: KeyframedValue::constant(1.0),
            rotation: KeyframedValue::constant(0.0),
            opacity: KeyframedValue::constant(1.0),
        }
    }
}

impl Transform {
    pub fn at(&self, frame: u64) -> EvaluatedTransform {
        EvaluatedTransform {
            x: self.x.value_at(frame),
            y: self.y.value_at(frame),
            scale_x: self.scale_x.value_at(frame),
            scale_y: self.scale_y.value_at(frame),
            rotation: self.rotation.value_at(frame),
            opacity: self.opacity.value_at(frame).clamp(0.0, 1.0),
        }
    }
}

impl Timeline {
    pub fn add_track(&mut self, track: Track) -> Result<(), TimelineError> {
        if self.tracks.iter().any(|existing| existing.id == track.id) {
            return Err(TimelineError::DuplicateId(track.id));
        }
        self.tracks.push(track);
        Ok(())
    }

    pub fn add_clip(&mut self, track_id: &str, clip: Clip) -> Result<(), TimelineError> {
        if self
            .tracks
            .iter()
            .flat_map(|track| track.clips.iter())
            .any(|existing| existing.id == clip.id)
        {
            return Err(TimelineError::DuplicateId(clip.id));
        }
        if clip.duration_frames == 0 {
            return Err(TimelineError::ZeroLengthClip(clip.id));
        }
        let track = self
            .tracks
            .iter_mut()
            .find(|track| track.id == track_id)
            .ok_or_else(|| TimelineError::TrackNotFound(track_id.into()))?;
        track.clips.push(clip);
        Ok(())
    }

    pub fn move_clip(&mut self, clip_id: &str, start_frame: u64) -> Result<(), TimelineError> {
        let clip = self
            .tracks
            .iter_mut()
            .flat_map(|track| track.clips.iter_mut())
            .find(|clip| clip.id == clip_id)
            .ok_or_else(|| TimelineError::ClipNotFound(clip_id.into()))?;
        clip.start_frame = start_frame;
        Ok(())
    }

    pub fn remove_clip(&mut self, clip_id: &str) -> Result<Clip, TimelineError> {
        for track in &mut self.tracks {
            if let Some(index) = track.clips.iter().position(|clip| clip.id == clip_id) {
                return Ok(track.clips.remove(index));
            }
        }
        Err(TimelineError::ClipNotFound(clip_id.into()))
    }

    pub fn render_items_at(&self, frame: u64) -> Vec<RenderItem<'_>> {
        self.tracks
            .iter()
            .filter(|track| !track.muted)
            .flat_map(|track| track.clips.iter())
            .filter(|clip| clip.contains_frame(frame))
            .map(|clip| RenderItem {
                clip,
                local_frame: frame - clip.start_frame,
                transform: clip.transform.at(frame - clip.start_frame),
            })
            .collect()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum TimelineError {
    #[error("duplicate timeline id: {0}")]
    DuplicateId(String),
    #[error("track not found: {0}")]
    TrackNotFound(String),
    #[error("clip not found: {0}")]
    ClipNotFound(String),
    #[error("clip `{0}` has zero duration")]
    ZeroLengthClip(String),
}

impl Clip {
    pub fn end_frame(&self) -> u64 {
        self.start_frame.saturating_add(self.duration_frames)
    }

    pub fn contains_frame(&self, frame: u64) -> bool {
        frame >= self.start_frame && frame < self.end_frame()
    }
}

impl Project {
    pub fn new(name: impl Into<String>, width: u32, height: u32, frame_rate: f32) -> Self {
        Self {
            version: 1,
            name: name.into(),
            frame_rate,
            width,
            height,
            timeline: Timeline::default(),
        }
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.width == 0 || self.height == 0 {
            return Err(ProjectError::InvalidDimensions);
        }
        if !self.frame_rate.is_finite() || self.frame_rate <= 0.0 {
            return Err(ProjectError::InvalidFrameRate);
        }
        for track in &self.timeline.tracks {
            for clip in &track.clips {
                if clip.duration_frames == 0 {
                    return Err(ProjectError::ZeroLengthClip {
                        clip_id: clip.id.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, ProjectError> {
        let project: Self = serde_json::from_str(json)?;
        project.validate()?;
        Ok(project)
    }
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project dimensions must be greater than zero")]
    InvalidDimensions,
    #[error("frame rate must be finite and greater than zero")]
    InvalidFrameRate,
    #[error("clip `{clip_id}` has zero duration")]
    ZeroLengthClip { clip_id: String },
    #[error("invalid project JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyframe::{Interpolation, Keyframe};

    #[test]
    fn round_trips_a_project_as_json() {
        let mut project = Project::new("test", 1920, 1080, 30.0);
        project.timeline.tracks.push(Track {
            id: "track-1".into(),
            name: "Main".into(),
            muted: false,
            clips: vec![Clip {
                id: "clip-1".into(),
                start_frame: 0,
                duration_frames: 60,
                source: ClipSource::Text {
                    content: "Hello".into(),
                },
                transform: Transform {
                    x: KeyframedValue {
                        default: 0.0,
                        keyframes: vec![Keyframe {
                            frame: 30,
                            value: 100.0,
                            interpolation: Interpolation::Linear,
                        }],
                    },
                    ..Transform::default()
                },
                effects: Vec::new(),
            }],
        });

        let restored = Project::from_json(&project.to_json().unwrap()).unwrap();
        assert_eq!(restored, project);
        assert!(restored.timeline.tracks[0].clips[0].contains_frame(59));
        assert!(!restored.timeline.tracks[0].clips[0].contains_frame(60));
    }

    #[test]
    fn rejects_invalid_projects() {
        let project = Project::new("invalid", 0, 1080, 30.0);
        assert!(matches!(
            project.validate(),
            Err(ProjectError::InvalidDimensions)
        ));
    }

    #[test]
    fn evaluates_active_clips_at_a_frame() {
        let mut project = Project::new("preview", 1280, 720, 30.0);
        project.timeline.tracks.push(Track {
            id: "track-1".into(),
            name: "Main".into(),
            muted: false,
            clips: vec![Clip {
                id: "clip-1".into(),
                start_frame: 10,
                duration_frames: 20,
                source: ClipSource::Text {
                    content: "Preview".into(),
                },
                transform: Transform::default(),
                effects: Vec::new(),
            }],
        });

        let items = project.timeline.render_items_at(15);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].local_frame, 5);
        assert_eq!(items[0].transform.opacity, 1.0);
        assert!(project.timeline.render_items_at(30).is_empty());
    }

    #[test]
    fn edits_clips_through_timeline_api() {
        let mut timeline = Timeline::default();
        timeline
            .add_track(Track {
                id: "track-1".into(),
                name: "Main".into(),
                muted: false,
                clips: Vec::new(),
            })
            .unwrap();
        timeline
            .add_clip(
                "track-1",
                Clip {
                    id: "clip-1".into(),
                    start_frame: 0,
                    duration_frames: 10,
                    source: ClipSource::Image {
                        path: "image.png".into(),
                    },
                    transform: Transform::default(),
                    effects: Vec::new(),
                },
            )
            .unwrap();
        timeline.move_clip("clip-1", 20).unwrap();
        assert_eq!(timeline.tracks[0].clips[0].start_frame, 20);
        assert_eq!(timeline.remove_clip("clip-1").unwrap().id, "clip-1");
        assert!(timeline.tracks[0].clips.is_empty());
    }
}
