use super::*;
fn req(e: &mut Engine, command: &str, args: Value, revision: Option<u64>) -> Response {
    e.execute(Request {
        request_id: Some("test".into()),
        document_id: Some("sprite".into()),
        expected_revision: revision,
        command: command.into(),
        args,
    })
}
fn setup() -> Engine {
    let mut e = Engine::default();
    assert!(
        req(
            &mut e,
            "document.create",
            json!({"width":8,"height":8}),
            None
        )
        .ok
    );
    e
}
fn stroke() -> Value {
    json!({"layer_id":1,"frame_id":2,"points":[[1,1],[5,1]],"color":[100,200,50,255]})
}
#[test]
fn stroke_is_one_revision_and_one_undo() {
    let mut e = setup();
    let before = e.document("sprite").unwrap().clone();
    assert!(req(&mut e, "canvas.stroke", stroke(), Some(0)).ok);
    assert_eq!(e.revision("sprite"), Some(1));
    assert_eq!(
        e.document("sprite")
            .unwrap()
            .render(2)
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 0)
            .count(),
        5
    );
    assert!(req(&mut e, "history.undo", json!({}), Some(1)).ok);
    assert_eq!(e.document("sprite").unwrap(), &before);
    assert_eq!(e.revision("sprite"), Some(2));
    assert!(req(&mut e, "history.redo", json!({}), Some(2)).ok);
    assert_eq!(e.revision("sprite"), Some(3));
}
#[test]
fn failed_batch_rolls_back_content_history_and_ids() {
    let mut e = setup();
    let before = e.document("sprite").unwrap().clone();
    let r = req(
        &mut e,
        "command.batch",
        json!({"commands":[{"command":"layer.create","args":{"name":"Candidate"}},{"command":"canvas.line","args":{"layer_id":1,"frame_id":2,"from":[0,0],"to":[999,0],"color":[255,0,0,255]}}]}),
        Some(0),
    );
    assert!(!r.ok);
    assert_eq!(e.document("sprite").unwrap(), &before);
    assert_eq!(e.revision("sprite"), Some(0));
    assert!(!req(&mut e, "history.undo", json!({}), None).ok);
    let r = req(&mut e, "layer.create", json!({"name":"Valid"}), None);
    assert_eq!(r.result.unwrap()["results"][0]["layer_id"], 3);
}
#[test]
fn successful_batch_undoes_as_one_unit() {
    let mut e = setup();
    let before = e.document("sprite").unwrap().clone();
    assert!(req(&mut e,"command.batch",json!({"commands":[{"command":"canvas.stroke","args":stroke()},{"command":"palette.set","args":{"colors":[[100,200,50,255]]}}]}),Some(0)).ok);
    assert_eq!(e.revision("sprite"), Some(1));
    assert!(req(&mut e, "history.undo", json!({}), None).ok);
    assert_eq!(e.document("sprite").unwrap(), &before);
}
#[test]
fn stale_revision_cannot_edit_or_undo() {
    let mut e = setup();
    req(&mut e, "canvas.stroke", stroke(), Some(0));
    let before = e.document("sprite").unwrap().clone();
    for cmd in ["canvas.stroke", "history.undo"] {
        let r = req(
            &mut e,
            cmd,
            if cmd == "canvas.stroke" {
                stroke()
            } else {
                json!({})
            },
            Some(0),
        );
        assert_eq!(r.error.unwrap().code, "revision_conflict");
    }
    assert_eq!(e.document("sprite").unwrap(), &before);
}
#[test]
fn duplicate_frames_have_copy_on_write_pixels() {
    let mut e = setup();
    req(&mut e, "canvas.stroke", stroke(), None);
    let r = req(
        &mut e,
        "frame.add",
        json!({"source_frame_id":2,"duration_ms":140}),
        None,
    );
    let id = r.result.unwrap()["results"][0]["frame_id"]
        .as_u64()
        .unwrap();
    let before = e.document("sprite").unwrap().render(2).unwrap();
    let mut a = stroke();
    a["frame_id"] = json!(id);
    a["color"] = json!([1, 2, 3, 255]);
    assert!(req(&mut e, "canvas.stroke", a, None).ok);
    assert_eq!(e.document("sprite").unwrap().render(2).unwrap(), before);
    assert_ne!(e.document("sprite").unwrap().render(id).unwrap(), before);
}
#[test]
fn ids_are_not_reused_after_undo() {
    let mut e = setup();
    req(&mut e, "layer.create", json!({"name":"Old"}), None);
    req(&mut e, "history.undo", json!({}), None);
    let r = req(&mut e, "layer.create", json!({"name":"New"}), None);
    assert_eq!(r.result.unwrap()["results"][0]["layer_id"], 4);
    assert!(!req(&mut e, "history.redo", json!({}), None).ok);
}
#[test]
fn fill_obeys_four_connected_boundary() {
    let mut e = setup();
    req(
        &mut e,
        "canvas.line",
        json!({"layer_id":1,"frame_id":2,"from":[4,0],"to":[4,7],"color":[255,0,0,255]}),
        None,
    );
    let r = req(
        &mut e,
        "canvas.fill",
        json!({"layer_id":1,"frame_id":2,"at":[0,0],"color":[0,255,0,255]}),
        None,
    );
    assert_eq!(r.result.unwrap()["results"][0]["pixels"], 32);
    let data = e.document("sprite").unwrap().render(2).unwrap();
    assert_eq!(&data[7 * 4..8 * 4], &[0, 0, 0, 0]);
}
#[test]
fn locked_layer_and_unknown_args_reject_without_edits() {
    let mut e = setup();
    req(
        &mut e,
        "layer.update",
        json!({"layer_id":1,"locked":true}),
        None,
    );
    let r = req(&mut e, "canvas.stroke", stroke(), None);
    assert_eq!(r.error.unwrap().code, "layer_locked");
    let mut a = stroke();
    a["typo"] = json!(true);
    assert!(!req(&mut e, "canvas.stroke", a, None).ok);
    assert_eq!(e.revision("sprite"), Some(1));
}
#[test]
fn no_op_keeps_revision_and_redo() {
    let mut e = setup();
    req(&mut e, "canvas.stroke", stroke(), None);
    req(&mut e, "history.undo", json!({}), None);
    assert!(
        req(
            &mut e,
            "layer.update",
            json!({"layer_id":1,"visible":true}),
            None
        )
        .ok
    );
    assert_eq!(e.revision("sprite"), Some(2));
    assert!(req(&mut e, "history.redo", json!({}), None).ok);
}
#[test]
fn compositor_handles_opacity_visibility_and_alpha() {
    let mut e = setup();
    req(
        &mut e,
        "canvas.fill",
        json!({"layer_id":1,"frame_id":2,"at":[0,0],"color":[255,0,0,255]}),
        None,
    );
    req(&mut e, "layer.create", json!({"name":"Top"}), None);
    req(
        &mut e,
        "canvas.fill",
        json!({"layer_id":3,"frame_id":2,"at":[0,0],"color":[0,0,255,128]}),
        None,
    );
    assert_eq!(
        &e.document("sprite").unwrap().render(2).unwrap()[..4],
        &[127, 0, 128, 255]
    );
    req(
        &mut e,
        "layer.update",
        json!({"layer_id":3,"visible":false}),
        None,
    );
    assert_eq!(
        &e.document("sprite").unwrap().render(2).unwrap()[..4],
        &[255, 0, 0, 255]
    );
}
#[test]
fn invalid_documents_and_tag_ranges_fail() {
    let mut e = setup();
    let mut d = e.document("sprite").unwrap().clone();
    d.frames[0].id = 1;
    assert!(d.validate().is_err());
    let r = req(
        &mut e,
        "animation.tag",
        json!({"name":"idle","from_frame":2,"to_frame":99}),
        None,
    );
    assert!(!r.ok);
    assert_eq!(e.revision("sprite"), Some(0));
}
#[test]
fn schemas_describe_every_edit() {
    let c = catalog();
    assert_eq!(c.len(), 35);
    for d in c {
        assert_eq!(d.arguments_schema["type"], "object");
        assert_eq!(d.arguments_schema["additionalProperties"], false);
    }
}

#[test]
fn resource_budget_rejects_an_extra_full_size_cel_atomically() {
    let mut e = Engine::default();
    assert!(
        req(
            &mut e,
            "document.create",
            json!({"width":512,"height":512}),
            None
        )
        .ok
    );
    assert!(
        req(
            &mut e,
            "canvas.fill",
            json!({"layer_id":1,"frame_id":2,"at":[0,0],"color":[255,255,255,255]}),
            None
        )
        .ok
    );
    assert!(
        req(
            &mut e,
            "frame.add",
            json!({"source_frame_id":2,"duration_ms":100}),
            None
        )
        .ok
    );
    let before = e.document("sprite").unwrap().clone();
    let revision = e.revision("sprite");
    let rejected = req(
        &mut e,
        "frame.add",
        json!({"source_frame_id":2,"duration_ms":100}),
        None,
    );
    assert_eq!(rejected.error.unwrap().code, "resource_limit");
    assert_eq!(e.document("sprite").unwrap(), &before);
    assert_eq!(e.revision("sprite"), revision);
    assert!(serde_json::to_vec(&before).unwrap().len() < 16 * 1024 * 1024);
}

#[test]
fn close_releases_a_session_and_huge_loaded_ids_reject() {
    let mut e = setup();
    let mut invalid_doc = e.document("sprite").unwrap().clone();
    invalid_doc.frames[0].id = u64::MAX;
    assert!(invalid_doc.validate().is_err());
    assert!(req(&mut e, "document.close", json!({}), Some(0)).ok);
    assert!(e.document("sprite").is_err());
    assert!(
        req(
            &mut e,
            "document.create",
            json!({"width":8,"height":8}),
            None
        )
        .ok
    );
    assert_eq!(e.revision("sprite"), Some(0));
}

#[test]
fn detached_preview_matches_commit_without_mutating_history_or_ids() {
    let mut e = setup();
    let before = e.document("sprite").unwrap().clone();
    let preview = req(
        &mut e,
        "command.preview",
        json!({"edit":{"command":"canvas.stroke","args":stroke()},"frame_id":2}),
        Some(0),
    );
    assert!(preview.ok);
    assert_eq!(e.revision("sprite"), Some(0));
    assert_eq!(e.document("sprite").unwrap(), &before);
    assert!(!req(&mut e, "history.undo", json!({}), None).ok);
    req(&mut e, "canvas.stroke", stroke(), Some(0));
    assert_eq!(
        preview.result.unwrap()["rgba"],
        json!(e.document("sprite").unwrap().render(2).unwrap())
    );
    req(
        &mut e,
        "command.preview",
        json!({"edit":{"command":"layer.create","args":{"name":"Draft"}},"frame_id":2}),
        Some(1),
    );
    assert_eq!(
        req(&mut e, "layer.create", json!({"name":"Real"}), Some(1))
            .result
            .unwrap()["results"][0]["layer_id"],
        3
    );
}

#[test]
fn advanced_shapes_brush_mirroring_and_perfect_preview() {
    let mut e = setup();
    assert!(req(&mut e,"canvas.rectangle",json!({"layer_id":1,"frame_id":2,"rect":{"x":1,"y":1,"width":4,"height":3},"color":[1,2,3,255],"filled":false}),None).ok);
    let d = e.document("sprite").unwrap();
    assert_eq!(d.frames[0].cels[&1].iter().filter(|p| p[3] > 0).count(), 10);
    req(
        &mut e,
        "cel.clear",
        json!({"layer_id":1,"frame_id":2}),
        None,
    );
    assert!(req(&mut e,"canvas.stroke",json!({"layer_id":1,"frame_id":2,"points":[[0,0]],"color":[9,8,7,255],"size":3,"mirror_x":true,"mirror_y":true}),None).ok);
    assert_eq!(
        e.document("sprite").unwrap().frames[0].cels[&1]
            .iter()
            .filter(|p| p[3] > 0)
            .count(),
        16
    );
    req(
        &mut e,
        "cel.clear",
        json!({"layer_id":1,"frame_id":2}),
        None,
    );
    assert!(req(&mut e,"canvas.stroke",json!({"layer_id":1,"frame_id":2,"points":[[1,1],[2,1],[2,2]],"color":[9,8,7,255],"pixel_perfect":true}),None).ok);
    assert_eq!(
        e.document("sprite").unwrap().frames[0].cels[&1]
            .iter()
            .filter(|p| p[3] > 0)
            .count(),
        2
    );
    assert!(req(&mut e,"canvas.ellipse",json!({"layer_id":1,"frame_id":2,"rect":{"x":3,"y":3,"width":3,"height":3},"color":[1,2,3,255],"filled":true}),None).ok);
}
#[test]
fn overlapping_move_preserves_source_and_failed_transform_is_atomic() {
    let mut e = setup();
    req(
        &mut e,
        "canvas.patch",
        json!({"layer_id":1,"frame_id":2,"x":1,"y":1,"width":3,"height":1,"pixels":[[1,0,0,255],[2,0,0,255],[3,0,0,255]]}),
        None,
    );
    assert!(req(&mut e,"canvas.transform",json!({"layer_id":1,"frame_id":2,"rect":{"x":1,"y":1,"width":3,"height":1},"operation":"move","dx":1}),None).ok);
    assert_eq!(
        &e.document("sprite").unwrap().frames[0].cels[&1][9..13],
        &[[0; 4], [1, 0, 0, 255], [2, 0, 0, 255], [3, 0, 0, 255]]
    );
    let before = e.document("sprite").unwrap().clone();
    let revision = e.revision("sprite");
    assert!(!req(&mut e,"canvas.transform",json!({"layer_id":1,"frame_id":2,"rect":{"x":2,"y":1,"width":3,"height":1},"operation":"move","dx":7}),None).ok);
    assert_eq!(e.document("sprite").unwrap(), &before);
    assert_eq!(e.revision("sprite"), revision);
    req(
        &mut e,
        "canvas.transform",
        json!({"layer_id":1,"frame_id":2,"rect":{"x":2,"y":1,"width":3,"height":1},"operation":"rotate90"}),
        None,
    );
    assert_eq!(
        e.document("sprite").unwrap().frames[0].cels[&1][10],
        [1, 0, 0, 255]
    );
    assert_eq!(
        e.document("sprite").unwrap().frames[0].cels[&1][26],
        [3, 0, 0, 255]
    );
}
#[test]
fn frame_reorder_removal_repair_tag_members_and_undo() {
    let mut e = setup();
    for _ in 0..2 {
        req(&mut e, "frame.add", json!({"duration_ms":120}), None);
    }
    req(
        &mut e,
        "animation.tag",
        json!({"name":"walk","from_frame":2,"to_frame":3}),
        None,
    );
    let before = e.document("sprite").unwrap().clone();
    assert!(
        req(
            &mut e,
            "frame.reorder",
            json!({"frame_id":2,"index":2}),
            None
        )
        .ok
    );
    let d = e.document("sprite").unwrap();
    assert_eq!(d.tags[0].from_frame, 3);
    assert_eq!(d.tags[0].to_frame, 2);
    assert!(req(&mut e, "frame.remove", json!({"frame_id":2}), None).ok);
    assert!(e.document("sprite").unwrap().validate().is_ok());
    req(&mut e, "history.undo", json!({}), None);
    req(&mut e, "history.undo", json!({}), None);
    assert_eq!(e.document("sprite").unwrap(), &before);
    req(
        &mut e,
        "animation.tag",
        json!({"name":"solo","from_frame":4,"to_frame":4}),
        None,
    );
    req(&mut e, "frame.remove", json!({"frame_id":4}), None);
    assert!(!e
        .document("sprite")
        .unwrap()
        .tags
        .iter()
        .any(|t| t.name == "solo"));
}
#[test]
fn duplicate_layer_detaches_and_resize_roundtrips() {
    let mut e = setup();
    req(&mut e, "canvas.stroke", stroke(), None);
    let before = e.document("sprite").unwrap().clone();
    assert!(
        req(
            &mut e,
            "layer.duplicate",
            json!({"layer_id":1,"name":"Copy"}),
            None
        )
        .ok
    );
    req(
        &mut e,
        "canvas.replace_color",
        json!({"layer_id":3,"frame_id":2,"from":[100,200,50,255],"to":[9,9,9,255]}),
        None,
    );
    assert_eq!(
        e.document("sprite").unwrap().frames[0].cels[&1],
        before.frames[0].cels[&1]
    );
    req(
        &mut e,
        "document.resize",
        json!({"width":16,"height":16}),
        None,
    );
    assert_eq!(
        e.document("sprite").unwrap().frames[0].cels[&1][34],
        [100, 200, 50, 255]
    );
    req(
        &mut e,
        "document.canvas_size",
        json!({"width":8,"height":8,"offset_x":-1,"offset_y":-1}),
        None,
    );
    assert!(e.document("sprite").unwrap().validate().is_ok());
    for _ in 0..4 {
        req(&mut e, "history.undo", json!({}), None);
    }
    assert_eq!(e.document("sprite").unwrap(), &before);
}
#[test]
fn snapshot_and_onion_are_read_only() {
    let mut e = setup();
    req(&mut e, "canvas.stroke", stroke(), None);
    req(&mut e, "frame.add", json!({"duration_ms":100}), None);
    let revision = e.revision("sprite");
    let before = e.document("sprite").unwrap().clone();
    let snapshot = req(&mut e, "document.snapshot", json!({"frame_id":3}), None)
        .result
        .unwrap();
    assert_eq!(snapshot["document"], json!(before));
    let onion = req(&mut e, "document.onion", json!({"frame_id":3}), None)
        .result
        .unwrap();
    assert_eq!(onion["rgba"][36], 220);
    assert_eq!(onion["rgba"][39], 80);
    assert_eq!(e.document("sprite").unwrap(), &before);
    assert_eq!(e.revision("sprite"), revision);
}
