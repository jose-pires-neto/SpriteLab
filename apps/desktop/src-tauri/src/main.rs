#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use assetforge_command::{Request, Response};
use serde_json::{json, Value};
use spritelab_cli::session::{SessionServer, SharedRuntime};
use spritelab_cli::Runtime;
use std::{fs, path::PathBuf};
struct Host {
    runtime: SharedRuntime,
    session: SessionServer,
    recovery_error: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}
#[tauri::command]
fn execute(request: Request, state: tauri::State<Host>) -> Result<Response, String> {
    Ok(state
        .runtime
        .lock()
        .map_err(|e| e.to_string())?
        .execute(request))
}
#[tauri::command]
fn workspace(state: tauri::State<Host>) -> Result<Value, String> {
    let mut runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    let response = runtime.execute(Request {
        request_id: None,
        document_id: None,
        expected_revision: None,
        command: "workspace.inspect".into(),
        args: json!({}),
    });
    if !response.ok {
        return Err(response.error.unwrap().message);
    }
    let mut result = response.result.unwrap();
    result["socket"] = json!(state.session.socket);
    result["discovery"] = json!(state.session.discovery);
    result["recovery_error"] = json!(*state.recovery_error.lock().map_err(|e| e.to_string())?);
    Ok(result)
}
fn main() {
    let root = std::env::args()
        .skip_while(|a| a != "--workspace")
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("Documents/ChatGPT/SpriteLab/assets")
        });
    fs::create_dir_all(&root).expect("Cannot create SpriteLab workspace");
    let example = root.join("exemplo-cogumelo.spritelab");
    if !example.exists() {
        fs::write(
            example,
            include_str!("../../../../demo/mushroom-000.spritelab"),
        )
        .expect("Cannot install example");
    }
    let runtime = Runtime::new(&root).expect("Cannot open workspace");
    let session = SessionServer::start(runtime, &root).expect("Cannot start local session");
    let recovery_error = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
    let worker_error = recovery_error.clone();
    let worker_runtime = session.runtime.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(15));
        let mut runtime = match worker_runtime.lock() {
            Ok(r) => r,
            Err(_) => break,
        };
        let listing = runtime.execute(Request {
            request_id: None,
            document_id: None,
            expected_revision: None,
            command: "document.list".into(),
            args: json!({}),
        });
        let mut error = None;
        if let Some(documents) = listing
            .result
            .and_then(|v| v["documents"].as_array().cloned())
        {
            for document in documents.iter().filter(|d| d["dirty"] == true) {
                let response = runtime.execute(Request {
                    request_id: None,
                    document_id: document["document_id"].as_str().map(String::from),
                    expected_revision: None,
                    command: "recovery.checkpoint".into(),
                    args: json!({}),
                });
                if let Some(e) = response.error {
                    error = Some(e.message);
                    break;
                }
            }
        } else {
            error = Some("Cannot inspect documents for recovery".into());
        }
        if let Ok(mut value) = worker_error.lock() {
            *value = error;
        }
    });
    tauri::Builder::default()
        .manage(Host {
            runtime: session.runtime.clone(),
            session,
            recovery_error,
        })
        .invoke_handler(tauri::generate_handler![execute, workspace])
        .run(tauri::generate_context!())
        .expect("Cannot start SpriteLab");
}
