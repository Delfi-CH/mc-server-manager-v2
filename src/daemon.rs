use axum::{routing::{get, any},Router, response::IntoResponse, extract::{WebSocketUpgrade, ws::{WebSocket, Message, CloseFrame}}};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokio::process::ChildStdin;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::Command;

static PROGRAM_IO: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PROGRAM_STDIN: Lazy<Mutex<HashMap<String, ChildStdin>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[tokio::main]
async fn main() {
    println!("Hello, daemon!");
    // TODO:
    // Setup websocket

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/socket", any(websocket_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


// WebSocketUpgrade: Extractor for establishing WebSocket connections.
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // Finalize upgrading the connection and call the provided callback with the stream.
    ws.on_failed_upgrade(|error| println!("Error upgrading websocket: {}", error))
        .on_upgrade(handle_socket)
}

// WebSocket: A stream of WebSocket messages.
async fn handle_socket(mut socket: WebSocket) {
    // Returns `None` if the stream has closed.
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(utf8_bytes) => {
                    println!("Text received: {}", utf8_bytes);
                    let result = socket
                        .send(Message::Text(
                            format!("Echo back text: {}", utf8_bytes).into(),
                        ))
                        .await;
                    if let Err(error) = result {
                        println!("Error sending: {}", error);
                        send_close_message(socket, 1011, &format!("Error occured: {}", error))
                            .await;
                        break;
                    }
                }
                Message::Binary(bytes) => {
                    println!("Received bytes of length: {}", bytes.len());
                    let result = socket
                        .send(Message::Text(
                            format!("Received bytes of length: {}", bytes.len()).into(),
                        ))
                        .await;
                    if let Err(error) = result {
                        println!("Error sending: {}", error);
                        send_close_message(socket, 1011, &format!("Error occured: {}", error))
                            .await;
                        break;
                    }
                }
                // Close, Ping, Pong will be handled automatically
                // Message::Close
                // After receiving a close frame, axum will automatically respond with a close frame if necessary (you do not have to deal with this yourself).
                // After sending a close frame, you may still read messages, but attempts to send another message will error.
                // Since no further messages will be received, you may either do nothing or explicitly drop the connection.
                _ => {}
            }
        } else {
            let error = msg.err().unwrap();
            println!("Error receiving message: {:?}", error);
            send_close_message(socket, 1011, &format!("Error occured: {}", error)).await;
            break;
        }
    }
}

// We MAY “uncleanly” close a WebSocket connection at any time by simply dropping the WebSocket, ie: Break out of the recv loop.
// However, you may also use the graceful closing protocol, in which
// peer A sends a close frame, and does not send any further messages;
// peer B responds with a close frame, and does not send any further messages;
// peer A processes the remaining messages sent by peer B, before finally
// both peers close the connection.
//
// Close Code: https://kapeli.com/cheat_sheets/WebSocket_Status_Codes.docset/Contents/Resources/Documents/index
async fn send_close_message(mut socket: WebSocket, code: u16, reason: &str) {
    _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: code,
            reason: reason.into(),
        })))
        .await;
}

async fn run_program_background(name: &str, cmd: &str, args: &[&str]) {
    let name = name.to_string();
    let cmd = cmd.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let mut child = Command::new(cmd)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start program");

    if let Some(stdin) = child.stdin.take() {
        PROGRAM_STDIN.lock().await.insert(name.clone(), stdin);
    }

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut stdout_accum = String::new();

        while let Ok(Some(line)) = lines.next_line().await {
            stdout_accum.push_str(&line);
            stdout_accum.push('\n');

            let mut io_map = PROGRAM_IO.lock().await;
            io_map.insert(name.clone(), stdout_accum.clone());
        }
    }

    let _ = child.wait().await;
}


async fn get_program_io(name: &str) -> Option<String> {
    let io_map = PROGRAM_IO.lock().await;
    io_map.get(name).cloned()
}

async fn write_to_program(name: &str, command: &str) {
    let mut stdin_map = PROGRAM_STDIN.lock().await;
    if let Some(stdin) = stdin_map.get_mut(name) {
        stdin.write_all(command.as_bytes()).await.unwrap();
        stdin.write_all(b"\n").await.unwrap();
        stdin.flush().await.unwrap();
    }
}