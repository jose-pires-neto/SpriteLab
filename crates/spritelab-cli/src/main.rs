use assetforge_command::{CommandError, Request, Response};
use spritelab_cli::{session::read_bounded_line, Runtime};
use std::{
    io::{self, Write},
    path::Path,
};
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|s| s == "--help") {
        println!("SpriteLab 0.5.0 — headless command engine\nUsage: spritelab --workspace DIRECTORY [--catalog | --host]\n       spritelab --attach SOCKET\nJSONL requests on stdin, structured responses on stdout.\nHeadless owns an isolated session; --attach shares an existing host. Paths are workspace-relative.");
        return Ok(());
    }
    #[cfg(unix)]
    if let Some(socket) = args.windows(2).find(|a| a[0] == "--attach").map(|a| &a[1]) {
        spritelab_cli::session::attach(
            Path::new(socket),
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
        )?;
        return Ok(());
    }
    let root = args
        .windows(2)
        .find(|a| a[0] == "--workspace")
        .map(|a| a[1].as_str())
        .ok_or("--workspace DIRECTORY is required")?;
    let mut runtime = Runtime::new(Path::new(root)).map_err(|e| e.message)?;
    if args.iter().any(|s| s == "--catalog") {
        println!(
            "{}",
            serde_json::to_string_pretty(&spritelab_cli::catalog())?
        );
        return Ok(());
    }
    #[cfg(unix)]
    if args.iter().any(|a| a == "--host") {
        let server = spritelab_cli::session::SessionServer::start(runtime, Path::new(root))?;
        println!(
            "{}",
            serde_json::json!({"socket":server.socket,"discovery":server.discovery})
        );
        loop {
            std::thread::park();
        }
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    while let Some(line) = read_bounded_line(&mut input)? {
        let response = match line {
            None => Response::from_result(
                None,
                None,
                Err(CommandError::new(
                    "resource_limit",
                    "Input line exceeds 16 MiB",
                )),
            ),
            Some(bytes) => match serde_json::from_slice::<Request>(&bytes) {
                Ok(req) => runtime.execute(req),
                Err(e) => Response::from_result(
                    None,
                    None,
                    Err(CommandError::new("invalid_request", e.to_string())),
                ),
            },
        };
        serde_json::to_writer(&mut output, &response)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
