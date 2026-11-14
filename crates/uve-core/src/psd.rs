use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PsdDocument {
    pub width: u32,
    pub height: u32,
    pub channels: u16,
    pub depth: u16,
    pub color_mode: u16,
    pub layers: Vec<PsdLayer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PsdLayer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub bounds: [i32; 4],
    pub children: Vec<PsdLayer>,
}

impl PsdDocument {
    pub fn visible_layers(&self) -> impl Iterator<Item = &PsdLayer> {
        self.layers.iter().filter(|layer| layer.visible)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PsdError> {
        if bytes.len() < 26 {
            return Err(PsdError::TooShort);
        }
        if &bytes[0..4] != b"8BPS" {
            return Err(PsdError::InvalidSignature);
        }
        let version = read_u16(bytes, 4)?;
        if version != 1 {
            return Err(PsdError::UnsupportedVersion(version));
        }
        if bytes[6..12].iter().any(|byte| *byte != 0) {
            return Err(PsdError::InvalidReservedBytes);
        }

        let channels = read_u16(bytes, 12)?;
        let height = read_u32(bytes, 14)?;
        let width = read_u32(bytes, 18)?;
        let depth = read_u16(bytes, 22)?;
        let color_mode = read_u16(bytes, 24)?;

        if width == 0 || height == 0 {
            return Err(PsdError::InvalidDimensions);
        }
        Ok(Self {
            width,
            height,
            channels,
            depth,
            color_mode,
            layers: Vec::new(),
        })
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, PsdError> {
    let slice = bytes.get(offset..offset + 2).ok_or(PsdError::TooShort)?;
    Ok(u16::from_be_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, PsdError> {
    let slice = bytes.get(offset..offset + 4).ok_or(PsdError::TooShort)?;
    Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[derive(Debug, Error, PartialEq)]
pub enum PsdError {
    #[error("PSD data is too short")]
    TooShort,
    #[error("invalid PSD signature")]
    InvalidSignature,
    #[error("unsupported PSD version: {0}")]
    UnsupportedVersion(u16),
    #[error("PSD reserved bytes must be zero")]
    InvalidReservedBytes,
    #[error("PSD dimensions must be greater than zero")]
    InvalidDimensions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_psd_header() {
        let mut bytes = vec![0; 26];
        bytes[0..4].copy_from_slice(b"8BPS");
        bytes[5] = 1;
        bytes[13] = 4;
        bytes[14..18].copy_from_slice(&1080u32.to_be_bytes());
        bytes[18..22].copy_from_slice(&1920u32.to_be_bytes());
        bytes[23] = 8;
        let document = PsdDocument::from_bytes(&bytes).unwrap();
        assert_eq!(document.width, 1920);
        assert_eq!(document.height, 1080);
        assert_eq!(document.channels, 4);
        assert_eq!(document.depth, 8);
    }

    #[test]
    fn rejects_non_psd_data() {
        assert_eq!(
            PsdDocument::from_bytes(b"not a psd"),
            Err(PsdError::TooShort)
        );
    }
}
