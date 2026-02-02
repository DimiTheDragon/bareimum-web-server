use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs;
use std::path::Path;
use std::thread;

const ROOT_DIR: &str = ".";
const STATFILES_SUBDIR: &str = "stat_files";

fn list_files(dir: &Path) -> Vec<String> {
  let mut files = Vec::new();

  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_file() {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
          files.push(stem.to_string());
        }
      }
    }
  }

  files
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("js")   => "text/javascript",
        Some("css")  => "text/css",
        Some("json") => "application/json",
        Some("png")  => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn handle_client(mut stream: TcpStream) {
  let statfiles_dir = Path::new(ROOT_DIR).join(STATFILES_SUBDIR);
  println!("La directory: {}", statfiles_dir.display());
  let mut buffer = [0; 1024];
  let bytes_read = match stream.read(&mut buffer) {
    Ok(n) => n,
    Err(_) => return,
  };
  let request = String::from_utf8_lossy(&buffer[..bytes_read]);
  let mut lines = request.lines();
  let first_line = lines.next().unwrap_or("");
  let mut parts = first_line.split_whitespace();
  let method = parts.next().unwrap_or("");
  let path = parts.next().unwrap_or("/");

  // Clean path
  let path = &path[1..]; // remove leading '/'
  let path = if path.is_empty() { "index.html" } else { path };
  let fs_path = Path::new(ROOT_DIR).join(path);

  match method {
    "GET" => {
      if path == "game-list" {
      let files = list_files(&statfiles_dir);
      let json = format!(
          "[{}]",
          files
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(",")
        );

        let _ = stream.write_all(format!(
          "HTTP/1.1 200 OK\r\n\
          Content-Type: application/json\r\n\
          Content-Length: {}\r\n\
          \r\n{}",
          json.len(),
          json
        ).as_bytes());

        return;
      }
    
      if fs_path.exists() && fs_path.is_file() {
        if let Ok(contents) = fs::read(&fs_path) {
          let content_type = mime_type(&fs_path);

          let _ = stream.write_all(format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: {}\r\n\
            Content-Length: {}\r\n\
            \r\n",
            content_type,
            contents.len()
          ).as_bytes());
          
          let _ = stream.write_all(&contents);
        }
      } else {
        let _ = stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\nFile not found");
      }
    }
    "POST" => {
      // parse the requested file name from query or body
      let file_name_raw = path.trim_start_matches('/');
      let file_name: String = file_name_raw
      .chars()
      .map(|c| {
        if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
          c
        } else {
          '_'
        }
      })
      .collect();
      eprintln!("This is the file to create: {}", file_name);
      let fs_path = Path::new(&statfiles_dir).join(file_name);
      
      // create an empty file
      match fs::File::create(&fs_path) {
        Ok(_) => {
          let _ = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nCreated");
        } 
        Err(e) => {
          eprintln!("Failed to create file {:?}: {}", fs_path, e);
          let _ = stream.write_all(b"HTTP/1.1 500 INTERNAL ERROR\r\n\r\nFailed");
        }
      }
    }
    _ => {
      let _ = stream.write_all(b"HTTP/1.1 405 METHOD NOT ALLOWED\r\n\r\n");
    }
  }
  println!("{} {}", method, path);
}

fn main() {
  let listener = TcpListener::bind("127.0.0.1:24375").expect("Failed to bind port 24375");
  println!("Serving HTTP on http://127.0.0.1:24375/");
  for stream in listener.incoming() {
    if let Ok(stream) = stream {
      thread::spawn(|| handle_client(stream));
    }
  }
}
