#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use assetforge_command::{Request, Response};
use serde_json::{json, Value};
use spritelab_cli::session::{SessionServer, SharedRuntime};
use spritelab_cli::Runtime;
use std::{fs, path::PathBuf};
struct Host {
    runtime: SharedRuntime,
    session: SessionServer,
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
    tauri::Builder::default()
        .manage(Host {
            runtime: session.runtime.clone(),
            session,
        })
        .invoke_handler(tauri::generate_handler![execute, workspace])
        .run(tauri::generate_context!())
        .expect("Cannot start SpriteLab");
}
