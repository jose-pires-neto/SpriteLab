use assetforge_command::{CommandError, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
pub type Id = u64;
pub type Color = [u8; 4];
pub const MAX_PIXELS: usize = 524_288;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub id: Id,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub id: Id,
    pub duration_ms: u32,
    pub cels: BTreeMap<Id, Arc<Vec<Color>>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tag {
    pub name: String,
    pub from_frame: Id,
    pub to_frame: Id,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format_version: u32,
    pub kind: String,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub frames: Vec<Frame>,
    pub palette: Vec<Color>,
    pub tags: Vec<Tag>,
}
pub fn invalid(message: impl Into<String>) -> CommandError {
    CommandError::new("invalid_arguments", message)
}
impl Document {
    /// Thumbnail content is independent of layer visibility and opacity.
    pub fn render_layer(&self, layer: Id, frame: Id) -> Result<Vec<u8>> {
        if !self.layers.iter().any(|l| l.id == layer) {
            return Err(CommandError::new("not_found", "Layer not found"));
        }
        let i = self.frame_index(frame)?;
        Ok(match self.frames[i].cels.get(&layer) {
            Some(cel) => cel.iter().flatten().copied().collect(),
            None => vec![0; self.area() * 4],
        })
    }
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let d = Self {
            format_version: 1,
            kind: "sprite".into(),
            width,
            height,
            layers: vec![Layer {
                id: 1,
                name: "Layer 1".into(),
                visible: true,
                locked: false,
                opacity: 255,
            }],
            frames: vec![Frame {
                id: 2,
                duration_ms: 100,
                cels: BTreeMap::new(),
            }],
            palette: vec![],
            tags: vec![],
        };
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1 || self.kind != "sprite" {
            return Err(invalid("Unsupported document format/version"));
        }
        if self.width == 0 || self.height == 0 || self.width > 512 || self.height > 512 {
            return Err(invalid("Dimensions must be 1..512"));
        }
        if self.layers.is_empty()
            || self.layers.len() > 32
            || self.frames.is_empty()
            || self.frames.len() > 64
        {
            return Err(invalid("Require 1..32 layers and 1..64 frames"));
        }
        if self.palette.len() > 256 || self.tags.len() > 64 {
            return Err(invalid("Palette/tag limit exceeded"));
        }
        let mut ids = BTreeSet::new();
        for layer in &self.layers {
            if layer.id == 0
                || layer.id > u32::MAX as u64
                || !ids.insert(layer.id)
                || layer.name.len() > 128
            {
                return Err(invalid("Invalid/duplicate layer ID or name"));
            }
        }
        let layer_ids: BTreeSet<_> = self.layers.iter().map(|l| l.id).collect();
        let mut pixels = 0;
        for frame in &self.frames {
            if frame.id == 0
                || frame.id > u32::MAX as u64
                || !ids.insert(frame.id)
                || !(1..=60_000).contains(&frame.duration_ms)
            {
                return Err(invalid("Invalid/duplicate frame ID or duration"));
            }
            for (layer, cel) in &frame.cels {
                if !layer_ids.contains(layer) || cel.len() != self.area() {
                    return Err(invalid(
                        "Cel references unknown layer or has invalid buffer length",
                    ));
                }
                pixels += cel.len();
            }
        }
        if pixels > MAX_PIXELS {
            return Err(CommandError::new(
                "resource_limit",
                "Document pixel budget exceeded",
            ));
        }
        let mut tag_names = BTreeSet::new();
        for tag in &self.tags {
            let start = self.frame_index(tag.from_frame)?;
            let end = self.frame_index(tag.to_frame)?;
            if tag.name.is_empty()
                || tag.name.len() > 128
                || !tag_names.insert(&tag.name)
                || start > end
            {
                return Err(invalid("Invalid animation tag"));
            }
        }
        Ok(())
    }
    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }
    pub fn frame_index(&self, id: Id) -> Result<usize> {
        self.frames
            .iter()
            .position(|f| f.id == id)
            .ok_or_else(|| CommandError::new("not_found", "Frame not found"))
    }
    pub fn pixel_index(&self, x: i32, y: i32) -> Result<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return Err(invalid("Pixel coordinates outside canvas"));
        }
        Ok(y as usize * self.width as usize + x as usize)
    }
    pub fn cel_mut(&mut self, layer: Id, frame: Id) -> Result<&mut Vec<Color>> {
        let l = self
            .layers
            .iter()
            .find(|l| l.id == layer)
            .ok_or_else(|| CommandError::new("not_found", "Layer not found"))?;
        if l.locked {
            return Err(CommandError::new("layer_locked", "Layer is locked"));
        }
        let n = self.area();
        let i = self.frame_index(frame)?;
        Ok(Arc::make_mut(
            self.frames[i]
                .cels
                .entry(layer)
                .or_insert_with(|| Arc::new(vec![[0; 4]; n])),
        ))
    }
    pub fn next_id(&self) -> Id {
        self.layers
            .iter()
            .map(|l| l.id)
            .chain(self.frames.iter().map(|f| f.id))
            .max()
            .unwrap_or(0)
            + 1
    }
    pub fn byte_size(&self) -> usize {
        self.frames
            .iter()
            .flat_map(|f| f.cels.values())
            .map(|c| c.len() * 4)
            .sum()
    }
    /// Integer source-over in stored sRGB RGBA bytes. No implicit color conversion.
    pub fn render(&self, frame: Id) -> Result<Vec<u8>> {
        let f = &self.frames[self.frame_index(frame)?];
        let mut out = vec![0u8; self.area() * 4];
        for layer in self.layers.iter().filter(|l| l.visible) {
            if let Some(cel) = f.cels.get(&layer.id) {
                for (src, dst) in cel.iter().zip(out.as_chunks_mut::<4>().0) {
                    let sa = (src[3] as u32 * layer.opacity as u32 + 127) / 255;
                    let da = dst[3] as u32;
                    let a = sa * 255 + da * (255 - sa);
                    if a == 0 {
                        continue;
                    }
                    for k in 0..3 {
                        dst[k] =
                            ((src[k] as u32 * sa * 255 + dst[k] as u32 * da * (255 - sa) + a / 2)
                                / a) as u8;
                    }
                    dst[3] = ((a + 127) / 255) as u8;
                }
            }
        }
        Ok(out)
    }
}
