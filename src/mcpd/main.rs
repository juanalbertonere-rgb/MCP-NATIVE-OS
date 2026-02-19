use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::fs;

#[path = "../common.rs"]
mod common;
#[path = "../security/risk_engine.rs"]
mod risk_engine;

use common::{Tool, RiskLevel};
use risk_engine::RiskEngine;

struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

struct McpDaemon {
    registry: Arc<Mutex<ToolRegistry>>,
}

impl McpDaemon {
    fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ToolRegistry {
                tools: HashMap::new(),
            })),
        }
    }

    async fn register_tool(&self, tool: Tool) {
        let mut registry = self.registry.lock().await;
        println!("Registering tool: {} from {}", tool.name, tool.provider);
        registry.tools.insert(tool.name.clone(), tool);
    }

    async fn handle_request(&self, request: serde_json::Value) -> Result<serde_json::Value, String> {
        let method = request["method"].as_str().ok_or("Missing method")?;
        let params = request["params"].clone();
        let id = request["id"].clone();

        let registry = self.registry.lock().await;
        let tool = registry.tools.get(method).ok_or_else(|| format!("Tool not found: {}", method))?;

        println!("Processing request for tool: {} (Risk: {:?})", tool.name, tool.risk_level);

        // Risk Assessment
        let context = request["context"].clone();
        let assessment = RiskEngine::assess(method, &params, &context);
        println!("Risk Assessment: level={:?}, requires_confirmation={}, reason={}",
                 assessment.level, assessment.requires_confirmation, assessment.reason);

        if assessment.requires_confirmation {
            println!("CONFIRMATION REQUIRED for {}", tool.name);
        }

        // Simulate execution
        let result = match method {
            "camera.capture" => serde_json::json!({"status": "success", "image_url": "content://media/external/images/media/123"}),
            "contacts.resolve" => serde_json::json!({"status": "success", "contact": {"id": "456", "name": "Mom", "phone": "555-0199"}}),
            "messages.send" => serde_json::json!({"status": "success", "message_id": "789"}),
            _ => serde_json::json!({"status": "success", "result": "Action executed (scaffold)"}),
        };

        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting MCP System Daemon (mcpd)...");

    let daemon = Arc::new(McpDaemon::new());

    daemon.register_tool(Tool {
        name: "camera.capture".to_string(),
        provider: "system.camera".to_string(),
        risk_level: RiskLevel::Medium,
    }).await;

    daemon.register_tool(Tool {
        name: "contacts.resolve".to_string(),
        provider: "system.contacts".to_string(),
        risk_level: RiskLevel::Low,
    }).await;

    daemon.register_tool(Tool {
        name: "messages.send".to_string(),
        provider: "system.messages".to_string(),
        risk_level: RiskLevel::Medium,
    }).await;

    let socket_path = "/tmp/mcpd.sock";
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;

    println!("mcpd is running and listening on UDS {}", socket_path);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let daemon_inner = Arc::clone(&daemon);

        tokio::spawn(async move {
            let mut buffer = [0; 4096];
            loop {
                let n = match socket.read(&mut buffer).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(_) => return,
                };

                let request_str = String::from_utf8_lossy(&buffer[..n]);
                if let Ok(request) = serde_json::from_str::<serde_json::Value>(&request_str) {
                    let response = daemon_inner.handle_request(request).await.unwrap_or_else(|e| {
                        serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32603, "message": e}, "id": null})
                    });

                    let response_str = serde_json::to_string(&response).unwrap();
                    let _ = socket.write_all(response_str.as_bytes()).await;
                }
            }
        });
    }
}
