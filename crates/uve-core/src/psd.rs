use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PsdDocument {
    pub width: u32,
    pub height: u32,
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
}
