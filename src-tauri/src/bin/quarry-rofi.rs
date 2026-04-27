use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("quarry.sock")
}

#[derive(serde::Serialize)]
struct RofiItem {
    name: String,
    description: Option<String>,
    shell: Option<String>,
}

/// Parse a single item string.
/// Formats:
///   "Name"                     → name only (selection copies name)
///   "Name:shell command"       → name + shell command
///   "Name:Description:shell"   → name + description + shell (splitn(3, ':'))
fn parse_item(s: &str) -> RofiItem {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    match parts.as_slice() {
        [name] => RofiItem {
            name: name.trim().to_string(),
            description: None,
            shell: None,
        },
        [name, shell] => RofiItem {
            name: name.trim().to_string(),
            description: None,
            shell: Some(shell.trim().to_string()),
        },
        [name, desc, shell] => RofiItem {
            name: name.trim().to_string(),
            description: Some(desc.trim().to_string()),
            shell: Some(shell.trim().to_string()),
        },
        _ => unreachable!(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let items: Vec<RofiItem> = if !args.is_empty() {
        args.iter().map(|a| parse_item(a)).collect()
    } else {
        // Read newline-separated items from stdin
        let stdin = std::io::stdin();
        stdin
            .lock()
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .map(|l| parse_item(&l))
            .collect()
    };

    if items.is_empty() {
        eprintln!("quarry-rofi: no items provided");
        eprintln!("Usage:");
        eprintln!("  quarry-rofi \"Name:shell command\" \"Name 2:Description:shell command\"");
        eprintln!("  echo -e \"Name:shell\\nName 2:shell\" | quarry-rofi");
        std::process::exit(1);
    }

    let socket_path = get_socket_path();
    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("quarry-rofi: failed to connect to quarry ({})", e);
            eprintln!("Is quarry running?");
            std::process::exit(1);
        }
    };

    let cmd = serde_json::json!({ "ShowRofi": { "items": items } });

    if let Err(e) = stream.write_all(format!("{}\n", cmd).as_bytes()) {
        eprintln!("quarry-rofi: failed to send command: {}", e);
        std::process::exit(1);
    }

    // Wait for acknowledgement
    let mut reader = BufReader::new(stream);
    let mut _response = String::new();
    let _ = reader.read_line(&mut _response);
}
