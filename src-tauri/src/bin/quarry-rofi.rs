use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("quarry.sock")
}

fn get_response_socket_path() -> PathBuf {
    std::env::temp_dir().join(format!("quarry-rofi-{}.sock", std::process::id()))
}

#[derive(serde::Serialize)]
struct RofiItem {
    name: String,
    description: Option<String>,
    shell: Option<String>,
}

/// Formats:
/// - "name" -> name only
/// - "name:description" -> name + description
/// - "name:description:shell" -> name + description + shell (splitn(3, ':'))
fn parse_item(s: &str) -> RofiItem {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    match parts.as_slice() {
        [name] => RofiItem {
            name: name.trim().to_string(),
            description: None,
            shell: None,
        },
        [name, desc] => RofiItem {
            name: name.trim().to_string(),
            description: Some(desc.trim().to_string()),
            shell: None,
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
        eprintln!("  quarry-rofi \"Name\" \"Name 2:Description\" \"Name 3:Desc:shell cmd\"");
        eprintln!("  echo -e \"Item 1\\nItem 2\" | quarry-rofi");
        std::process::exit(1);
    }

    let response_path = get_response_socket_path();
    let _ = std::fs::remove_file(&response_path);

    let listener = match UnixListener::bind(&response_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("quarry-rofi: failed to create response socket: {}", e);
            std::process::exit(1);
        }
    };

    let quarry_socket = get_socket_path();
    let mut stream = match UnixStream::connect(&quarry_socket) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("quarry-rofi: failed to connect to quarry ({})", e);
            eprintln!(" yo is quarry even running?");
            let _ = std::fs::remove_file(&response_path);
            std::process::exit(1);
        }
    };

    let response_socket_str = response_path.to_string_lossy().to_string();
    let cmd = serde_json::json!({
        "ShowRofi": {
            "items": items,
            "response_socket": response_socket_str,
        }
    });

    if let Err(e) = stream.write_all(format!("{}\n", cmd).as_bytes()) {
        eprintln!("quarry-rofi: failed to send command: {}", e);
        let _ = std::fs::remove_file(&response_path);
        std::process::exit(1);
    }

    let mut ack_reader = BufReader::new(stream);
    let mut _ack = String::new();
    let _ = ack_reader.read_line(&mut _ack);

    match listener.accept() {
        Ok((conn, _)) => {
            let mut reader = BufReader::new(conn);
            let mut result = String::new();
            let _ = reader.read_line(&mut result);
            let result = result.trim_end_matches('\n').trim_end_matches('\r');
            if !result.is_empty() {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("quarry-rofi: error accepting response: {}", e);
            let _ = std::fs::remove_file(&response_path);
            std::process::exit(1);
        }
    }

    let _ = std::fs::remove_file(&response_path);
}
