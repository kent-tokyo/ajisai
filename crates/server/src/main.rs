mod handlers;
mod protocol;

use handlers::*;
use protocol::*;
use std::io::{BufRead, BufReader, Write};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<RpcRequest>(line) {
            Ok(req) => {
                let response = match req.method.as_str() {
                    "ping" => RpcResponse {
                        id: req.id,
                        result: Some(ping().await),
                        error: None,
                    },
                    "get_transforms" => RpcResponse {
                        id: req.id,
                        result: Some(get_transforms()),
                        error: None,
                    },
                    "run_pipeline" => {
                        match run_pipeline(req.params.get("pipeline").cloned().unwrap_or_default())
                            .await
                        {
                            Ok(result) => RpcResponse {
                                id: req.id,
                                result: Some(result),
                                error: None,
                            },
                            Err(err) => RpcResponse {
                                id: req.id,
                                result: None,
                                error: Some(RpcError::internal_error(err)),
                            },
                        }
                    }
                    "validate_pipeline" => match validate_pipeline(
                        req.params.get("pipeline").cloned().unwrap_or_default(),
                    ) {
                        Ok(result) => RpcResponse {
                            id: req.id,
                            result: Some(result),
                            error: None,
                        },
                        Err(err) => RpcResponse {
                            id: req.id,
                            result: None,
                            error: Some(RpcError::internal_error(err)),
                        },
                    },
                    "load_pipeline" => {
                        let path = req
                            .params
                            .get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        match load_pipeline(path) {
                            Ok(result) => RpcResponse {
                                id: req.id,
                                result: Some(result),
                                error: None,
                            },
                            Err(err) => RpcResponse {
                                id: req.id,
                                result: None,
                                error: Some(RpcError::internal_error(err)),
                            },
                        }
                    }
                    "save_pipeline" => {
                        let path = req
                            .params
                            .get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let format = req
                            .params
                            .get("format")
                            .and_then(|v| v.as_str())
                            .unwrap_or("json")
                            .to_string();
                        let pipeline_json = req.params.get("pipeline").cloned().unwrap_or_default();
                        match save_pipeline(pipeline_json, path, format) {
                            Ok(result) => RpcResponse {
                                id: req.id,
                                result: Some(result),
                                error: None,
                            },
                            Err(err) => RpcResponse {
                                id: req.id,
                                result: None,
                                error: Some(RpcError::internal_error(err)),
                            },
                        }
                    }
                    _ => RpcResponse {
                        id: req.id,
                        result: None,
                        error: Some(RpcError::new(-32601, "Method not found")),
                    },
                };

                let response_line = serde_json::to_string(&response).unwrap();
                writer.write_all(response_line.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
            Err(e) => {
                eprintln!("Failed to parse JSON-RPC request: {}", e);
            }
        }
    }

    Ok(())
}
