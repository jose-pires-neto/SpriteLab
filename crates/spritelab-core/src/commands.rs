use crate::model::{Color, Id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "command", content = "args", deny_unknown_fields)]
pub enum Edit {
    #[serde(rename = "layer.create")]
    LayerCreate { name: String },
    #[serde(rename = "layer.update")]
    LayerUpdate {
        layer_id: Id,
        name: Option<String>,
        visible: Option<bool>,
        locked: Option<bool>,
        opacity: Option<u8>,
    },
    #[serde(rename = "layer.remove")]
    LayerRemove { layer_id: Id },
    #[serde(rename = "frame.add")]
    FrameAdd {
        source_frame_id: Option<Id>,
        duration_ms: u32,
    },
    #[serde(rename = "frame.duration")]
    FrameDuration { frame_id: Id, duration_ms: u32 },
    #[serde(rename = "palette.set")]
    PaletteSet { colors: Vec<Color> },
    #[serde(rename = "canvas.stroke")]
    Stroke {
        layer_id: Id,
        frame_id: Id,
        points: Vec<[i32; 2]>,
        color: Color,
        #[serde(default = "default_size")]
        size: u32,
        #[serde(default)]
        mirror_x: bool,
        #[serde(default)]
        mirror_y: bool,
        #[serde(default)]
        pixel_perfect: bool,
    },
    #[serde(rename = "canvas.line")]
    Line {
        layer_id: Id,
        frame_id: Id,
        from: [i32; 2],
        to: [i32; 2],
        color: Color,
    },
    #[serde(rename = "canvas.fill")]
    Fill {
        layer_id: Id,
        frame_id: Id,
        at: [i32; 2],
        color: Color,
    },
    #[serde(rename = "canvas.patch")]
    Patch {
        layer_id: Id,
        frame_id: Id,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        pixels: Vec<Color>,
    },
    #[serde(rename = "layer.reorder")]
    LayerReorder { layer_id: Id, index: usize },
    #[serde(rename = "layer.duplicate")]
    LayerDuplicate { layer_id: Id, name: String },
    #[serde(rename = "frame.remove")]
    FrameRemove { frame_id: Id },
    #[serde(rename = "frame.reorder")]
    FrameReorder { frame_id: Id, index: usize },
    #[serde(rename = "cel.clear")]
    CelClear { layer_id: Id, frame_id: Id },
    #[serde(rename = "cel.transfer")]
    CelTransfer {
        source_layer_id: Id,
        source_frame_id: Id,
        layer_id: Id,
        frame_id: Id,
        #[serde(default)]
        move_source: bool,
        #[serde(default)]
        overwrite: bool,
    },
    #[serde(rename = "canvas.rectangle")]
    Rectangle {
        layer_id: Id,
        frame_id: Id,
        rect: Rect,
        color: Color,
        #[serde(default)]
        filled: bool,
    },
    #[serde(rename = "canvas.ellipse")]
    Ellipse {
        layer_id: Id,
        frame_id: Id,
        rect: Rect,
        color: Color,
        #[serde(default)]
        filled: bool,
    },
    #[serde(rename = "canvas.transform")]
    Transform {
        layer_id: Id,
        frame_id: Id,
        rect: Rect,
        operation: RegionOperation,
        #[serde(default)]
        dx: i32,
        #[serde(default)]
        dy: i32,
    },
    #[serde(rename = "document.resize")]
    Resize { width: u32, height: u32 },
    #[serde(rename = "document.canvas_size")]
    CanvasSize {
        width: u32,
        height: u32,
        #[serde(default)]
        offset_x: i32,
        #[serde(default)]
        offset_y: i32,
    },
    #[serde(rename = "canvas.replace_color")]
    ReplaceColor {
        layer_id: Id,
        frame_id: Id,
        from: Color,
        to: Color,
    },
    #[serde(rename = "animation.tag_remove")]
    TagRemove { name: String },
    #[serde(rename = "animation.tag")]
    AnimationTag {
        name: String,
        from_frame: Id,
        to_frame: Id,
    },
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    #[schemars(range(min = 1, max = 512))]
    pub width: u32,
    #[schemars(range(min = 1, max = 512))]
    pub height: u32,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Empty {}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    pub commands: Vec<Edit>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Render {
    pub frame_id: Id,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub edit: Edit,
    pub frame_id: Id,
}

pub fn default_size() -> u32 {
    1
}
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegionOperation {
    FlipX,
    FlipY,
    Rotate90,
    Move,
    Copy,
    Clear,
}
