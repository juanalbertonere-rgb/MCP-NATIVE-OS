use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt};
use std::fs;

#[path = "../common.rs"]
mod common;
#[path = "../security/risk_engine.rs"]
mod risk_engine;

use common::{Tool, RiskLevel};
use risk_engine::RiskEngine;

#[derive(Serialize, Deserialize)]
struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            let _ = fs::write("tool_registry.json", json);
        }
    }

    fn load_from_disk() -> Self {
        if let Ok(data) = fs::read_to_string("tool_registry.json") {
            if let Ok(registry) = serde_json::from_str(&data) {
                return registry;
            }
        }
        Self { tools: HashMap::new() }
    }
}

struct McpDaemon {
    registry: Arc<Mutex<ToolRegistry>>,
    cached_requests: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl McpDaemon {
    fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ToolRegistry::load_from_disk())),
            cached_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn register_tool(&self, tool: Tool) {
        let mut registry = self.registry.lock().await;
        println!("Registering tool: {} from {}", tool.name, tool.provider);
        registry.tools.insert(tool.name.clone(), tool);
        registry.save_to_disk();
    }

    async fn handle_request(&self, request: serde_json::Value) -> Result<serde_json::Value, String> {
        let method = request["method"].as_str().ok_or("Missing method")?;
        let params = request["params"].clone();
        let id = request["id"].clone();

        if method == "system.confirm" {
            let token = params["confirmation_token"].as_str().ok_or("Missing confirmation_token")?;
            let mut cached = self.cached_requests.lock().await;
            let original_request = cached.remove(token).ok_or("Invalid or expired confirmation token")?;

            let original_method = original_request["method"].as_str().unwrap();
            let original_params = original_request["params"].clone();

            println!("Confirmation received for {}. Executing...", original_method);
            return self.execute_tool(original_method, original_params, id).await;
        }

        if method == "system.register_tool" {
            let tool_name = params["name"].as_str().ok_or("Missing name")?.to_string();
            let provider = params["provider"].as_str().ok_or("Missing provider")?.to_string();
            let risk_level_str = params["risk_level"].as_str().unwrap_or("Low");

            let risk_level = match risk_level_str {
                "High" => RiskLevel::High,
                "Medium" => RiskLevel::Medium,
                _ => RiskLevel::Low,
            };

            self.register_tool(Tool {
                name: tool_name.clone(),
                provider,
                risk_level,
            }).await;

            return Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"status": "registered", "tool": tool_name},
                "id": id
            }));
        }

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
            let token = format!("conf_{}", uuid::Uuid::new_v4());
            let mut cached = self.cached_requests.lock().await;
            cached.insert(token.clone(), request.clone());

            return Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32000,
                    "message": "Confirmation required",
                    "data": {
                        "confirmation_token": token,
                        "reason": assessment.reason
                    }
                },
                "id": id
            }));
        }

        self.execute_tool(method, params, id).await
    }

    async fn log_transaction(&self, method: &str, params: &serde_json::Value, result: &serde_json::Value) {
        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "method": method,
            "params": params,
            "result": result
        });
        if let Ok(line) = serde_json::to_string(&entry) {
            use std::io::Write;
            if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open("transactions.log") {
                let _ = writeln!(file, "{}", line);
            }
        }
    }

    async fn execute_tool(&self, method: &str, params: serde_json::Value, id: serde_json::Value) -> Result<serde_json::Value, String> {
        // Simulate execution
        let result = match method {
            "camera.capture" => serde_json::json!({"status": "success", "image_url": "content://media/external/images/media/123"}),
            "contacts.resolve" => serde_json::json!({"status": "success", "contact": {"id": "456", "name": "Mom", "phone": "555-0199"}}),
            "messages.send" => serde_json::json!({"status": "success", "message_id": "789"}),
            _ => serde_json::json!({"status": "success", "result": "Action executed (scaffold)"}),
        };

        self.log_transaction(method, &params, &result).await;

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

    daemon.register_tool(Tool {
        name: "financial.transfer".to_string(),
        provider: "system.finance".to_string(),
        risk_level: RiskLevel::High,
    }).await;

    let socket_path = "/tmp/mcpd.sock";
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;

    println!("mcpd is running and listening on UDS {}", socket_path);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let daemon_inner = Arc::clone(&daemon);

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut lines = tokio::io::BufReader::new(reader).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
                    let response = daemon_inner.handle_request(request).await.unwrap_or_else(|e| {
                        serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32603, "message": e}, "id": null})
                    });

                    let mut response_str = serde_json::to_string(&response).unwrap();
                    response_str.push('\n');
                    let _ = writer.write_all(response_str.as_bytes()).await;
                }
            }
        });
    }
}
