use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use uuid::Uuid;

#[test]
fn port_only_startup_generates_and_reports_configuration_sources() {
    let root = std::env::temp_dir().join(format!("ter-port-only-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).expect("create isolated runtime directory");
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve a test port");
    let port = listener.local_addr().expect("local address").port();
    drop(listener);

    let mut child = Command::new(env!("CARGO_BIN_EXE_telemetry-export-receipts"))
        .current_dir(&root)
        .env_clear()
        .env("PORT", port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start server with only PORT");

    let response = (0..100).find_map(|_| {
        let mut stream = match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => stream,
            Err(_) => {
                thread::sleep(Duration::from_millis(20));
                return None;
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("set read timeout");
        stream
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .expect("write health request");
        let mut body = String::new();
        stream
            .read_to_string(&mut body)
            .expect("read health response");
        Some(body)
    });

    child.kill().expect("stop isolated server");
    let output = child.wait_with_output().expect("collect server output");
    let logs = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let response = response.unwrap_or_else(|| panic!("server did not become ready; logs: {logs}"));
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains(r#"{"build_sha":"development","status":"ok"}"#));
    assert!(logs.contains("configuration sources"));
    assert!(logs.contains(r#""database":"default durable path""#));
    assert!(logs.contains(r#""signing_key":"generated""#));
    assert!(logs.contains(r#""admin_token":"generated""#));
    assert!(logs.contains(r#""upstream":"unset""#));

    for name in ["receipt-signing.key", "admin-access.key"] {
        let path = root.join("data").join(name);
        assert!(path.is_file(), "{} was not generated", path.display());
        let mode = fs::metadata(&path)
            .expect("secret metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "{} must be owner-only", path.display());
    }

    // The UUID-scoped directory contains only this test process's generated state.
    fs::remove_dir_all(&root).expect("remove isolated runtime directory");
}
