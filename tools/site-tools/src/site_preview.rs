//! Small development-only static HTTP server for the generated dist directory.
//! It deliberately does not implement TLS, keep-alive, or a production server.
use std::{
    env, fs,
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

const MAX_HEADERS: u64 = 8192;

#[derive(Debug, PartialEq)]
enum Method {
    Get,
    Head,
    Other,
}

struct Request {
    method: Method,
    target: String,
}

fn read_request(reader: impl BufRead) -> io::Result<Request> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, "invalid HTTP request");
    let mut reader = reader.take(MAX_HEADERS + 1);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let first = first.strip_suffix("\r\n").ok_or_else(invalid)?;
    let mut parts = first.split(' ');
    let method = match parts.next() {
        Some("GET") => Method::Get,
        Some("HEAD") => Method::Head,
        Some(s) if !s.is_empty() => Method::Other,
        _ => return Err(invalid()),
    };
    let target = parts.next().ok_or_else(invalid)?;
    if !target.starts_with('/')
        || target.starts_with("//")
        || target.bytes().any(|b| b <= 0x20 || b == 0x7f)
        || !matches!(parts.next(), Some("HTTP/1.0" | "HTTP/1.1"))
        || parts.next().is_some()
    {
        return Err(invalid());
    }
    let mut line = Vec::new();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 || reader.limit() == 0 {
            return Err(invalid());
        }
        if line == b"\r\n" {
            break;
        }
        if !line.ends_with(b"\r\n") {
            return Err(invalid());
        }
    }
    Ok(Request {
        method,
        target: target.to_owned(),
    })
}

fn base_path(raw: &str) -> Option<String> {
    if !raw.starts_with('/') || raw.starts_with("//") {
        return None;
    }
    let tail = raw.trim_start_matches('/').trim_end_matches('/');
    if raw == "/" {
        return Some("/".into());
    }
    if tail.is_empty() || raw.ends_with("//") {
        return None;
    }
    for segment in tail.split('/') {
        if segment.is_empty()
            || matches!(segment, "." | "..")
            || !segment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        {
            return None;
        }
    }
    Some(format!("/{tail}/"))
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn decode_segment(segment: &str) -> Option<String> {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            out.push(hex(*bytes.get(i + 1)?)? * 16 + hex(*bytes.get(i + 2)?)?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let decoded = String::from_utf8(out).ok()?;
    if decoded.is_empty()
        || matches!(decoded.as_str(), "." | "..")
        || decoded
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '?' | '#'))
        || decoded.ends_with('.')
        || decoded.ends_with(' ')
    {
        return None;
    }
    Some(decoded)
}

fn contained_file(root: &Path, path: &Path) -> Option<PathBuf> {
    let canonical = fs::canonicalize(root.join(path)).ok()?;
    (canonical.starts_with(root) && canonical.is_file()).then_some(canonical)
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

struct Response {
    status: &'static str,
    content_type: &'static str,
    extra: String,
    body: Vec<u8>,
}

fn plain(status: &'static str, body: &str) -> Response {
    Response {
        status,
        content_type: "text/plain; charset=utf-8",
        extra: String::new(),
        body: body.as_bytes().to_vec(),
    }
}

fn not_found(root: &Path) -> Response {
    match contained_file(root, Path::new("404.html")).and_then(|p| fs::read(p).ok()) {
        Some(body) => Response {
            status: "404 Not Found",
            content_type: "text/html; charset=utf-8",
            extra: String::new(),
            body,
        },
        None => plain("404 Not Found", "找不到頁面"),
    }
}

fn redirect(location: &str) -> Response {
    let mut r = plain("308 Permanent Redirect", "Redirect");
    r.extra = format!("Location: {location}\r\n");
    r
}

fn respond(root: &Path, base: &str, req: &Request) -> Response {
    if req.method == Method::Other {
        let mut r = plain("405 Method Not Allowed", "Only GET and HEAD are supported");
        r.extra = "Allow: GET, HEAD\r\n".into();
        return r;
    }
    let raw = req.target.split('?').next().unwrap_or("");
    if base != "/" && raw == base.trim_end_matches('/') {
        return redirect(base);
    }
    let Some(relative) = raw.strip_prefix(base) else {
        return not_found(root);
    };
    let relative = relative.strip_suffix('/').unwrap_or(relative);
    let mut path = PathBuf::new();
    if !relative.is_empty() {
        for segment in relative.split('/') {
            let Some(segment) = decode_segment(segment) else {
                return not_found(root);
            };
            path.push(segment);
        }
    }
    // A directory is served only through its index.html, never listed.
    let index = path.join("index.html");
    let file = if let Some(file) = contained_file(root, &index) {
        if !raw.ends_with('/') {
            return redirect(&format!("{raw}/"));
        }
        file
    } else if raw.ends_with('/') {
        return not_found(root);
    } else if let Some(file) = contained_file(root, &path) {
        file
    } else {
        return not_found(root);
    };
    match fs::read(&file) {
        Ok(body) => Response {
            status: "200 OK",
            content_type: mime(&file),
            extra: String::new(),
            body,
        },
        Err(_) => not_found(root),
    }
}

fn write_response(mut writer: impl Write, response: Response, head: bool) -> io::Result<()> {
    write!(
        writer,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n{}\r\n",
        response.status,
        response.content_type,
        response.body.len(),
        response.extra
    )?;
    if !head {
        writer.write_all(&response.body)?;
    }
    writer.flush()
}

fn handle(mut stream: TcpStream, root: &Path, base: &str) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    match read_request(BufReader::new(&mut stream)) {
        Ok(request) => write_response(
            &mut stream,
            respond(root, base, &request),
            request.method == Method::Head,
        ),
        Err(_) => write_response(
            &mut stream,
            plain("400 Bad Request", "Invalid HTTP request"),
            false,
        ),
    }
}

pub fn cli() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args_os().nth(1).ok_or("Usage: site-preview <dist>")?;
    serve(Path::new(&path))
}

pub fn serve(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let root = fs::canonicalize(path)?;
    if !root.join("index.html").is_file() {
        return Err("找不到建置產物 index.html".into());
    }
    let base =
        base_path(&env::var("BASE_PATH").unwrap_or_else(|_| "/".into())).ok_or("BASE_PATH 無效")?;
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "4173".into()).parse()?;
    let listener = TcpListener::bind((host.as_str(), port))?;
    let address = listener.local_addr()?;
    if !address.ip().is_loopback() {
        eprintln!("注意：預覽伺服器沒有驗證或 TLS，僅可在可信任的區域網路使用。");
    }
    println!("Rust 學習工坊 http://{address}{base}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle(stream, &root, &base) {
                    eprintln!("預覽連線：{err}");
                }
            }
            Err(err) => eprintln!("接受連線失敗：{err}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_base_and_decoded_segments() {
        assert_eq!(
            base_path("/-rust-learning"),
            Some("/-rust-learning/".into())
        );
        assert_eq!(base_path("/"), Some("/".into()));
        for invalid in ["//x", "/x//y", "/../", "/x?y", "/x//"] {
            assert_eq!(base_path(invalid), None);
        }
        for invalid in [
            "..", "%2e%2e", "%2fetc", "%5Cfile", "%00", "%", "C%3a", "name.",
        ] {
            assert_eq!(decode_segment(invalid), None);
        }
        assert_eq!(decode_segment("%E6%95%99%E6%9D%90"), Some("教材".into()));
    }

    #[test]
    fn parses_bounded_requests_and_head() {
        let req =
            read_request(&b"HEAD /chapters/ownership/?x=1 HTTP/1.1\r\nHost: localhost\r\n\r\n"[..])
                .unwrap();
        assert_eq!(req.method, Method::Head);
        assert_eq!(req.target, "/chapters/ownership/?x=1");
        assert!(read_request(&b"GET //other/ HTTP/1.1\r\n\r\n"[..]).is_err());
        assert!(read_request(&b"GET / HTTP/1.1\r\n"[..]).is_err());
        let huge = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "x".repeat(9000));
        assert!(read_request(huge.as_bytes()).is_err());
        let mut output = Vec::new();
        write_response(&mut output, plain("200 OK", "abc"), true).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Content-Length: 3\r\n"));
        assert!(output.ends_with("\r\n\r\n"));
    }

    #[test]
    fn serves_only_dist_files_with_correct_status_and_mime() {
        let temp = env::temp_dir().join(format!("rust-preview-test-{}", std::process::id()));
        let root = temp.join("dist");
        fs::create_dir_all(root.join("chapters/ownership")).unwrap();
        fs::write(root.join("index.html"), "home").unwrap();
        fs::write(root.join("chapters/ownership/index.html"), "lesson").unwrap();
        fs::write(root.join("style.css"), "body{}").unwrap();
        fs::write(root.join("404.html"), "missing").unwrap();
        fs::write(temp.join("secret.txt"), "outside").unwrap();
        let root = fs::canonicalize(root).unwrap();
        let get = |target: &str| {
            respond(
                &root,
                "/course/",
                &Request {
                    method: Method::Get,
                    target: target.into(),
                },
            )
        };
        assert_eq!(get("/course/").body, b"home");
        assert_eq!(
            get("/course/chapters/ownership").status,
            "308 Permanent Redirect"
        );
        assert_eq!(get("/course/chapters/ownership/?v=1").body, b"lesson");
        assert_eq!(
            get("/course/style.css").content_type,
            "text/css; charset=utf-8"
        );
        assert_eq!(get("/course/%2e%2e/secret.txt").status, "404 Not Found");
        assert_eq!(get("/course/not-here").body, b"missing");
        assert_eq!(get("/outside/").status, "404 Not Found");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(temp.join("secret.txt"), root.join("escape.txt")).unwrap();
            assert_eq!(get("/course/escape.txt").status, "404 Not Found");
        }
        fs::remove_dir_all(temp).unwrap();
    }
}
