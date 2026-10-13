use super::*;
fn test_root(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("spritelab-test-{}-{name}", std::process::id()));
    if p.exists() {
        fs::remove_dir_all(&p).unwrap();
    }
    fs::create_dir_all(&p).unwrap();
    p
}
fn call(r: &mut Runtime, id: &str, c: &str, a: Value) -> Response {
    r.execute(Request {
        request_id: None,
        document_id: Some(id.into()),
        expected_revision: None,
        command: c.into(),
        args: a,
    })
}
#[test]
fn project_png_atlas_roundtrip() {
    let root = test_root("roundtrip");
    let mut r = Runtime::new(&root).unwrap();
    assert!(
        call(
            &mut r,
            "a",
            "document.create",
            json!({"width":4,"height":4})
        )
        .ok
    );
    assert!(
        call(
            &mut r,
            "a",
            "canvas.fill",
            json!({"layer_id":1,"frame_id":2,"at":[0,0],"color":[25,50,75,255]})
        )
        .ok
    );
    assert!(
        call(
            &mut r,
            "a",
            "document.save",
            json!({"path":"demo.spritelab"})
        )
        .ok
    );
    assert!(
        !call(
            &mut r,
            "a",
            "document.save",
            json!({"path":"demo.spritelab"})
        )
        .ok
    );
    assert!(
        call(
            &mut r,
            "b",
            "document.open",
            json!({"path":"demo.spritelab"})
        )
        .ok
    );
    assert_eq!(
        r.engine.document("a").unwrap(),
        r.engine.document("b").unwrap()
    );
    assert!(
        call(
            &mut r,
            "a",
            "export.png",
            json!({"path":"demo.png","frame_id":2})
        )
        .ok
    );
    assert!(call(&mut r, "c", "import.png", json!({"path":"demo.png"})).ok);
    assert_eq!(
        r.engine.document("a").unwrap().render(2).unwrap(),
        r.engine.document("c").unwrap().render(2).unwrap()
    );
    call(
        &mut r,
        "a",
        "frame.add",
        json!({"source_frame_id":2,"duration_ms":125}),
    );
    let out = call(
        &mut r,
        "a",
        "export.spritesheet",
        json!({"path":"atlas.png","columns":2,"padding":1}),
    );
    assert!(out.ok);
    let meta = &out.result.unwrap()["metadata"];
    assert_eq!(meta["size"]["w"], 11);
    assert_eq!(meta["size"]["h"], 6);
    assert_eq!(meta["frames"][1]["frame"]["x"], 6);
    assert_eq!(meta["frames"][1]["duration_ms"], 125);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn path_policy_rejects_escape_and_external_symlinks() {
    let root = test_root("paths");
    let mut r = Runtime::new(&root).unwrap();
    call(
        &mut r,
        "a",
        "document.create",
        json!({"width":2,"height":2}),
    );
    for path in [
        "../escape.spritelab",
        "/tmp/escape.spritelab",
        "missing/file.spritelab",
    ] {
        assert!(!call(&mut r, "a", "document.save", json!({"path":path})).ok);
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(std::env::temp_dir(), root.join("escape")).unwrap();
        assert_eq!(
            call(
                &mut r,
                "a",
                "document.save",
                json!({"path":"escape/out.spritelab"})
            )
            .error
            .unwrap()
            .code,
            "path_denied"
        );
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn preview_matches_export_and_keeps_revision() {
    let root = test_root("preview");
    let mut r = Runtime::new(&root).unwrap();
    call(
        &mut r,
        "a",
        "document.create",
        json!({"width":3,"height":3}),
    );
    let preview = call(&mut r, "a", "document.preview", json!({"frame_id":2}));
    assert!(preview.ok);
    call(
        &mut r,
        "a",
        "export.png",
        json!({"path":"p.png","frame_id":2}),
    );
    assert_eq!(
        STANDARD
            .decode(preview.result.unwrap()["png_base64"].as_str().unwrap())
            .unwrap(),
        fs::read(root.join("p.png")).unwrap()
    );
    assert_eq!(r.engine.revision("a"), Some(0));
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn malformed_project_does_not_install() {
    let root = test_root("malformed");
    let mut r = Runtime::new(&root).unwrap();
    fs::write(root.join("bad.spritelab"), b"{\"format_version\":999}").unwrap();
    assert!(
        !call(
            &mut r,
            "a",
            "document.open",
            json!({"path":"bad.spritelab"})
        )
        .ok
    );
    assert!(r.engine.document("a").is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn shared_editor_context_is_validated_and_repairs_deleted_objects() {
    let root = std::env::temp_dir().join(format!("sl-view-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut r = Runtime::new(&root).unwrap();
    let mut run = |command: &str, args: Value| {
        r.execute(Request {
            request_id: None,
            document_id: Some("view".into()),
            expected_revision: None,
            command: command.into(),
            args,
        })
    };
    assert!(run("document.create", json!({"width":8,"height":8})).ok);
    assert!(
        run(
            "editor.configure",
            json!({"document_id":"view","tool":"ellipse","grid":true,"brush_size":3})
        )
        .ok
    );
    assert!(!run("editor.configure", json!({"brush_size":900})).ok);
    assert_eq!(
        run("editor.inspect", json!({})).result.unwrap()["view"]["brush_size"],
        3
    );
    run("frame.add", json!({"duration_ms":120}));
    run("editor.configure", json!({"frame_id":3}));
    run("frame.remove", json!({"frame_id":3}));
    assert_eq!(
        run("editor.inspect", json!({})).result.unwrap()["view"]["frame_id"],
        2
    );
    run("document.close", json!({}));
    assert!(run("editor.inspect", json!({})).result.unwrap()["view"]["document_id"].is_null());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn gif_export_retains_frame_count_timing_and_binary_transparency() {
    let root = std::env::temp_dir().join(format!("sl-gif-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut r = Runtime::new(&root).unwrap();
    let mut run = |command: &str, args: Value| {
        r.execute(Request {
            request_id: None,
            document_id: Some("gif".into()),
            expected_revision: None,
            command: command.into(),
            args,
        })
    };
    run("document.create", json!({"width":4,"height":4}));
    run(
        "canvas.stroke",
        json!({"layer_id":1,"frame_id":2,"points":[[0,0]],"color":[255,0,0,255]}),
    );
    run("frame.add", json!({"source_frame_id":2,"duration_ms":160}));
    assert!(run("export.gif", json!({"path":"test.gif","overwrite":true})).ok);
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = options
        .read_info(std::fs::File::open(root.join("test.gif")).unwrap())
        .unwrap();
    let f = reader.read_next_frame().unwrap().unwrap();
    assert_eq!(f.delay, 10);
    assert_eq!(&f.buffer[..4], &[255, 0, 0, 255]);
    assert_eq!(f.buffer[7], 0);
    let f = reader.read_next_frame().unwrap().unwrap();
    assert_eq!(f.delay, 16);
    assert!(reader.read_next_frame().unwrap().is_none());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn save_open_and_undo_share_dirty_state_with_every_client() {
    let root = std::env::temp_dir().join(format!("sl-dirty-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut r = Runtime::new(&root).unwrap();
    let mut call = |command: &str, args: Value| {
        r.execute(Request {
            request_id: None,
            document_id: Some("dirty".into()),
            expected_revision: None,
            command: command.into(),
            args,
        })
    };
    call("document.create", json!({"width":4,"height":4}));
    call("editor.configure", json!({"document_id":"dirty"}));
    assert_eq!(
        call("editor.inspect", json!({})).result.unwrap()["dirty"],
        true
    );
    call(
        "document.save",
        json!({"path":"project.spritelab","overwrite":true}),
    );
    let status = call("editor.inspect", json!({})).result.unwrap();
    assert_eq!(status["dirty"], false);
    assert_eq!(status["document_path"], "project.spritelab");
    call(
        "canvas.stroke",
        json!({"layer_id":1,"frame_id":2,"points":[[0,0]],"color":[1,2,3,255]}),
    );
    assert_eq!(
        call("editor.inspect", json!({})).result.unwrap()["dirty"],
        true
    );
    call("history.undo", json!({}));
    assert_eq!(
        call("editor.inspect", json!({})).result.unwrap()["dirty"],
        false
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn session_document_statuses_cover_inactive_work_and_color_alpha_is_shared() {
    let root = test_root("inactive");
    let mut r = Runtime::new(&root).unwrap();
    call(
        &mut r,
        "first",
        "document.create",
        json!({"width":2,"height":2}),
    );
    call(
        &mut r,
        "first",
        "document.save",
        json!({"path":"first.spritelab"}),
    );
    call(
        &mut r,
        "second",
        "document.create",
        json!({"width":2,"height":2}),
    );
    call(
        &mut r,
        "first",
        "editor.configure",
        json!({"document_id":"first","color":[1,2,3,89]}),
    );
    let view = call(&mut r, "first", "editor.inspect", json!({}))
        .result
        .unwrap();
    assert_eq!(view["dirty"], false);
    assert_eq!(view["documents"][1]["document_id"], "second");
    assert_eq!(view["documents"][1]["dirty"], true);
    assert_eq!(view["view"]["alpha"], 89);
    call(&mut r, "first", "editor.configure", json!({"alpha":143}));
    let view = call(&mut r, "first", "editor.inspect", json!({}))
        .result
        .unwrap();
    assert_eq!(view["view"]["color"], json!([1, 2, 3, 143]));
    call(&mut r, "second", "document.close", json!({}));
    let list = call(&mut r, "first", "document.list", json!({}))
        .result
        .unwrap();
    assert_eq!(list["documents"].as_array().unwrap().len(), 1);
    assert_eq!(list["documents"][0]["path"], "first.spritelab");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_survives_runtime_and_keeps_unsaved_baseline() {
    let root = test_root("recovery");
    let mut runtime = Runtime::new(&root).unwrap();
    assert!(
        call(
            &mut runtime,
            "a",
            "document.create",
            json!({"width":4,"height":4})
        )
        .ok
    );
    assert!(
        call(
            &mut runtime,
            "a",
            "document.save",
            json!({"path":"original.spritelab"})
        )
        .ok
    );
    let original = fs::read(root.join("original.spritelab")).unwrap();
    assert!(
        call(
            &mut runtime,
            "a",
            "canvas.fill",
            json!({"layer_id":1,"frame_id":2,"at":[0,0],"color":[20,40,60,255]})
        )
        .ok
    );
    let checkpoint = call(&mut runtime, "a", "recovery.checkpoint", json!({}));
    assert!(checkpoint.ok);
    let entry = checkpoint.result.unwrap()["entry"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        call(&mut runtime, "a", "recovery.checkpoint", json!({}))
            .result
            .unwrap()["changed"],
        false
    );
    assert_eq!(
        call(&mut runtime, "a", "editor.inspect", json!({}))
            .result
            .unwrap()["documents"][0]["dirty"],
        true
    );
    assert_eq!(fs::read(root.join("original.spritelab")).unwrap(), original);
    let expected = runtime.engine.document("a").unwrap().clone();
    drop(runtime);
    let mut reopened = Runtime::new(&root).unwrap();
    assert_eq!(
        call(&mut reopened, "b", "recovery.list", json!({}))
            .result
            .unwrap()["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        call(
            &mut reopened,
            "b",
            "recovery.restore",
            json!({"entry":entry})
        )
        .ok
    );
    assert_eq!(*reopened.engine.document("b").unwrap(), expected);
    assert!(
        !call(
            &mut reopened,
            "b",
            "recovery.restore",
            json!({"entry":entry})
        )
        .ok
    );
    assert!(
        !call(
            &mut reopened,
            "b",
            "recovery.restore",
            json!({"entry":"../original.spritelab"})
        )
        .ok
    );
    assert!(
        call(
            &mut reopened,
            "b",
            "recovery.delete",
            json!({"entry":entry})
        )
        .ok
    );
    assert_eq!(*reopened.engine.document("b").unwrap(), expected);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn layer_preview_is_isolated_read_only_and_ignores_visibility() {
    let root = test_root("layer-preview");
    let mut r = Runtime::new(&root).unwrap();
    assert!(
        call(
            &mut r,
            "a",
            "document.create",
            json!({"width":2,"height":2})
        )
        .ok
    );
    assert!(
        call(
            &mut r,
            "a",
            "canvas.stroke",
            json!({"layer_id":1,"frame_id":2,"points":[[0,0]],"color":[100,200,50,128]})
        )
        .ok
    );
    assert!(
        call(
            &mut r,
            "a",
            "layer.update",
            json!({"layer_id":1,"visible":false,"opacity":0})
        )
        .ok
    );
    let before = r.engine.document("a").unwrap().clone();
    let revision = r.engine.revision("a");
    let preview = call(
        &mut r,
        "a",
        "document.preview",
        json!({"frame_id":2,"layer_id":1}),
    );
    assert!(preview.ok);
    let encoded = STANDARD
        .decode(preview.result.unwrap()["png_base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        encoded,
        encode_png(
            2,
            2,
            &[100, 200, 50, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        )
        .unwrap()
    );
    assert!(
        !call(
            &mut r,
            "a",
            "document.preview",
            json!({"frame_id":2,"layer_id":999})
        )
        .ok
    );
    assert!(
        !call(
            &mut r,
            "a",
            "document.preview",
            json!({"frame_id":999,"layer_id":1})
        )
        .ok
    );
    assert_eq!(r.engine.document("a").unwrap(), &before);
    assert_eq!(r.engine.revision("a"), revision);
    assert!(call(&mut r, "a", "frame.add", json!({"duration_ms":100})).ok);
    let empty = call(
        &mut r,
        "a",
        "document.preview",
        json!({"frame_id":3,"layer_id":1}),
    );
    assert!(empty.ok);
    assert_eq!(
        STANDARD
            .decode(empty.result.unwrap()["png_base64"].as_str().unwrap())
            .unwrap(),
        encode_png(2, 2, &[0; 16]).unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}
