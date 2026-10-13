//! Shared file/runtime boundary for CLI, MCP and the future Tauri shell.
pub mod editor;
mod recovery;
pub mod session;
use assetforge_command::{decode, CommandError, Descriptor, Request, Response, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use spritelab_core::{
    commands::Empty,
    model::{invalid, Document, Id},
    Engine,
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
pub const MAX_INPUT: usize = 16 * 1024 * 1024;
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReadPath {
    path: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WritePath {
    path: String,
    #[serde(default)]
    overwrite: bool,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PngArgs {
    path: String,
    frame_id: Id,
    #[serde(default)]
    overwrite: bool,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SheetArgs {
    path: String,
    columns: u32,
    #[serde(default)]
    padding: u32,
    #[serde(default)]
    overwrite: bool,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PreviewArgs {
    frame_id: Id,
    layer_id: Option<Id>,
}
pub fn catalog() -> Vec<Descriptor> {
    let mut out = spritelab_core::catalog();
    out.push(Descriptor {
        id: "workspace.inspect".into(),
        summary: "List bounded project/PNG/GIF files in the current workspace".into(),
        effect: "query".into(),
        undoable: false,
        arguments_schema: serde_json::to_value(schemars::schema_for!(Empty)).unwrap(),
    });
    out.push(Descriptor {
        id: "editor.inspect".into(),
        summary: "Read shared ephemeral editor context and asset revision".into(),
        effect: "query".into(),
        undoable: false,
        arguments_schema: serde_json::to_value(schemars::schema_for!(Empty)).unwrap(),
    });
    out.push(Descriptor {
        id: "editor.configure".into(),
        summary: "Configure shared editor context without asset undo history".into(),
        effect: "view".into(),
        undoable: false,
        arguments_schema: serde_json::to_value(schemars::schema_for!(editor::Configure)).unwrap(),
    });
    for (id, summary, effect, s) in [
        (
            "document.open",
            "Load a versioned SpriteLab project under a new session ID",
            "lifecycle",
            serde_json::to_value(schemars::schema_for!(ReadPath)).unwrap(),
        ),
        (
            "document.save",
            "Atomically save an editable project inside the workspace",
            "file_write",
            serde_json::to_value(schemars::schema_for!(WritePath)).unwrap(),
        ),
        (
            "document.preview",
            "Render a canonical PNG without writing a file",
            "query",
            serde_json::to_value(schemars::schema_for!(PreviewArgs)).unwrap(),
        ),
        (
            "import.png",
            "Create a document from a bounded PNG",
            "lifecycle",
            serde_json::to_value(schemars::schema_for!(ReadPath)).unwrap(),
        ),
        (
            "export.gif",
            "Export animated GIF (quantized color, binary alpha, centisecond timing)",
            "file_write",
            serde_json::to_value(schemars::schema_for!(WritePath)).unwrap(),
        ),
        (
            "export.png",
            "Export the canonical composited frame PNG",
            "file_write",
            serde_json::to_value(schemars::schema_for!(PngArgs)).unwrap(),
        ),
        (
            "export.spritesheet",
            "Export an untrimmed row-major PNG atlas; return metadata",
            "file_write",
            serde_json::to_value(schemars::schema_for!(SheetArgs)).unwrap(),
        ),
    ] {
        out.push(Descriptor {
            id: id.into(),
            summary: summary.into(),
            effect: effect.into(),
            undoable: false,
            arguments_schema: s,
        });
    }
    out.extend(recovery::catalog());
    out
}
pub struct Runtime {
    pub engine: Engine,
    root: PathBuf,
    view: editor::EditorView,
    recovery: recovery::Recovery,
    saved: std::collections::BTreeMap<String, (String, Document)>,
}
impl Runtime {
    pub fn new(root: &Path) -> Result<Self> {
        let root = root.canonicalize().map_err(io_error)?;
        if !root.is_dir() {
            return Err(invalid("Workspace must be a directory"));
        }
        Ok(Self {
            recovery: recovery::Recovery::new(),
            engine: Engine::default(),
            root,
            view: editor::EditorView::new(),
            saved: Default::default(),
        })
    }
    pub fn execute(&mut self, req: Request) -> Response {
        let special = req.command.starts_with("recovery.")
            || matches!(
                req.command.as_str(),
                "workspace.inspect"
                    | "document.list"
                    | "editor.inspect"
                    | "editor.configure"
                    | "command.list"
                    | "document.open"
                    | "document.save"
                    | "document.preview"
                    | "import.png"
                    | "export.gif"
                    | "export.png"
                    | "export.spritesheet"
            );
        if !special {
            let response = self.engine.execute(req);
            self.view.reconcile(&self.engine);
            self.saved.retain(|id, _| self.engine.document(id).is_ok());
            return response;
        }
        let result = self.dispatch(&req);
        self.view.reconcile(&self.engine);
        let revision = req
            .document_id
            .as_deref()
            .and_then(|id| self.engine.revision(id));
        Response::from_result(req.request_id, revision, result)
    }
    fn path(&self, text: &str, write: bool) -> Result<PathBuf> {
        let p = Path::new(text);
        if p.as_os_str().is_empty()
            || p.is_absolute()
            || p.components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(CommandError::new(
                "path_denied",
                "Use a relative workspace path without '.' or '..'",
            ));
        }
        let p = self.root.join(p);
        let parent = p.parent().unwrap().canonicalize().map_err(io_error)?;
        if !parent.starts_with(&self.root) {
            return Err(CommandError::new(
                "path_denied",
                "Parent resolves outside workspace",
            ));
        }
        if !write || p.exists() || p.symlink_metadata().is_ok() {
            let real = p.canonicalize().map_err(io_error)?;
            if !real.starts_with(&self.root) {
                return Err(CommandError::new(
                    "path_denied",
                    "Path resolves outside workspace",
                ));
            }
            if !real.is_file() {
                return Err(invalid("Path must be a regular file"));
            }
            // Do not replace a symlink or follow one for writes.
            if write
                && p.symlink_metadata()
                    .map_err(io_error)?
                    .file_type()
                    .is_symlink()
            {
                return Err(CommandError::new(
                    "path_denied",
                    "Write target is a symlink",
                ));
            }
        }
        Ok(parent.join(p.file_name().unwrap()))
    }
    fn read(&self, path: &str) -> Result<Vec<u8>> {
        let p = self.path(path, false)?;
        let file = fs::File::open(p).map_err(io_error)?;
        let mut bytes = vec![];
        file.take(MAX_INPUT as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > MAX_INPUT {
            return Err(CommandError::new("resource_limit", "File exceeds 16 MiB"));
        }
        Ok(bytes)
    }
    fn document_statuses(&mut self) -> Result<Value> {
        let response = self.engine.execute(Request {
            request_id: None,
            document_id: None,
            expected_revision: None,
            command: "document.list".into(),
            args: json!({}),
        });
        let mut result = response
            .result
            .ok_or_else(|| invalid("Cannot inspect documents"))?;
        for entry in result["documents"].as_array_mut().unwrap() {
            let id = entry["document_id"].as_str().unwrap();
            let baseline = self.saved.get(id);
            let dirty = self
                .engine
                .document(id)
                .is_ok_and(|d| baseline.is_none_or(|(_, saved)| d != saved));
            let path = baseline.map(|(path, _)| path.clone());
            entry["dirty"] = json!(dirty);
            entry["path"] = json!(path);
        }
        Ok(result)
    }
    fn dispatch(&mut self, r: &Request) -> Result<Value> {
        if r.command.starts_with("recovery.") {
            return self.dispatch_recovery(r);
        }
        if r.command == "document.list" {
            decode::<Empty>(r.args.clone())?;
            return self.document_statuses();
        }
        if r.command == "workspace.inspect" {
            decode::<Empty>(r.args.clone())?;
            let mut files = Vec::new();
            for entry in fs::read_dir(&self.root).map_err(io_error)? {
                let entry = entry.map_err(io_error)?;
                if entry.file_type().map_err(io_error)?.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if [".spritelab", ".png", ".gif"]
                        .iter()
                        .any(|ext| name.ends_with(ext))
                    {
                        files.push(name);
                        if files.len() > 1024 {
                            return Err(CommandError::new(
                                "resource_limit",
                                "Workspace listing exceeds 1024 files",
                            ));
                        }
                    }
                }
            }
            files.sort();
            return Ok(json!({"path":self.root,"files":files}));
        }
        if r.command == "editor.configure" {
            let c: editor::Configure = decode(r.args.clone())?;
            self.view.configure(c, &self.engine)?;
            return Ok(json!({"view":self.view}));
        }
        if r.command == "editor.inspect" {
            decode::<Empty>(r.args.clone())?;
            let documents = self.document_statuses()?["documents"].clone();
            let id = self.view.document_id.as_deref();
            let baseline = id.and_then(|id| self.saved.get(id));
            let dirty = id
                .and_then(|id| self.engine.document(id).ok())
                .is_some_and(|d| baseline.is_none_or(|(_, saved)| d != saved));
            return Ok(
                json!({"view":self.view,"document_revision":id.and_then(|id|self.engine.revision(id)),"document_path":baseline.map(|(path,_)|path),"dirty":dirty,"documents":documents}),
            );
        }
        if r.command == "command.list" {
            decode::<Empty>(r.args.clone())?;
            return Ok(json!({"commands":catalog()}));
        }
        let id = r
            .document_id
            .as_deref()
            .ok_or_else(|| invalid("document_id required"))?;
        if matches!(r.command.as_str(), "document.open" | "import.png") {
            if r.expected_revision.is_some() {
                return Err(invalid("Open/import must omit expected_revision"));
            }
            let a: ReadPath = decode(r.args.clone())?;
            let bytes = self.read(&a.path)?;
            let d = if r.command == "document.open" {
                decode::<Document>(
                    serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?,
                )?
            } else {
                import_png(&bytes)?
            };
            let result = self.engine.install(id, d.clone())?;
            if r.command == "document.open" {
                self.saved.insert(id.into(), (a.path, d));
            }
            return Ok(result);
        }
        self.engine.check_revision(id, r.expected_revision)?;
        let d = self.engine.document(id)?;
        match r.command.as_str() {
            "document.save" => {
                let a: WritePath = decode(r.args.clone())?;
                let p = self.path(&a.path, true)?;
                atomic_write(
                    &p,
                    &serde_json::to_vec(d).map_err(|e| invalid(e.to_string()))?,
                    a.overwrite,
                )?;
                self.saved.insert(id.into(), (a.path.clone(), d.clone()));
                Ok(json!({"path":a.path,"format_version":1}))
            }
            "document.preview" => {
                let a: PreviewArgs = decode(r.args.clone())?;
                let rgba = match a.layer_id {
                    Some(layer) => d.render_layer(layer, a.frame_id)?,
                    None => d.render(a.frame_id)?,
                };
                let png = encode_png(d.width, d.height, &rgba)?;
                Ok(
                    json!({"mime_type":"image/png","width":d.width,"height":d.height,"png_base64":STANDARD.encode(png)}),
                )
            }
            "export.gif" => {
                let a: WritePath = decode(r.args.clone())?;
                let p = self.path(&a.path, true)?;
                if d.area() * d.frames.len() > 4_194_304 {
                    return Err(CommandError::new(
                        "resource_limit",
                        "GIF exceeds 4M rendered pixels",
                    ));
                }
                let mut bytes = Vec::new();
                let mut partial_alpha = false;
                {
                    let mut encoder =
                        gif::Encoder::new(&mut bytes, d.width as u16, d.height as u16, &[])
                            .map_err(|e| invalid(e.to_string()))?;
                    encoder
                        .set_repeat(gif::Repeat::Infinite)
                        .map_err(|e| invalid(e.to_string()))?;
                    for f in &d.frames {
                        let mut rgba = d.render(f.id)?;
                        for p in rgba.as_chunks_mut::<4>().0 {
                            if p[3] > 0 && p[3] < 255 {
                                partial_alpha = true
                            }
                            p[3] = if p[3] < 128 { 0 } else { 255 };
                        }
                        let mut frame = gif::Frame::from_rgba_speed(
                            d.width as u16,
                            d.height as u16,
                            &mut rgba,
                            10,
                        );
                        frame.delay = ((f.duration_ms + 5) / 10).max(1) as u16;
                        frame.dispose = gif::DisposalMethod::Background;
                        encoder
                            .write_frame(&frame)
                            .map_err(|e| invalid(e.to_string()))?;
                    }
                }
                atomic_write(&p, &bytes, a.overwrite)?;
                Ok(
                    json!({"path":a.path,"frames":d.frames.len(),"partial_alpha_thresholded":partial_alpha,"timing_unit_ms":10,"palette_limit":256}),
                )
            }
            "export.png" => {
                let a: PngArgs = decode(r.args.clone())?;
                let p = self.path(&a.path, true)?;
                let png = encode_png(d.width, d.height, &d.render(a.frame_id)?)?;
                atomic_write(&p, &png, a.overwrite)?;
                Ok(json!({"path":a.path,"width":d.width,"height":d.height}))
            }
            "export.spritesheet" => {
                let a: SheetArgs = decode(r.args.clone())?;
                if a.columns == 0 || a.columns > d.frames.len() as u32 || a.padding > 64 {
                    return Err(invalid("columns must be 1..frame count; padding 0..64"));
                }
                let rows = (d.frames.len() as u32).div_ceil(a.columns);
                let w = (d.width + a.padding) * a.columns + a.padding;
                let h = (d.height + a.padding) * rows + a.padding;
                if w as usize * h as usize > 4_194_304 {
                    return Err(CommandError::new(
                        "resource_limit",
                        "Atlas exceeds 4M pixels",
                    ));
                }
                let mut rgba = vec![0u8; w as usize * h as usize * 4];
                let mut frames = vec![];
                for (i, f) in d.frames.iter().enumerate() {
                    let x = a.padding + (i as u32 % a.columns) * (d.width + a.padding);
                    let y = a.padding + (i as u32 / a.columns) * (d.height + a.padding);
                    let pixels = d.render(f.id)?;
                    for row in 0..d.height as usize {
                        let dst = ((y as usize + row) * w as usize + x as usize) * 4;
                        let src = row * d.width as usize * 4;
                        rgba[dst..dst + d.width as usize * 4]
                            .copy_from_slice(&pixels[src..src + d.width as usize * 4]);
                    }
                    frames.push(json!({"frame_id":f.id,"frame":{"x":x,"y":y,"w":d.width,"h":d.height},"duration_ms":f.duration_ms}));
                }
                let p = self.path(&a.path, true)?;
                atomic_write(&p, &encode_png(w, h, &rgba)?, a.overwrite)?;
                Ok(
                    json!({"path":a.path,"metadata":{"schema":"spritelab.atlas/1","image":a.path,"size":{"w":w,"h":h},"frames":frames,"tags":d.tags}}),
                )
            }
            _ => unreachable!(),
        }
    }
}
fn io_error(e: std::io::Error) -> CommandError {
    CommandError::new("io_error", e.to_string())
}
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
fn atomic_write(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(CommandError::new(
            "overwrite_required",
            "File exists; pass overwrite=true explicitly",
        ));
    }
    let temp = path.with_file_name(format!(
        ".spritelab-{}-{}.tmp",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(io_error)?;
    let result = (|| {
        file.write_all(bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
        drop(file);
        if overwrite {
            fs::rename(&temp, path).map_err(io_error)?;
        } else {
            fs::hard_link(&temp, path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    CommandError::new("overwrite_required", "File created concurrently")
                } else {
                    io_error(e)
                }
            })?;
            fs::remove_file(&temp).map_err(io_error)?;
        }
        Ok(())
    })();
    if temp.exists() {
        let _ = fs::remove_file(temp);
    }
    result
}
fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let mut bytes = vec![];
    {
        let mut enc = png::Encoder::new(&mut bytes, width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(|e| invalid(e.to_string()))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| invalid(e.to_string()))?;
    }
    Ok(bytes)
}
fn import_png(bytes: &[u8]) -> Result<Document> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| invalid(e.to_string()))?;
    let info = reader.info();
    let mut d = Document::new(info.width, info.height)?;
    if info.animation_control.is_some() {
        return Err(invalid("Animated PNG import is unsupported"));
    }
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| invalid(e.to_string()))?;
    let mut pixels = vec![];
    for p in buf[..info.buffer_size()].chunks_exact(info.color_type.samples()) {
        pixels.push(match info.color_type {
            png::ColorType::Rgba => [p[0], p[1], p[2], p[3]],
            png::ColorType::Rgb => [p[0], p[1], p[2], 255],
            png::ColorType::Grayscale => [p[0], p[0], p[0], 255],
            png::ColorType::GrayscaleAlpha => [p[0], p[0], p[0], p[1]],
            _ => return Err(invalid("Unsupported PNG pixel format")),
        });
    }
    d.frames[0].cels.insert(1, std::sync::Arc::new(pixels));
    d.validate()?;
    Ok(d)
}
#[cfg(test)]
mod tests;
