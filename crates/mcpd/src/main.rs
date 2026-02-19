use std::collections::BTreeMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tokio::net::UnixListener;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use std::fs;
use std::time::{SystemTime, Duration};

use mcp_common::{Tool, RiskLevel};
use mcp_security::{RiskEngine, IntegrityManager};
use mcp_protocol::{JsonRpcRequest, JsonRpcResponse};

#[derive(Serialize, Deserialize)]
struct ToolRegistry {
    tools: BTreeMap<String, Tool>,
    #[serde(default)]
    signature: String,
}

impl ToolRegistry {
    fn save_to_disk(&mut self) {
        self.signature = "".to_string(); // Clear signature for signing
        if let Ok(json) = serde_json::to_string(&self) {
            let signature = IntegrityManager::sign_data(&json, b"secret_key");
            self.signature = signature;
            if let Ok(final_json) = serde_json::to_string_pretty(&self) {
                let _ = fs::write("tool_registry.json", final_json);
            }
        }
    }

    fn load_from_disk() -> Self {
        if let Ok(data) = fs::read_to_string("tool_registry.json") {
            if let Ok(mut registry) = serde_json::from_str::<ToolRegistry>(&data) {
                let sig = registry.signature.clone();
                registry.signature = "".to_string();
                if let Ok(json) = serde_json::to_string(&registry) {
                    if IntegrityManager::verify_data(&json, &sig, b"secret_key") {
                        println!("Tool registry integrity verified.");
                        return registry;
                    } else {
                        eprintln!("WARNING: Tool registry integrity check failed! Loading empty registry.");
                    }
                }
            }
        }
        Self { tools: BTreeMap::new(), signature: "".to_string() }
    }
}

#[derive(Clone)]
struct CachedRequest {
    request: JsonRpcRequest,
    timestamp: SystemTime,
}

struct McpDaemon {
    registry: Arc<Mutex<ToolRegistry>>,
    cached_requests: Arc<Mutex<BTreeMap<String, CachedRequest>>>,
    config: SecurityConfig,
}

#[derive(Deserialize, Clone)]
struct DaemonConfig {
    security: SecurityConfig,
}

#[derive(Deserialize, Clone)]
struct SecurityConfig {
    #[serde(default = "default_ttl")]
    confirmation_ttl_secs: u64,
    #[serde(default = "default_max_tokens")]
    max_pending_tokens: usize,
    #[serde(default = "default_allowed_uids")]
    allowed_uids: Vec<u32>,
}

fn default_ttl() -> u64 { 30 }
fn default_max_tokens() -> usize { 100 }
fn default_allowed_uids() -> Vec<u32> { vec![1001] } // Default for testing

impl McpDaemon {
    fn new(config: SecurityConfig) -> Self {
        Self {
            registry: Arc::new(Mutex::new(ToolRegistry::load_from_disk())),
            cached_requests: Arc::new(Mutex::new(BTreeMap::new())),
            config,
        }
    }

    async fn register_tool(&self, tool: Tool) {
        let mut registry = self.registry.lock().await;
        println!("Registering tool: {} from {} with capabilities {:?}", tool.name, tool.provider, tool.capabilities);
        registry.tools.insert(tool.name.clone(), tool);
        registry.save_to_disk();
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let method = request.method.as_str();
        let params = &request.params;
        let id = request.id.clone();

        if method == "system.confirm" {
            let token = match params["confirmation_token"].as_str() {
                Some(t) => t,
                None => return JsonRpcResponse::error(id, -32602, "Missing confirmation_token".to_string(), None),
            };

            let mut cached_lock = self.cached_requests.lock().await;

            // Cleanup expired tokens
            let ttl = Duration::from_secs(self.config.confirmation_ttl_secs);
            cached_lock.retain(|_, v| v.timestamp.elapsed().unwrap_or(ttl) < ttl);

            let original = match cached_lock.remove(token) {
                Some(c) => c,
                None => return JsonRpcResponse::error(id, -32000, "Invalid or expired confirmation token".to_string(), None),
            };
            drop(cached_lock);

            println!("Confirmation received for {}. Executing...", original.request.method);
            return self.execute_tool(&original.request.method, &original.request.params, id).await;
        }

        if method == "system.register_tool" {
            let tool_name = match params["name"].as_str() {
                Some(n) => n.to_string(),
                None => return JsonRpcResponse::error(id, -32602, "Missing name".to_string(), None),
            };
            let provider = match params["provider"].as_str() {
                Some(p) => p.to_string(),
                None => return JsonRpcResponse::error(id, -32602, "Missing provider".to_string(), None),
            };
            let risk_level_str = params["risk_level"].as_str().unwrap_or("Low");
            let capabilities = params["capabilities"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(Vec::new);

            let risk_level = match risk_level_str {
                "High" => RiskLevel::High,
                "Medium" => RiskLevel::Medium,
                _ => RiskLevel::Low,
            };

            self.register_tool(Tool {
                name: tool_name.clone(),
                provider,
                risk_level,
                capabilities,
            }).await;

            return JsonRpcResponse::success(id, serde_json::json!({"status": "registered", "tool": tool_name}));
        }

        let registry = self.registry.lock().await;
        let tool = match registry.tools.get(method) {
            Some(t) => t,
            None => return JsonRpcResponse::error(id, -32601, format!("Tool not found: {}", method), None),
        };

        println!("Processing request for tool: {} (Risk: {:?})", tool.name, tool.risk_level);

        // Risk Assessment
        let assessment = RiskEngine::assess(tool, params, &request.context);
        println!("Risk Assessment: level={:?}, requires_confirmation={}, reason={}",
                 assessment.level, assessment.requires_confirmation, assessment.reason);

        if assessment.requires_confirmation {
            println!("CONFIRMATION REQUIRED for {}", tool.name);
            let token = format!("conf_{}", uuid::Uuid::new_v4());
            let mut cached = self.cached_requests.lock().await;

            // Cleanup expired tokens
            let ttl = Duration::from_secs(self.config.confirmation_ttl_secs);
            cached.retain(|_, v| v.timestamp.elapsed().unwrap_or(ttl) < ttl);

            if cached.len() >= self.config.max_pending_tokens {
                return JsonRpcResponse::error(id, -32000, "Too many pending confirmations".to_string(), None);
            }

            cached.insert(token.clone(), CachedRequest {
                request: request.clone(),
                timestamp: SystemTime::now(),
            });

            return JsonRpcResponse::error(id, -32000, "Confirmation required".to_string(), Some(serde_json::json!({
                "confirmation_token": token,
                "reason": assessment.reason
            })));
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
            // Simple rotation logic
            let log_path = "transactions.log";
            if let Ok(metadata) = fs::metadata(log_path) {
                if metadata.len() > 10 * 1024 * 1024 { // 10MB
                    let _ = fs::rename("transactions.log.1", "transactions.log.2");
                    let _ = fs::rename("transactions.log", "transactions.log.1");
                }
            }

            if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(log_path) {
                let _ = writeln!(file, "{}", line);
            }
        }
    }

    async fn execute_tool(&self, method: &str, params: &serde_json::Value, id: serde_json::Value) -> JsonRpcResponse {
        // Simulate execution
        let result = match method {
            "camera.capture" => serde_json::json!({"status": "success", "image_url": "content://media/external/images/media/123"}),
            "contacts.resolve" => serde_json::json!({"status": "success", "contact": {"id": "456", "name": "Mom", "phone": "555-0199"}}),
            "messages.send" => serde_json::json!({"status": "success", "message_id": "789"}),
            _ => serde_json::json!({"status": "success", "result": "Action executed (scaffold)"}),
        };

        self.log_transaction(method, params, &result).await;
        JsonRpcResponse::success(id, result)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting MCP System Daemon (mcpd)...");

    // Load config
    let config: SecurityConfig = match fs::read_to_string("config.toml") {
        Ok(content) => {
            let full_config: DaemonConfig = toml::from_str(&content).unwrap_or_else(|e| {
                println!("Invalid config.toml ({}), using defaults", e);
                DaemonConfig { security: SecurityConfig { confirmation_ttl_secs: 30, max_pending_tokens: 100, allowed_uids: vec![1001] } }
            });
            full_config.security
        },
        Err(_) => SecurityConfig { confirmation_ttl_secs: 30, max_pending_tokens: 100, allowed_uids: vec![1001] },
    };

    let daemon = Arc::new(McpDaemon::new(config.clone()));

    // Initial tool registration
    {
        let mut reg = daemon.registry.lock().await;
        reg.tools.insert("camera.capture".to_string(), Tool {
            name: "camera.capture".to_string(),
            provider: "system.camera".to_string(),
            risk_level: RiskLevel::Medium,
            capabilities: vec!["privacy_sensitive".to_string()],
        });
        reg.tools.insert("contacts.resolve".to_string(), Tool {
            name: "contacts.resolve".to_string(),
            provider: "system.contacts".to_string(),
            risk_level: RiskLevel::Low,
            capabilities: vec![],
        });
        reg.tools.insert("messages.send".to_string(), Tool {
            name: "messages.send".to_string(),
            provider: "system.messages".to_string(),
            risk_level: RiskLevel::Medium,
            capabilities: vec!["privacy_sensitive".to_string()],
        });
        reg.tools.insert("financial.transfer".to_string(), Tool {
            name: "financial.transfer".to_string(),
            provider: "system.finance".to_string(),
            risk_level: RiskLevel::High,
            capabilities: vec!["financial".to_string()],
        });
        reg.save_to_disk();
    }

    let socket_path = "/tmp/mcpd.sock";
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;

    println!("mcpd is running and listening on UDS {}", socket_path);

    loop {
        let (mut socket, _) = listener.accept().await?;

        // Peer credential check
        #[cfg(target_os = "linux")]
        {
            if let Ok(cred) = socket.peer_cred() {
                if !config.allowed_uids.contains(&cred.uid()) {
                    eprintln!("Unauthorized connection from UID {}", cred.uid());
                    continue;
                }
            }
        }

        let daemon_inner = Arc::clone(&daemon);

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut lines = tokio::io::BufReader::new(reader).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
                    let response = daemon_inner.handle_request(request).await;
                    if let Ok(mut response_str) = serde_json::to_string(&response) {
                        response_str.push('\n');
                        let _ = writer.write_all(response_str.as_bytes()).await;
                    }
                } else {
                    let error = JsonRpcResponse::error(serde_json::Value::Null, -32700, "Parse error".to_string(), None);
                    if let Ok(mut response_str) = serde_json::to_string(&error) {
                        response_str.push('\n');
                        let _ = writer.write_all(response_str.as_bytes()).await;
                    }
                }
            }
        });
    }
}
