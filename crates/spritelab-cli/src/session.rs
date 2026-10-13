//! Private local IPC: GUI and attached clients serialize calls to the same Runtime.
use crate::Runtime;
use assetforge_command::{CommandError, Request, Response};
use std::{
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
pub type SharedRuntime = Arc<Mutex<Runtime>>;
pub fn read_bounded_line(input: &mut impl BufRead) -> io::Result<Option<Option<Vec<u8>>>> {
    let mut out = Vec::new();
    let mut oversized = false;
    let mut seen = false;
    loop {
        let buf = input.fill_buf()?;
        if buf.is_empty() {
            return Ok(if seen {
                Some(if oversized { None } else { Some(out) })
            } else {
                None
            });
        }
        seen = true;
        let end = buf.iter().position(|b| *b == b'\n');
        let count = end.map(|i| i + 1).unwrap_or(buf.len());
        if !oversized {
            if out.len() + count > crate::MAX_INPUT {
                oversized = true;
                out.clear()
            } else {
                out.extend_from_slice(&buf[..count])
            }
        }
        input.consume(count);
        if end.is_some() {
            return Ok(Some(if oversized { None } else { Some(out) }));
        }
    }
}
pub fn process_line(runtime: &SharedRuntime, line: Option<Vec<u8>>) -> Response {
    match line {
        None => Response::from_result(
            None,
            None,
            Err(CommandError::new("resource_limit", "Input exceeds 16 MiB")),
        ),
        Some(bytes) => match serde_json::from_slice::<Request>(&bytes) {
            Ok(req) => match runtime.lock() {
                Ok(mut r) => r.execute(req),
                Err(_) => Response::from_result(
                    req.request_id,
                    None,
                    Err(CommandError::new("session_failed", "Runtime lock poisoned")),
                ),
            },
            Err(e) => Response::from_result(
                None,
                None,
                Err(CommandError::new("invalid_request", e.to_string())),
            ),
        },
    }
}
#[cfg(unix)]
mod local {
    use super::*;
    use std::{
        fs,
        os::unix::{
            fs::PermissionsExt,
            net::{UnixListener, UnixStream},
        },
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
        thread,
        time::Duration,
    };
    pub struct SessionServer {
        pub runtime: SharedRuntime,
        pub socket: PathBuf,
        pub discovery: PathBuf,
        stop: Arc<AtomicBool>,
    }
    impl SessionServer {
        pub fn start(runtime: Runtime, workspace: &Path) -> io::Result<Self> {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let socket =
                std::env::temp_dir().join(format!("sl-{}-{stamp:x}.sock", std::process::id()));
            let listener = UnixListener::bind(&socket)?;
            fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
            listener.set_nonblocking(true)?;
            let discovery =
                workspace.join(format!(".spritelab-session-{}.json", std::process::id()));
            let runtime = Arc::new(Mutex::new(runtime));
            let stop = Arc::new(AtomicBool::new(false));
            let bytes=serde_json::to_vec(&serde_json::json!({"protocol_version":1,"pid":std::process::id(),"socket":socket,"workspace":workspace.canonicalize()?})).map_err(io::Error::other)?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&discovery)?;
            fs::set_permissions(&discovery, fs::Permissions::from_mode(0o600))?;
            file.write_all(&bytes)?;
            let rt = runtime.clone();
            let stopped = stop.clone();
            let clients = Arc::new(AtomicUsize::new(0));
            thread::spawn(move || {
                while !stopped.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            if clients.fetch_add(1, Ordering::SeqCst) >= 16 {
                                clients.fetch_sub(1, Ordering::SeqCst);
                                continue;
                            }
                            let rt = rt.clone();
                            let clients = clients.clone();
                            thread::spawn(move || {
                                if let Err(e) = serve(stream, &rt) {
                                    eprintln!("Session client disconnected: {e}");
                                }
                                clients.fetch_sub(1, Ordering::SeqCst);
                            });
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(20))
                        }
                        Err(_) => break,
                    }
                }
            });
            Ok(Self {
                runtime,
                socket,
                discovery,
                stop,
            })
        }
    }
    impl Drop for SessionServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            let _ = fs::remove_file(&self.socket);
            let _ = fs::remove_file(&self.discovery);
        }
    }
    fn serve(mut stream: UnixStream, runtime: &SharedRuntime) -> io::Result<()> {
        stream.set_nonblocking(false)?;
        // A connected MCP may remain idle while the artist works.
        stream.set_read_timeout(None)?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        let mut reader = BufReader::new(stream.try_clone()?);
        while let Some(line) = read_bounded_line(&mut reader)? {
            let response = process_line(runtime, line);
            serde_json::to_writer(&mut stream, &response).map_err(io::Error::other)?;
            stream.write_all(b"\n")?;
            stream.flush()?;
        }
        Ok(())
    }
    pub fn attach(
        socket: &Path,
        input: &mut impl BufRead,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let mut stream = UnixStream::connect(socket)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        let mut reader = BufReader::new(stream.try_clone()?);
        while let Some(line) = read_bounded_line(input)? {
            let bytes = line.ok_or_else(|| io::Error::other("Input exceeds 16 MiB"))?;
            stream.write_all(&bytes)?;
            if !bytes.ends_with(b"\n") {
                stream.write_all(b"\n")?
            }
            stream.flush()?;
            let response = read_bounded_line(&mut reader)?.flatten().ok_or_else(|| {
                io::Error::other("Session closed or oversized response; outcome unknown")
            })?;
            output.write_all(&response)?;
            output.flush()?;
        }
        Ok(())
    }
}
#[cfg(unix)]
pub use local::*;

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use serde_json::json;
    use std::os::unix::net::UnixStream;
    #[test]
    fn socket_clients_share_gui_runtime_and_only_one_stale_edit_commits() {
        let root = std::env::temp_dir().join(format!("sl-host-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let server = SessionServer::start(Runtime::new(&root).unwrap(), &root).unwrap();
        let create = Request {
            request_id: None,
            document_id: Some("live".into()),
            expected_revision: None,
            command: "document.create".into(),
            args: json!({"width":8,"height":8}),
        };
        assert!(server.runtime.lock().unwrap().execute(create).ok);
        let handles:Vec<_>=(0..2).map(|i|{let path=server.socket.clone();std::thread::spawn(move||{
   let mut stream=UnixStream::connect(path).unwrap();std::thread::sleep(std::time::Duration::from_millis(60));let request=json!({"request_id":format!("client-{i}"),"document_id":"live","expected_revision":0,"command":"layer.create","args":{"name":format!("Layer {i}")}});
   writeln!(stream,"{request}").unwrap();let mut reader=BufReader::new(stream);let bytes=read_bounded_line(&mut reader).unwrap().unwrap().unwrap();serde_json::from_slice::<Response>(&bytes).unwrap()
  })}).collect();
        let responses: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(responses.iter().filter(|r| r.ok).count(), 1);
        assert_eq!(
            responses
                .iter()
                .find_map(|r| r.error.as_ref())
                .unwrap()
                .code,
            "revision_conflict"
        );
        assert_eq!(
            server
                .runtime
                .lock()
                .unwrap()
                .engine
                .document("live")
                .unwrap()
                .layers
                .len(),
            2
        );
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&server.socket)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let socket = server.socket.clone();
        drop(server);
        assert!(!socket.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn line_reader_recovers_after_oversized_input() {
        let mut bytes = vec![b'x'; crate::MAX_INPUT + 1];
        bytes.extend_from_slice(b"\n{}\n");
        let mut reader = std::io::Cursor::new(bytes);
        assert!(read_bounded_line(&mut reader).unwrap().unwrap().is_none());
        assert_eq!(
            read_bounded_line(&mut reader).unwrap().unwrap().unwrap(),
            b"{}\n"
        );
    }
}
