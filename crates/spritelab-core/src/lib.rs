//! Original raster domain implementation. All durable mutations enter Engine::execute.
mod advanced;
pub mod commands;
pub mod model;
use assetforge_command::{decode, CommandError, Descriptor, Request, Response, Result};
use commands::*;
use model::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
const HISTORY_BUDGET: usize = 32 * 1024 * 1024;
struct Session {
    document: Document,
    revision: u64,
    next_id: Id,
    undo: Vec<Document>,
    redo: Vec<Document>,
}
#[derive(Default)]
pub struct Engine {
    sessions: BTreeMap<String, Session>,
}
fn schema<T: schemars::JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).unwrap()
}
pub fn catalog() -> Vec<Descriptor> {
    let mut out = vec![];
    for (id, summary, effect, s) in [
        (
            "document.create",
            "Create a bounded RGBA sprite",
            "lifecycle",
            schema::<Create>(),
        ),
        (
            "document.inspect",
            "Read authoritative document content",
            "query",
            schema::<Empty>(),
        ),
        (
            "document.list",
            "List current session documents",
            "query",
            schema::<Empty>(),
        ),
        (
            "document.close",
            "Release a document session and its unsaved history",
            "lifecycle",
            schema::<Empty>(),
        ),
        (
            "command.list",
            "Discover authoritative command schemas",
            "query",
            schema::<Empty>(),
        ),
        (
            "document.render",
            "Read composited frame RGBA bytes",
            "query",
            schema::<Render>(),
        ),
        (
            "document.onion",
            "Render tinted adjacent frames for the editor overlay",
            "query",
            schema::<Render>(),
        ),
        (
            "document.snapshot",
            "Read document and canonical frame atomically",
            "query",
            schema::<Render>(),
        ),
        (
            "command.preview",
            "Render a detached edit without committing history or IDs",
            "query",
            schema::<Preview>(),
        ),
        (
            "history.undo",
            "Undo one edit or atomic batch",
            "history",
            schema::<Empty>(),
        ),
        (
            "history.redo",
            "Redo one edit or atomic batch",
            "history",
            schema::<Empty>(),
        ),
        (
            "command.batch",
            "Commit edits atomically to one document",
            "edit",
            schema::<Batch>(),
        ),
    ] {
        out.push(Descriptor {
            id: id.into(),
            summary: summary.into(),
            effect: effect.into(),
            undoable: id == "command.batch",
            arguments_schema: s,
        });
    }
    let all = schema::<Edit>();
    for branch in all["oneOf"].as_array().unwrap() {
        let id = branch["properties"]["command"]["enum"][0].as_str().unwrap();
        let mut s = branch["properties"]["args"].clone();
        if all["definitions"].is_object() {
            s["definitions"] = all["definitions"].clone();
        }
        out.push(Descriptor {
            id: id.into(),
            summary: format!("Apply {id} as one reversible edit"),
            effect: "edit".into(),
            undoable: true,
            arguments_schema: s,
        });
    }
    out
}
impl Engine {
    pub fn revision(&self, id: &str) -> Option<u64> {
        self.sessions.get(id).map(|s| s.revision)
    }
    pub fn document(&self, id: &str) -> Result<&Document> {
        self.sessions
            .get(id)
            .map(|s| &s.document)
            .ok_or_else(|| CommandError::new("not_found", "Document not found"))
    }
    pub fn install(&mut self, id: &str, document: Document) -> Result<Value> {
        check_id(id)?;
        document.validate()?;
        if self.sessions.contains_key(id) {
            return Err(CommandError::new(
                "already_exists",
                "Document ID already open",
            ));
        }
        if self.sessions.len() >= 4 {
            return Err(CommandError::new(
                "resource_limit",
                "Prototype supports four documents per session",
            ));
        }
        let next_id = document.next_id();
        self.sessions.insert(
            id.into(),
            Session {
                document,
                revision: 0,
                next_id,
                undo: vec![],
                redo: vec![],
            },
        );
        Ok(json!({"document_id":id,"revision":0}))
    }
    pub fn check_revision(&self, id: &str, expected: Option<u64>) -> Result<()> {
        let actual = self
            .revision(id)
            .ok_or_else(|| CommandError::new("not_found", "Document not found"))?;
        if expected.is_some_and(|e| e != actual) {
            return Err(CommandError::new(
                "revision_conflict",
                format!("Expected {expected:?}, current revision {actual}"),
            ));
        }
        Ok(())
    }
    pub fn execute(&mut self, req: Request) -> Response {
        let result = self.dispatch(&req);
        let revision = req.document_id.as_deref().and_then(|id| self.revision(id));
        Response::from_result(req.request_id, revision, result)
    }
    fn dispatch(&mut self, req: &Request) -> Result<Value> {
        if req.command == "command.list" {
            decode::<Empty>(req.args.clone())?;
            return Ok(json!({"commands":catalog()}));
        }
        if req.command == "document.list" {
            decode::<Empty>(req.args.clone())?;
            return Ok(
                json!({"documents":self.sessions.iter().map(|(id,s)|json!({"document_id":id,"revision":s.revision})).collect::<Vec<_>>()}),
            );
        }
        let id = req
            .document_id
            .as_deref()
            .ok_or_else(|| invalid("document_id required"))?;
        if req.command == "document.create" {
            if req.expected_revision.is_some() {
                return Err(invalid("Creation must omit expected_revision"));
            }
            let a: Create = decode(req.args.clone())?;
            return self.install(id, Document::new(a.width, a.height)?);
        }
        self.check_revision(id, req.expected_revision)?;
        if req.command == "document.close" {
            decode::<Empty>(req.args.clone())?;
            self.sessions.remove(id);
            return Ok(json!({"document_id":id,"closed":true}));
        }
        match req.command.as_str() {
            "document.inspect" => {
                decode::<Empty>(req.args.clone())?;
                Ok(json!({"document":self.document(id)?}))
            }
            "document.onion" => {
                let a: Render = decode(req.args.clone())?;
                let d = self.document(id)?;
                let i = d.frame_index(a.frame_id)?;
                let mut overlay = Document::new(d.width, d.height)?;
                overlay.layers.push(Layer {
                    id: 3,
                    name: "Next onion".into(),
                    visible: true,
                    locked: false,
                    opacity: 255,
                });
                for (index, layer_id, tint) in [
                    (i.checked_sub(1), 1, [220, 80, 80]),
                    ((i + 1 < d.frames.len()).then_some(i + 1), 3, [80, 140, 230]),
                ] {
                    if let Some(index) = index {
                        let mut rgba = d.render(d.frames[index].id)?;
                        for p in rgba.as_chunks_mut::<4>().0 {
                            p[..3].copy_from_slice(&tint);
                            p[3] = (p[3] as u32 * 80 / 255) as u8;
                        }
                        overlay.frames[0].cels.insert(
                            layer_id,
                            std::sync::Arc::new(rgba.as_chunks::<4>().0.to_vec()),
                        );
                    }
                }
                Ok(json!({"width":d.width,"height":d.height,"rgba":overlay.render(2)?}))
            }
            "document.snapshot" => {
                let a: Render = decode(req.args.clone())?;
                let d = self.document(id)?;
                let frame = if d.frames.iter().any(|f| f.id == a.frame_id) {
                    a.frame_id
                } else {
                    d.frames[0].id
                };
                Ok(
                    json!({"document":d,"frame_id":frame,"width":d.width,"height":d.height,"rgba":d.render(frame)?}),
                )
            }
            "document.render" => {
                let a: Render = decode(req.args.clone())?;
                let d = self.document(id)?;
                Ok(json!({"width":d.width,"height":d.height,"rgba":d.render(a.frame_id)?}))
            }
            "command.preview" => {
                let a: Preview = decode(req.args.clone())?;
                let session = self.sessions.get(id).unwrap();
                let mut draft = session.document.clone();
                let mut next_id = session.next_id;
                apply(&mut draft, &mut next_id, a.edit)?;
                draft.validate()?;
                Ok(
                    json!({"width":draft.width,"height":draft.height,"rgba":draft.render(a.frame_id)?}),
                )
            }
            "history.undo" | "history.redo" => {
                decode::<Empty>(req.args.clone())?;
                let s = self.sessions.get_mut(id).unwrap();
                let old = if req.command == "history.undo" {
                    s.undo.pop()
                } else {
                    s.redo.pop()
                }
                .ok_or_else(|| CommandError::new("history_empty", "No history entry"))?;
                let current = std::mem::replace(&mut s.document, old);
                if req.command == "history.undo" {
                    s.redo.push(current)
                } else {
                    s.undo.push(current)
                };
                s.revision += 1;
                Ok(json!({"revision":s.revision}))
            }
            _ => {
                if !catalog().iter().any(|d| d.id == req.command) {
                    return Err(CommandError::new(
                        "unknown_command",
                        "Command not registered",
                    ));
                }
                let edits = if req.command == "command.batch" {
                    decode::<Batch>(req.args.clone())?.commands
                } else {
                    vec![decode::<Edit>(
                        json!({"command":req.command,"args":req.args}),
                    )?]
                };
                if edits.is_empty() || edits.len() > 256 {
                    return Err(invalid("Batch must contain 1..256 edits"));
                }
                let s = self.sessions.get_mut(id).unwrap();
                let mut draft = s.document.clone();
                let mut next_id = s.next_id;
                let mut results = vec![];
                for edit in edits {
                    results.push(apply(&mut draft, &mut next_id, edit)?);
                    draft.validate()?;
                }
                // The candidate commits once; failure above publishes nothing, including IDs/history.
                if draft == s.document {
                    return Ok(json!({"changed":false,"results":results}));
                }
                let old = std::mem::replace(&mut s.document, draft);
                s.undo.push(old);
                s.redo.clear();
                while s.undo.len() > 32
                    || s.undo.iter().map(Document::byte_size).sum::<usize>() > HISTORY_BUDGET
                {
                    s.undo.remove(0);
                }
                s.next_id = next_id;
                s.revision += 1;
                Ok(json!({"changed":true,"results":results}))
            }
        }
    }
}
fn check_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(invalid(
            "ID must be 1..64 ASCII letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}
fn allocate(next: &mut Id) -> Id {
    let id = *next;
    *next += 1;
    id
}
fn apply(d: &mut Document, next: &mut Id, edit: Edit) -> Result<Value> {
    match edit {
        Edit::LayerCreate { name } => {
            if d.layers.len() >= 32 || name.len() > 128 {
                return Err(invalid("Layer limit or name exceeded"));
            }
            let id = allocate(next);
            d.layers.push(Layer {
                id,
                name,
                visible: true,
                locked: false,
                opacity: 255,
            });
            Ok(json!({"layer_id":id}))
        }
        Edit::LayerUpdate {
            layer_id,
            name,
            visible,
            locked,
            opacity,
        } => {
            let l = d
                .layers
                .iter_mut()
                .find(|l| l.id == layer_id)
                .ok_or_else(|| CommandError::new("not_found", "Layer not found"))?;
            if let Some(v) = name {
                l.name = v
            };
            if let Some(v) = visible {
                l.visible = v
            };
            if let Some(v) = locked {
                l.locked = v
            };
            if let Some(v) = opacity {
                l.opacity = v
            };
            Ok(json!({"layer_id":layer_id}))
        }
        Edit::LayerRemove { layer_id } => {
            if d.layers.len() == 1 {
                return Err(invalid("Cannot remove last layer"));
            }
            if !d.layers.iter().any(|l| l.id == layer_id) {
                return Err(CommandError::new("not_found", "Layer not found"));
            }
            d.layers.retain(|l| l.id != layer_id);
            for f in &mut d.frames {
                f.cels.remove(&layer_id);
            }
            Ok(json!({"layer_id":layer_id}))
        }
        Edit::FrameAdd {
            source_frame_id,
            duration_ms,
        } => {
            if d.frames.len() >= 64 {
                return Err(invalid("Frame limit exceeded"));
            }
            let cels = if let Some(id) = source_frame_id {
                d.frames[d.frame_index(id)?].cels.clone()
            } else {
                BTreeMap::new()
            };
            let id = allocate(next);
            d.frames.push(Frame {
                id,
                duration_ms,
                cels,
            });
            Ok(json!({"frame_id":id}))
        }
        Edit::FrameDuration {
            frame_id,
            duration_ms,
        } => {
            let i = d.frame_index(frame_id)?;
            d.frames[i].duration_ms = duration_ms;
            Ok(json!({"frame_id":frame_id}))
        }
        Edit::PaletteSet { colors } => {
            d.palette = colors;
            Ok(json!({"colors":d.palette.len()}))
        }
        Edit::Stroke {
            layer_id,
            frame_id,
            points,
            color,
            size,
            mirror_x,
            mirror_y,
            pixel_perfect,
        } => {
            if points.is_empty() || points.len() > 4096 {
                return Err(invalid("Stroke requires 1..4096 points"));
            }
            for p in &points {
                d.pixel_index(p[0], p[1])?;
            }
            let mut indexes = vec![d.pixel_index(points[0][0], points[0][1])?];
            for pair in points.windows(2) {
                indexes.extend(line_indices(d, pair[0], pair[1])?);
            }
            advanced::brush(
                d,
                layer_id,
                frame_id,
                indexes,
                color,
                size,
                mirror_x,
                mirror_y,
                pixel_perfect,
            )?;
            Ok(json!({"points":points.len()}))
        }
        Edit::Line {
            layer_id,
            frame_id,
            from,
            to,
            color,
        } => {
            let indexes = line_indices(d, from, to)?;
            let count = indexes.len();
            let cel = d.cel_mut(layer_id, frame_id)?;
            for i in indexes {
                cel[i] = color;
            }
            Ok(json!({"pixels":count}))
        }
        Edit::Fill {
            layer_id,
            frame_id,
            at,
            color,
        } => {
            let start = d.pixel_index(at[0], at[1])?;
            let w = d.width as usize;
            let h = d.height as usize;
            let cel = d.cel_mut(layer_id, frame_id)?;
            let old = cel[start];
            if old == color {
                return Ok(json!({"pixels":0}));
            }
            let mut stack = vec![start];
            cel[start] = color;
            let mut count = 0;
            while let Some(i) = stack.pop() {
                count += 1;
                let x = i % w;
                let y = i / w;
                for j in [
                    if x > 0 { Some(i - 1) } else { None },
                    if x + 1 < w { Some(i + 1) } else { None },
                    if y > 0 { Some(i - w) } else { None },
                    if y + 1 < h { Some(i + w) } else { None },
                ]
                .into_iter()
                .flatten()
                {
                    if cel[j] == old {
                        cel[j] = color;
                        stack.push(j);
                    }
                }
            }
            Ok(json!({"pixels":count}))
        }
        Edit::Patch {
            layer_id,
            frame_id,
            x,
            y,
            width,
            height,
            pixels,
        } => {
            if width == 0
                || height == 0
                || width > d.width
                || height > d.height
                || pixels.len() != width as usize * height as usize
            {
                return Err(invalid("Invalid patch dimensions or buffer"));
            }
            d.pixel_index(x, y)?;
            d.pixel_index(
                x.checked_add(width as i32 - 1)
                    .ok_or_else(|| invalid("Coordinate overflow"))?,
                y.checked_add(height as i32 - 1)
                    .ok_or_else(|| invalid("Coordinate overflow"))?,
            )?;
            let dw = d.width as usize;
            let cel = d.cel_mut(layer_id, frame_id)?;
            for row in 0..height as usize {
                let dst = (y as usize + row) * dw + x as usize;
                let src = row * width as usize;
                cel[dst..dst + width as usize].copy_from_slice(&pixels[src..src + width as usize]);
            }
            Ok(json!({"pixels":pixels.len()}))
        }
        Edit::AnimationTag {
            name,
            from_frame,
            to_frame,
        } => {
            let tag = Tag {
                name: name.clone(),
                from_frame,
                to_frame,
            };
            if let Some(t) = d.tags.iter_mut().find(|t| t.name == name) {
                *t = tag
            } else {
                d.tags.push(tag)
            };
            Ok(json!({"tag":name}))
        }
        other => advanced::apply(d, next, other),
    }
}
fn line_indices(d: &Document, from: [i32; 2], to: [i32; 2]) -> Result<Vec<usize>> {
    d.pixel_index(from[0], from[1])?;
    d.pixel_index(to[0], to[1])?;
    let (mut x, mut y) = (from[0], from[1]);
    let dx = (to[0] - x).abs();
    let dy = -(to[1] - y).abs();
    let sx = if x < to[0] { 1 } else { -1 };
    let sy = if y < to[1] { 1 } else { -1 };
    let mut err = dx + dy;
    let mut out = vec![];
    loop {
        out.push(d.pixel_index(x, y)?);
        if [x, y] == to {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests;
