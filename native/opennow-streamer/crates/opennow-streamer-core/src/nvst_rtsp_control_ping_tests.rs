use super::*;
use std::net::TcpListener;

fn socket_pair() -> (RtspClient, WebSocket<TcpStream>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(6)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        tungstenite::accept(stream).unwrap()
    });
    let stream = TcpStream::connect(address).unwrap();
    stream.set_nodelay(true).unwrap();
    let (socket, _) = tungstenite::client(
        format!("ws://{address}/rtsp"),
        MaybeTlsStream::Plain(stream),
    )
    .unwrap();
    (
        RtspClient {
            socket,
            cseq: 7,
            buffer: String::new(),
        },
        server.join().unwrap(),
    )
}

fn active_session(client: RtspClient, ping: &NvstControlPing) -> ActiveNvstRtspSession {
    ActiveNvstRtspSession::spawn(client, ping.clone()).unwrap()
}

fn server_ping(server: &mut WebSocket<TcpStream>) {
    server.send(Message::Ping(vec![1, 2, 3].into())).unwrap();
    assert_eq!(server.read().unwrap(), Message::Pong(vec![1, 2, 3].into()));
}

fn client_ping(server: &mut WebSocket<TcpStream>) -> Vec<u8> {
    let message = server.read().unwrap();
    let Message::Ping(bytes) = message else {
        panic!("expected WebSocket ping, received {message:?}");
    };
    assert_eq!(bytes.len(), 8);
    bytes.to_vec()
}

fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while !condition() {
        assert!(Instant::now() < deadline, "worker did not respond promptly");
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn describe_accepts_large_text_binary_and_split_responses() {
    for (binary, chunk_size) in [
        (false, usize::MAX),
        (true, usize::MAX),
        (false, 16 * 1024),
        (true, 16 * 1024),
    ] {
        let (mut client, mut server) = socket_pair();
        let body = "a=x-nv-test:description\r\n".repeat(12_000);
        assert!(body.len() > MAX_CONTROL_RESPONSE_BYTES);
        let expected_body = body.clone();
        let sender = thread::spawn(move || {
            let Message::Text(request) = server.read().unwrap() else {
                panic!("expected DESCRIBE request");
            };
            assert!(request.starts_with("DESCRIBE "));
            assert!(request.contains("\r\nCSeq: 8\r\n"));
            let response = format!(
                "RTSP/1.0 200 OK\r\nCSeq: 8\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            for chunk in response.as_bytes().chunks(chunk_size) {
                let message = if binary {
                    Message::Binary(chunk.to_vec().into())
                } else {
                    Message::Text(String::from_utf8(chunk.to_vec()).unwrap().into())
                };
                server.send(message).unwrap();
            }
        });
        let response = client
            .request("DESCRIBE", "rtsps://seat.nvidiagrid.net:322", &[], "")
            .unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, expected_body);
        assert!(client.buffer.is_empty());
        sender.join().unwrap();
    }
}

#[test]
fn request_rejects_unbounded_partial_response() {
    let (mut client, mut server) = socket_pair();
    let sender = thread::spawn(move || {
        server.read().unwrap();
        server
            .send(Message::Text("x".repeat(MAX_REQUEST_RESPONSE_BYTES).into()))
            .unwrap();
        server.send(Message::Text("x".into())).unwrap();
    });
    let error = client
        .request("DESCRIBE", "rtsps://seat.nvidiagrid.net:322", &[], "")
        .err()
        .unwrap();
    assert_eq!(error.code, "nvst-rtsp-failed");
    assert!(
        error
            .message
            .contains("DESCRIBE response exceeds request buffer limit")
    );
    sender.join().unwrap();
}

#[test]
fn request_bounds_websocket_message_size() {
    let (mut client, mut server) = socket_pair();
    let sender = thread::spawn(move || {
        server.read().unwrap();
        let _ = server.send(Message::Text(
            "x".repeat(MAX_REQUEST_RESPONSE_BYTES + 1).into(),
        ));
    });
    let error = client
        .request("DESCRIBE", "rtsps://seat.nvidiagrid.net:322", &[], "")
        .err()
        .unwrap();
    assert_eq!(error.code, "nvst-rtsp-failed");
    assert!(error.message.contains("Space limit exceeded"));
    assert!(client.buffer.is_empty());
    sender.join().unwrap();
}

#[test]
fn rtsp_parser_accepts_response_at_request_size_limit() {
    let prefix = "RTSP/1.0 200 OK\r\nCSeq: 8\r\nContent-Length: ";
    let body_length = MAX_REQUEST_RESPONSE_BYTES
        - prefix.len()
        - MAX_REQUEST_RESPONSE_BYTES.to_string().len()
        - 4;
    let body = "x".repeat(body_length);
    let mut buffer = format!("{prefix}{body_length}\r\n\r\n{body}");
    assert_eq!(buffer.len(), MAX_REQUEST_RESPONSE_BYTES);
    let response = take_rtsp_response(&mut buffer, 8).unwrap().unwrap();
    assert_eq!(response.body, body);
    assert!(buffer.is_empty());
}

#[test]
fn rtsp_parser_bounds_declared_response_size_before_receiving_body() {
    let mut oversized = format!(
        "RTSP/1.0 200 OK\r\nCSeq: 8\r\nContent-Length: {MAX_REQUEST_RESPONSE_BYTES}\r\n\r\n"
    );
    let error = take_rtsp_response(&mut oversized, 8).err().unwrap();
    assert!(
        error
            .message
            .contains("response exceeds request buffer limit")
    );

    let mut bounded = "RTSP/1.0 200 OK\r\nCSeq: 8\r\nContent-Length: 262144\r\n\r\n".to_owned();
    assert!(take_rtsp_response(&mut bounded, 8).unwrap().is_none());
}

#[test]
fn control_ping_parser_rejects_overflowing_and_invalid_body_boundaries() {
    let mut overflowing = format!(
        "RTSP/1.0 551 Response\r\nCSeq: 8\r\nContent-Length: {}\r\n\r\n",
        usize::MAX
    );
    assert!(take_rtsp_response(&mut overflowing, 8).is_err());
    let mut invalid = "RTSP/1.0 551 Response\r\nCSeq: 8\r\nContent-Length: 1\r\n\r\né".to_owned();
    assert!(take_rtsp_response(&mut invalid, 8).is_err());
}

#[test]
fn control_ping_samples_are_shared_expiring_and_clearable() {
    let ping = NvstControlPing::default();
    let clone = ping.clone();
    let sent = Instant::now();
    let received = sent + Duration::from_micros(4_750);
    assert_eq!(ping.ping_ms(sent), None);
    ping.record(sent, received);
    assert_eq!(clone.ping_ms(received), Some(4.75));
    assert_eq!(clone.ping_ms(sent), None);
    assert_eq!(ping.ping_ms(received + Duration::from_secs(4)), Some(4.75));
    assert_eq!(ping.ping_ms(received + CONTROL_PING_EXPIRY), None);
    assert_eq!(clone.ping_ms(received), None);
    ping.record(sent, received);
    clone.clear();
    assert_eq!(ping.ping_ms(received), None);
    ping.record(sent, sent);
    assert_eq!(ping.ping_ms(sent), Some(0.0));
}

#[test]
fn live_control_ping_uses_websocket_without_rtsp_requests() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    server_ping(&mut server);
    let payload = client_ping(&mut server);
    server.send(Message::Pong(vec![0; 8].into())).unwrap();
    assert_eq!(ping.ping_ms(Instant::now()), None);
    server.send(Message::Pong(payload.into())).unwrap();
    wait_until(|| ping.ping_ms(Instant::now()).is_some());
    assert!(ping.ping_ms(Instant::now()).unwrap() < 1000.0);
    server_ping(&mut server);
    server.close(None).unwrap();
    wait_until(|| active.worker.as_ref().unwrap().is_finished());
    assert_eq!(ping.ping_ms(Instant::now()), None);
    let shutdown_started = Instant::now();
    active.shutdown();
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
}

#[test]
fn live_control_ping_times_out_without_accepting_late_pongs() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    let first = client_ping(&mut server);
    server.send(Message::Pong(first.into())).unwrap();
    wait_until(|| ping.ping_ms(Instant::now()).is_some());
    let missing = client_ping(&mut server);
    server.send(Message::Pong(vec![0; 8].into())).unwrap();
    let next = client_ping(&mut server);
    server.send(Message::Pong(vec![0; 8].into())).unwrap();
    assert_eq!(
        u64::from_be_bytes(next.clone().try_into().unwrap()),
        u64::from_be_bytes(missing.clone().try_into().unwrap()) + 1
    );
    assert_eq!(ping.ping_ms(Instant::now()), None);
    server.send(Message::Pong(missing.into())).unwrap();
    server_ping(&mut server);
    assert_eq!(ping.ping_ms(Instant::now()), None);
    server.send(Message::Pong(next.into())).unwrap();
    wait_until(|| ping.ping_ms(Instant::now()).is_some());
    drop(server);
    wait_until(|| active.worker.as_ref().unwrap().is_finished());
    assert_eq!(ping.ping_ms(Instant::now()), None);
    active.shutdown();
}

#[test]
fn unanswered_websocket_ping_does_not_end_the_session() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    let first = client_ping(&mut server);
    server.send(Message::Pong(vec![0; 8].into())).unwrap();
    let second = client_ping(&mut server);
    server.send(Message::Pong(vec![0; 8].into())).unwrap();
    assert_ne!(first, second);
    assert_eq!(ping.ping_ms(Instant::now()), None);
    assert!(!active.worker.as_ref().unwrap().is_finished());
    let shutdown = thread::spawn(move || active.shutdown());
    assert!(matches!(server.read().unwrap(), Message::Close(_)));
    shutdown.join().unwrap();
}

#[test]
fn live_control_ping_shutdown_closes_without_rtsp_teardown() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    client_ping(&mut server);
    let shutdown_started = Instant::now();
    let shutdown = thread::spawn(move || active.shutdown());
    assert!(matches!(server.read().unwrap(), Message::Close(_)));
    shutdown.join().unwrap();
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
    assert_eq!(ping.ping_ms(Instant::now()), None);
}

#[test]
fn live_control_ping_shutdown_does_not_wait_for_pending_pong() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    client_ping(&mut server);
    let shutdown_started = Instant::now();
    let shutdown = thread::spawn(move || active.shutdown());
    assert!(matches!(server.read().unwrap(), Message::Close(_)));
    assert_eq!(ping.ping_ms(Instant::now()), None);
    shutdown.join().unwrap();
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
}

#[test]
fn live_control_ping_shutdown_clears_sample() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    let payload = client_ping(&mut server);
    server.send(Message::Pong(payload.into())).unwrap();
    wait_until(|| ping.ping_ms(Instant::now()).is_some());
    let shutdown_started = Instant::now();
    let shutdown = thread::spawn(move || active.shutdown());
    assert!(matches!(server.read().unwrap(), Message::Close(_)));
    assert_eq!(ping.ping_ms(Instant::now()), None);
    shutdown.join().unwrap();
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
}

#[test]
fn live_control_ping_rejects_unbounded_partial_response() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let mut active = active_session(client, &ping);
    let payload = client_ping(&mut server);
    server.send(Message::Pong(payload.into())).unwrap();
    wait_until(|| ping.ping_ms(Instant::now()).is_some());
    server
        .send(Message::Text("x".repeat(MAX_CONTROL_RESPONSE_BYTES).into()))
        .unwrap();
    server.send(Message::Text("x".into())).unwrap();
    wait_until(|| active.worker.as_ref().unwrap().is_finished());
    assert_eq!(ping.ping_ms(Instant::now()), None);
    active.shutdown();
}

fn exit_reason(mut active: ActiveNvstRtspSession) -> ControlExit {
    let worker = active.worker.take().unwrap();
    wait_until(|| worker.is_finished());
    worker.join().unwrap()
}

#[test]
fn live_control_exit_records_server_close_code_and_reason() {
    // Isolate the actual persistent logger: other core tests also replace its
    // process-global sink, so a test-thread lock here would not prevent races.
    if std::env::var_os("OPENNOW_RTSPS_PRIVACY_TEST_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "nvst_rtsp::control_ping_tests::live_control_exit_records_server_close_code_and_reason", "--nocapture", "--test-threads=1"])
            .env("OPENNOW_RTSPS_PRIVACY_TEST_CHILD", "1")
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("privacy_log_fixture="),
            "privacy child did not reach its final assertions: {stdout}"
        );
        assert!(
            stdout.contains("1 passed"),
            "privacy child did not run its one test: {stdout}"
        );
        println!("{stdout}");
        return;
    }
    let path = std::env::temp_dir().join(format!(
        "opennow-rtsps-privacy-{}-{}.log",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    opennow_streamer_protocol::log::set_log_file(path.to_str().unwrap()).unwrap();
    use tungstenite::protocol::CloseFrame;
    use tungstenite::protocol::frame::coding::CloseCode;
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let active = active_session(client, &ping);
    server
        .close(Some(CloseFrame {
            code: CloseCode::Policy,
            reason: "privacy-close-secret session=private-id\nexpired".into(),
        }))
        .unwrap();
    let reason = exit_reason(active);
    assert_eq!(reason, ControlExit::ServerClose(Some(1008)));
    assert_eq!(reason.to_string(), "server-close code=1008");
    live_control_exit_records_unparseable_server_request_head();
    live_control_exit_survives_mismatched_response_and_reports_shutdown();
    live_control_exit_records_transport_loss();
    let evidence = std::fs::read_to_string(&path).unwrap();
    for secret in [
        "privacy-close-secret",
        "private-id",
        "privacy-request-secret",
        "privacy-parser-secret",
        "privacy-header-secret",
        "privacy-status-secret",
        "privacy-unsolicited-secret",
        "privacy-transport-secret",
        "rtsps://seat.nvidiagrid.net",
    ] {
        assert!(
            !evidence.contains(secret),
            "peer text leaked into persistent diagnostics: {secret}"
        );
    }
    assert!(evidence.contains("control_response status=551"));
    assert!(evidence.contains("control_unsolicited category=sequence-mismatch"));
    assert!(evidence.contains("control_exit reason=server-close code=1008"));
    assert!(evidence.contains("control_exit reason=unparseable-server-message"));
    let transport_exit = evidence
        .lines()
        .find(|line| {
            line.contains("control_exit reason=read-failed")
                || line.contains("control_exit reason=ping-send-failed")
        })
        .unwrap();
    assert!(transport_exit.contains("error_category="));
    assert!(transport_exit.contains("io_kind="));
    assert!(evidence.contains("error_category=Io io_kind=Some(ConnectionReset)"));
    assert!(evidence.contains("error_category=Io io_kind=Some(UnexpectedEof)"));
    assert!(evidence.contains("error_category=Utf8 io_kind=None"));
    for line in evidence
        .lines()
        .filter(|line| line.contains("control_exit"))
    {
        for field in [
            "uptime_ms=",
            "pings_sent=",
            "pongs=",
            "missed_pongs=",
            "last_inbound_ms_ago=",
        ] {
            assert!(line.contains(field), "lifecycle counter missing: {field}");
        }
    }
    println!("privacy_log_fixture={}", path.display());
}

#[test]
fn live_control_exit_records_unparseable_server_request_head() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let active = active_session(client, &ping);
    server
        .send(Message::Text(
            "TEARDOWN rtsps://seat.nvidiagrid.net:322/privacy-request-secret?token=privacy-parser-secret RTSP/1.0\r\nCSeq: 1\r\nX-Session: privacy-header-secret\r\n\r\n".into(),
        ))
        .unwrap();
    let reason = exit_reason(active);
    assert_eq!(reason, ControlExit::UnparseableServerMessage);
    assert_eq!(reason.to_string(), "unparseable-server-message");
}

#[test]
fn live_control_exit_survives_mismatched_response_and_reports_shutdown() {
    let (client, mut server) = socket_pair();
    let ping = NvstControlPing::default();
    let active = active_session(client, &ping);
    server
        .send(Message::Text("RTSP/1.0 551 privacy-status-secret\r\nCSeq: 7\r\nX-Session: privacy-header-secret\r\n\r\n".into()))
        .unwrap();
    server_ping(&mut server);
    server
        .send(Message::Text(
            "RTSP/1.0 200 privacy-unsolicited-secret\r\nCSeq: 3\r\n\r\n".into(),
        ))
        .unwrap();
    server_ping(&mut server);
    assert!(!active.worker.as_ref().unwrap().is_finished());
    active.control.send(Control::Shutdown).unwrap();
    assert!(matches!(server.read().unwrap(), Message::Close(_)));
    assert_eq!(exit_reason(active), ControlExit::LocalShutdown);
}

#[test]
fn live_control_exit_records_transport_loss() {
    let (client, server) = socket_pair();
    let ping = NvstControlPing::default();
    let active = active_session(client, &ping);
    drop(server);
    let reason = exit_reason(active);
    assert!(
        matches!(
            reason,
            ControlExit::ReadFailed(_) | ControlExit::PingSendFailed(_)
        ),
        "{reason}"
    );
    // Preserve reset/EOF evidence without ever retaining an I/O error message.
    // Exercise all three send/read exit formatters and a payload-bearing non-I/O error.
    for error in [
        tungstenite::Error::Io(std::io::Error::new(
            ErrorKind::ConnectionReset,
            "privacy-transport-secret",
        )),
        tungstenite::Error::Io(std::io::Error::new(
            ErrorKind::UnexpectedEof,
            "privacy-transport-secret",
        )),
        tungstenite::Error::Utf8("privacy-transport-secret".into()),
    ] {
        let expected_kind = match &error {
            tungstenite::Error::Io(error) => Some(error.kind()),
            _ => None,
        };
        for exit in [
            ControlExit::ReadFailed,
            ControlExit::PingSendFailed,
            ControlExit::PongSendFailed,
        ] {
            let failure = ControlFailure::from_error(&error);
            assert_eq!(failure.io_kind, expected_kind);
            assert_eq!(
                failure.category,
                if expected_kind.is_some() {
                    ControlFailureCategory::Io
                } else {
                    ControlFailureCategory::Utf8
                }
            );
            let formatted = exit(failure).to_string();
            assert!(!formatted.contains("privacy-transport-secret"));
            assert!(formatted.contains(&format!("io_kind={expected_kind:?}")));
            // Same persistent sink as the isolated fixture; no real transport text.
            opennow_streamer_protocol::log::log_line("INFO", "rtsps-privacy-fixture", &formatted);
        }
    }
}
