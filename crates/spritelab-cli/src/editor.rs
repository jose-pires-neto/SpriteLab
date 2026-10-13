//! Ephemeral editor view configuration shared with automation; never asset undo history.
use assetforge_command::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use spritelab_core::commands::Rect;
use spritelab_core::{
    model::{invalid, Id},
    Engine,
};
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct EditorView {
    pub document_id: Option<String>,
    pub frame_id: Option<Id>,
    pub layer_id: Option<Id>,
    pub tool: String,
    pub color: [u8; 4],
    pub filled: bool,
    pub zoom: u32,
    pub grid: bool,
    pub onion_skin: bool,
    pub playback: bool,
    pub brush_size: u32,
    pub alpha: u8,
    pub mirror_x: bool,
    pub mirror_y: bool,
    pub pixel_perfect: bool,
    pub selection: Option<Rect>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Configure {
    document_id: Option<String>,
    frame_id: Option<Id>,
    layer_id: Option<Id>,
    tool: Option<String>,
    color: Option<[u8; 4]>,
    filled: Option<bool>,
    zoom: Option<u32>,
    grid: Option<bool>,
    onion_skin: Option<bool>,
    playback: Option<bool>,
    brush_size: Option<u32>,
    alpha: Option<u8>,
    mirror_x: Option<bool>,
    mirror_y: Option<bool>,
    pixel_perfect: Option<bool>,
    selection: Option<Rect>,
    #[serde(default)]
    clear_selection: bool,
}
impl EditorView {
    pub fn new() -> Self {
        Self {
            tool: "stroke".into(),
            color: [227, 167, 92, 255],
            zoom: 12,
            brush_size: 1,
            alpha: 255,
            ..Self::default()
        }
    }
    pub fn reconcile(&mut self, engine: &Engine) {
        if let Some(id) = &self.document_id {
            if let Ok(d) = engine.document(id) {
                if self.frame_id.is_none_or(|f| d.frame_index(f).is_err()) {
                    self.frame_id = Some(d.frames[0].id)
                }
                if self
                    .layer_id
                    .is_none_or(|l| !d.layers.iter().any(|a| a.id == l))
                {
                    self.layer_id = Some(d.layers[0].id)
                }
                if self.selection.is_some_and(|r| {
                    r.x < 0
                        || r.y < 0
                        || r.x as i64 + r.width as i64 > d.width as i64
                        || r.y as i64 + r.height as i64 > d.height as i64
                }) {
                    self.selection = None
                }
            } else {
                self.document_id = None;
                self.frame_id = None;
                self.layer_id = None;
                self.selection = None;
                self.playback = false
            }
        }
    }
    pub fn configure(&mut self, c: Configure, engine: &Engine) -> Result<()> {
        let mut v = self.clone();
        if let Some(id) = c.document_id {
            let d = engine.document(&id)?;
            if v.document_id.as_ref() != Some(&id) {
                v.document_id = Some(id);
                v.frame_id = Some(d.frames[0].id);
                v.layer_id = Some(d.layers[0].id);
                v.selection = None;
            }
        }
        macro_rules! field {($($n:ident),*)=>{$(if let Some(a)=c.$n{v.$n=a;})*};}
        field!(
            color,
            filled,
            tool,
            zoom,
            grid,
            onion_skin,
            playback,
            brush_size,
            alpha,
            mirror_x,
            mirror_y,
            pixel_perfect
        );
        // Alpha is the same value as color[3]; explicit alpha wins if both are supplied.
        if let Some(color) = c.color {
            v.alpha = color[3];
        }
        if let Some(alpha) = c.alpha {
            v.alpha = alpha;
        }
        v.color[3] = v.alpha;
        if let Some(id) = c.frame_id {
            v.frame_id = Some(id)
        }
        if let Some(id) = c.layer_id {
            v.layer_id = Some(id)
        }
        if let Some(r) = c.selection {
            v.selection = Some(r)
        }
        if c.clear_selection {
            v.selection = None
        }
        if !(1..=32).contains(&v.zoom)
            || !(1..=32).contains(&v.brush_size)
            || ![
                "stroke",
                "eraser",
                "line",
                "fill",
                "pick",
                "rectangle",
                "ellipse",
                "select",
            ]
            .contains(&v.tool.as_str())
        {
            return Err(invalid("Unsupported tool, zoom or brush size"));
        }
        if let Some(id) = &v.document_id {
            let d = engine.document(id)?;
            if let Some(f) = v.frame_id {
                d.frame_index(f)?;
            }
            if v.layer_id
                .is_some_and(|l| !d.layers.iter().any(|a| a.id == l))
            {
                return Err(invalid("Layer not found"));
            }
            if let Some(r) = v.selection {
                if r.width == 0 || r.height == 0 || r.width > d.width || r.height > d.height {
                    return Err(invalid("Selection outside canvas"));
                }
                d.pixel_index(r.x, r.y)?;
                d.pixel_index(
                    r.x.checked_add(r.width as i32 - 1)
                        .ok_or_else(|| invalid("Selection overflow"))?,
                    r.y.checked_add(r.height as i32 - 1)
                        .ok_or_else(|| invalid("Selection overflow"))?,
                )?;
            }
        }
        *self = v;
        Ok(())
    }
}
