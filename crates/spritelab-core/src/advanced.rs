use crate::{allocate, commands::*, model::*};
use assetforge_command::{CommandError, Result};
use serde_json::{json, Value};
use std::sync::Arc;
fn rect_check(d: &Document, r: Rect) -> Result<()> {
    if r.width == 0 || r.height == 0 || r.width > d.width || r.height > d.height {
        return Err(invalid("Invalid region dimensions"));
    }
    d.pixel_index(r.x, r.y)?;
    d.pixel_index(
        r.x.checked_add(r.width as i32 - 1)
            .ok_or_else(|| invalid("Coordinate overflow"))?,
        r.y.checked_add(r.height as i32 - 1)
            .ok_or_else(|| invalid("Coordinate overflow"))?,
    )?;
    Ok(())
}
fn tag_members(d: &Document) -> Vec<Vec<Id>> {
    d.tags
        .iter()
        .map(|t| {
            d.frames[d.frame_index(t.from_frame).unwrap()..=d.frame_index(t.to_frame).unwrap()]
                .iter()
                .map(|f| f.id)
                .collect()
        })
        .collect()
}
fn retag(d: &mut Document, members: Vec<Vec<Id>>) {
    let frames = &d.frames;
    d.tags = std::mem::take(&mut d.tags)
        .into_iter()
        .zip(members)
        .filter_map(|(mut t, ids)| {
            let indexes: Vec<_> = frames
                .iter()
                .enumerate()
                .filter(|(_, f)| ids.contains(&f.id))
                .map(|(i, _)| i)
                .collect();
            let first = *indexes.first()?;
            t.from_frame = frames[first].id;
            t.to_frame = frames[*indexes.last().unwrap()].id;
            Some(t)
        })
        .collect();
}
pub fn apply(d: &mut Document, next: &mut Id, edit: Edit) -> Result<Value> {
    match edit {
        Edit::LayerReorder { layer_id, index } => {
            if index >= d.layers.len() {
                return Err(invalid("Layer index outside range"));
            }
            let old = d
                .layers
                .iter()
                .position(|l| l.id == layer_id)
                .ok_or_else(|| invalid("Layer not found"))?;
            let l = d.layers.remove(old);
            d.layers.insert(index, l);
            Ok(json!({"layer_id":layer_id}))
        }
        Edit::LayerDuplicate { layer_id, name } => {
            if d.layers.len() >= 32 {
                return Err(invalid("Layer limit exceeded"));
            }
            let old = d
                .layers
                .iter()
                .position(|l| l.id == layer_id)
                .ok_or_else(|| invalid("Layer not found"))?;
            let mut l = d.layers[old].clone();
            l.id = allocate(next);
            l.name = name;
            let id = l.id;
            d.layers.insert(old + 1, l);
            for f in &mut d.frames {
                if let Some(c) = f.cels.get(&layer_id).cloned() {
                    f.cels.insert(id, c);
                }
            }
            Ok(json!({"layer_id":id}))
        }
        Edit::FrameRemove { frame_id } => {
            if d.frames.len() == 1 {
                return Err(invalid("Cannot remove last frame"));
            }
            let i = d.frame_index(frame_id)?;
            let tags = tag_members(d);
            d.frames.remove(i);
            retag(d, tags);
            Ok(json!({"frame_id":frame_id}))
        }
        Edit::FrameReorder { frame_id, index } => {
            if index >= d.frames.len() {
                return Err(invalid("Frame index outside range"));
            }
            let i = d.frame_index(frame_id)?;
            let tags = tag_members(d);
            let f = d.frames.remove(i);
            d.frames.insert(index, f);
            retag(d, tags);
            Ok(json!({"frame_id":frame_id}))
        }
        Edit::CelClear { layer_id, frame_id } => {
            let i = d.frame_index(frame_id)?;
            d.cel_mut(layer_id, frame_id)?;
            d.frames[i].cels.remove(&layer_id);
            Ok(json!({"frame_id":frame_id,"layer_id":layer_id}))
        }
        Edit::Rectangle {
            layer_id,
            frame_id,
            rect,
            color,
            filled,
        } => shape(d, layer_id, frame_id, rect, color, filled, false),
        Edit::Ellipse {
            layer_id,
            frame_id,
            rect,
            color,
            filled,
        } => shape(d, layer_id, frame_id, rect, color, filled, true),
        Edit::ReplaceColor {
            layer_id,
            frame_id,
            from,
            to,
        } => {
            let cel = d.cel_mut(layer_id, frame_id)?;
            let mut count = 0;
            for p in cel {
                if *p == from {
                    *p = to;
                    count += 1
                }
            }
            Ok(json!({"pixels":count}))
        }
        Edit::TagRemove { name } => {
            if !d.tags.iter().any(|t| t.name == name) {
                return Err(invalid("Tag not found"));
            }
            d.tags.retain(|t| t.name != name);
            Ok(json!({"tag":name}))
        }
        Edit::Transform {
            layer_id,
            frame_id,
            rect,
            operation,
            dx,
            dy,
        } => {
            rect_check(d, rect)?;
            if !matches!(operation, RegionOperation::Copy | RegionOperation::Move)
                && (dx != 0 || dy != 0)
            {
                return Err(invalid("Offsets only apply to move/copy"));
            }
            let (tw, th) = if matches!(operation, RegionOperation::Rotate90) {
                (rect.height, rect.width)
            } else {
                (rect.width, rect.height)
            };
            let x = rect
                .x
                .checked_add(dx)
                .ok_or_else(|| invalid("Offset overflow"))?;
            let y = rect
                .y
                .checked_add(dy)
                .ok_or_else(|| invalid("Offset overflow"))?;
            rect_check(
                d,
                Rect {
                    x,
                    y,
                    width: tw,
                    height: th,
                },
            )?;
            let dw = d.width as usize;
            let cel = d.cel_mut(layer_id, frame_id)?;
            let mut source = Vec::new();
            for j in 0..rect.height as usize {
                let i = (rect.y as usize + j) * dw + rect.x as usize;
                source.extend_from_slice(&cel[i..i + rect.width as usize]);
            }
            if matches!(
                operation,
                RegionOperation::Move | RegionOperation::Clear | RegionOperation::Rotate90
            ) {
                for j in 0..rect.height as usize {
                    let i = (rect.y as usize + j) * dw + rect.x as usize;
                    cel[i..i + rect.width as usize].fill([0; 4]);
                }
            }
            if !matches!(operation, RegionOperation::Clear) {
                for sy in 0..rect.height as usize {
                    for sx in 0..rect.width as usize {
                        let (tx, ty) = match operation {
                            RegionOperation::FlipX => (rect.width as usize - 1 - sx, sy),
                            RegionOperation::FlipY => (sx, rect.height as usize - 1 - sy),
                            RegionOperation::Rotate90 => (rect.height as usize - 1 - sy, sx),
                            _ => (sx, sy),
                        };
                        cel[(y as usize + ty) * dw + x as usize + tx] =
                            source[sy * rect.width as usize + sx];
                    }
                }
            }
            Ok(json!({"rect":{"x":x,"y":y,"width":tw,"height":th}}))
        }
        Edit::Resize { width, height } => resize(d, width, height, None),
        Edit::CanvasSize {
            width,
            height,
            offset_x,
            offset_y,
        } => resize(d, width, height, Some((offset_x, offset_y))),
        _ => Err(CommandError::new("unknown_command", "Unsupported edit")),
    }
}
fn resize(d: &mut Document, width: u32, height: u32, offset: Option<(i32, i32)>) -> Result<Value> {
    if width == 0 || height == 0 || width > 512 || height > 512 {
        return Err(invalid("Dimensions must be 1..512"));
    }
    let count = d.frames.iter().map(|f| f.cels.len()).sum::<usize>();
    if count * width as usize * height as usize > MAX_PIXELS {
        return Err(CommandError::new(
            "resource_limit",
            "Resized cels exceed pixel budget",
        ));
    }
    for f in &mut d.frames {
        for cel in f.cels.values_mut() {
            let mut pixels = vec![[0; 4]; width as usize * height as usize];
            for y in 0..height {
                for x in 0..width {
                    let (sx, sy) = if let Some((ox, oy)) = offset {
                        (x as i64 - ox as i64, y as i64 - oy as i64)
                    } else {
                        (
                            (x as u64 * d.width as u64 / width as u64) as i64,
                            (y as u64 * d.height as u64 / height as u64) as i64,
                        )
                    };
                    if sx >= 0 && sy >= 0 && sx < d.width as i64 && sy < d.height as i64 {
                        pixels[(y * width + x) as usize] = cel[(sy * d.width as i64 + sx) as usize]
                    }
                }
            }
            *cel = Arc::new(pixels);
        }
    }
    d.width = width;
    d.height = height;
    Ok(json!({"width":width,"height":height}))
}
fn shape(
    d: &mut Document,
    layer: Id,
    frame: Id,
    r: Rect,
    color: Color,
    filled: bool,
    ellipse: bool,
) -> Result<Value> {
    rect_check(d, r)?;
    let dw = d.width as usize;
    let cel = d.cel_mut(layer, frame)?;
    let inside = |x: i64, y: i64| {
        if x < 0 || y < 0 || x >= r.width as i64 || y >= r.height as i64 {
            return false;
        }
        if !ellipse {
            return true;
        }
        let w = r.width as i64;
        let h = r.height as i64;
        let dx = 2 * x + 1 - w;
        let dy = 2 * y + 1 - h;
        dx * dx * h * h + dy * dy * w * w <= w * w * h * h
    };
    let mut count = 0;
    for y in 0..r.height as i64 {
        for x in 0..r.width as i64 {
            if inside(x, y)
                && (filled
                    || [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                        .iter()
                        .any(|&(a, b)| !inside(a, b)))
            {
                cel[(r.y as usize + y as usize) * dw + r.x as usize + x as usize] = color;
                count += 1
            }
        }
    }
    Ok(json!({"pixels":count}))
}
#[allow(clippy::too_many_arguments)]
pub fn brush(
    d: &mut Document,
    layer: Id,
    frame: Id,
    indexes: Vec<usize>,
    color: Color,
    size: u32,
    mx: bool,
    my: bool,
    perfect: bool,
) -> Result<()> {
    if !(1..=32).contains(&size) {
        return Err(invalid("Brush size must be 1..32"));
    }
    let w = d.width as i32;
    let h = d.height as i32;
    let mut samples: Vec<(i32, i32)> = Vec::new();
    for i in indexes {
        let p = ((i % d.width as usize) as i32, (i / d.width as usize) as i32);
        if samples.last() == Some(&p) {
            continue;
        }
        if perfect && samples.len() >= 2 {
            let a = samples[samples.len() - 2];
            let b = samples[samples.len() - 1];
            if (a.0 - p.0).abs() == 1
                && (a.1 - p.1).abs() == 1
                && ((a.0 == b.0 && b.1 == p.1) || (a.1 == b.1 && b.0 == p.0))
            {
                samples.pop();
            }
        }
        samples.push(p);
    }
    let cel = d.cel_mut(layer, frame)?;
    for (x, y) in samples {
        for cx in [Some(x), mx.then_some(w - 1 - x)].into_iter().flatten() {
            for cy in [Some(y), my.then_some(h - 1 - y)].into_iter().flatten() {
                for dy in 0..size as i32 {
                    for dx in 0..size as i32 {
                        let px = cx + dx - size as i32 / 2;
                        let py = cy + dy - size as i32 / 2;
                        if px >= 0 && py >= 0 && px < w && py < h {
                            cel[(py * w + px) as usize] = color
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
