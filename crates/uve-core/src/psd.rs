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
        let layers = parse_layers(bytes)?;
        Ok(Self {
            width,
            height,
            channels,
            depth,
            color_mode,
            layers,
        })
    }
}

fn parse_layers(bytes: &[u8]) -> Result<Vec<PsdLayer>, PsdError> {
    if bytes.len() == 26 {
        return Ok(Vec::new());
    }
    let mut cursor = 26;
    let color_mode_length = read_u32(bytes, cursor)? as usize;
    cursor = cursor
        .checked_add(4 + color_mode_length)
        .ok_or(PsdError::TooShort)?;
    let image_resources_length = read_u32(bytes, cursor)? as usize;
    cursor = cursor
        .checked_add(4 + image_resources_length)
        .ok_or(PsdError::TooShort)?;
    let layer_mask_length = read_u32(bytes, cursor)? as usize;
    cursor += 4;
    let section_end = cursor
        .checked_add(layer_mask_length)
        .ok_or(PsdError::TooShort)?;
    if section_end > bytes.len() || layer_mask_length < 4 {
        return Ok(Vec::new());
    }

    let layer_info_length = read_u32(bytes, cursor)? as usize;
    cursor += 4;
    if layer_info_length == 0 {
        return Ok(Vec::new());
    }
    let layer_info_end = cursor
        .checked_add(layer_info_length)
        .ok_or(PsdError::TooShort)?;
    if layer_info_end > section_end {
        return Err(PsdError::InvalidLayerSection);
    }

    let raw_count = read_i16(bytes, cursor)?;
    cursor += 2;
    let count = raw_count.unsigned_abs() as usize;
    let mut layers = Vec::with_capacity(count);
    for index in 0..count {
        let top = read_i32(bytes, cursor)?;
        let left = read_i32(bytes, cursor + 4)?;
        let bottom = read_i32(bytes, cursor + 8)?;
        let right = read_i32(bytes, cursor + 12)?;
        cursor += 16;

        let channel_count = read_u16(bytes, cursor)? as usize;
        cursor += 2 + channel_count * 6;
        cursor = cursor.checked_add(8).ok_or(PsdError::TooShort)?;
        let opacity = *bytes.get(cursor).ok_or(PsdError::TooShort)? as f32 / 255.0;
        let flags = *bytes.get(cursor + 2).ok_or(PsdError::TooShort)?;
        cursor += 4;

        let extra_length = read_u32(bytes, cursor)? as usize;
        cursor += 4;
        let extra_end = cursor.checked_add(extra_length).ok_or(PsdError::TooShort)?;
        if extra_end > layer_info_end {
            return Err(PsdError::InvalidLayerSection);
        }
        let mask_length = read_u32(bytes, cursor)? as usize;
        cursor += 4 + mask_length;
        let blending_ranges_length = read_u32(bytes, cursor)? as usize;
        cursor += 4 + blending_ranges_length;
        let name_length = *bytes.get(cursor).ok_or(PsdError::TooShort)? as usize;
        cursor += 1;
        let name_end = cursor.checked_add(name_length).ok_or(PsdError::TooShort)?;
        if name_end > extra_end {
            return Err(PsdError::InvalidLayerSection);
        }
        let name = String::from_utf8_lossy(&bytes[cursor..name_end]).into_owned();
        cursor = extra_end;

        layers.push(PsdLayer {
            id: format!("layer-{index}"),
            name,
            visible: flags & 0x02 == 0,
            opacity,
            bounds: [left, top, right, bottom],
            children: Vec::new(),
        });
    }
    Ok(layers)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, PsdError> {
    let slice = bytes.get(offset..offset + 2).ok_or(PsdError::TooShort)?;
    Ok(u16::from_be_bytes([slice[0], slice[1]]))
}

fn read_i16(bytes: &[u8], offset: usize) -> Result<i16, PsdError> {
    Ok(read_u16(bytes, offset)? as i16)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, PsdError> {
    let slice = bytes.get(offset..offset + 4).ok_or(PsdError::TooShort)?;
    Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, PsdError> {
    Ok(read_u32(bytes, offset)? as i32)
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
    #[error("invalid PSD layer section")]
    InvalidLayerSection,
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

    #[test]
    fn reads_layer_metadata() {
        let mut bytes = vec![0; 26];
        bytes[0..4].copy_from_slice(b"8BPS");
        bytes[5] = 1;
        bytes[14..18].copy_from_slice(&100u32.to_be_bytes());
        bytes[18..22].copy_from_slice(&200u32.to_be_bytes());
        bytes[23] = 8;
        bytes.extend_from_slice(&[0, 0, 0, 0]); // color mode data
        bytes.extend_from_slice(&[0, 0, 0, 0]); // image resources

        let mut layer_info = Vec::new();
        layer_info.extend_from_slice(&1i16.to_be_bytes());
        layer_info.extend_from_slice(&0i32.to_be_bytes()); // top
        layer_info.extend_from_slice(&0i32.to_be_bytes()); // left
        layer_info.extend_from_slice(&100i32.to_be_bytes()); // bottom
        layer_info.extend_from_slice(&200i32.to_be_bytes()); // right
        layer_info.extend_from_slice(&0u16.to_be_bytes()); // channels
        layer_info.extend_from_slice(b"8BIM");
        layer_info.extend_from_slice(b"norm");
        layer_info.push(200); // opacity
        layer_info.push(0); // clipping
        layer_info.push(0); // visible
        layer_info.push(0); // filler
        let name = b"Layer 1";
        let extra_length = 4 + 4 + 1 + name.len() + 1;
        layer_info.extend_from_slice(&(extra_length as u32).to_be_bytes());
        layer_info.extend_from_slice(&[0, 0, 0, 0]); // mask
        layer_info.extend_from_slice(&[0, 0, 0, 0]); // blending ranges
        layer_info.push(name.len() as u8);
        layer_info.extend_from_slice(name);
        layer_info.push(0); // pad to 4-byte boundary

        bytes.extend_from_slice(&((4 + layer_info.len()) as u32).to_be_bytes());
        bytes.extend_from_slice(&(layer_info.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&layer_info);

        let document = PsdDocument::from_bytes(&bytes).unwrap();
        assert_eq!(document.layers[0].name, "Layer 1");
        assert!(document.layers[0].visible);
        assert_eq!(document.layers[0].bounds, [0, 0, 200, 100]);
    }
}
