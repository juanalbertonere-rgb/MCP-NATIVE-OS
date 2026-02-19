use mcp_common::{RiskLevel, Tool};

pub struct RiskAssessment {
    pub level: RiskLevel,
    pub requires_confirmation: bool,
    pub reason: String,
}

pub struct RiskEngine;

impl RiskEngine {
    pub fn assess(tool: &Tool, _params: &serde_json::Value, context: &serde_json::Value) -> RiskAssessment {
        // Derive risk from capabilities
        let mut level = tool.risk_level.clone();

        for capability in &tool.capabilities {
            match capability.as_str() {
                "financial" | "system_admin" => {
                    if level != RiskLevel::High {
                        level = RiskLevel::High;
                    }
                },
                "privacy_sensitive" => {
                    if level == RiskLevel::Low {
                        level = RiskLevel::Medium;
                    }
                },
                _ => {}
            }
        }

        // 2. AI Contextual Layer (Simulated)
        let is_ambiguous = context["confidence"].as_f64().unwrap_or(1.0) < 0.7;

        if level == RiskLevel::High {
            return RiskAssessment {
                level: RiskLevel::High,
                requires_confirmation: true,
                reason: "High-risk system action detected.".to_string(),
            };
        }

        if is_ambiguous {
            return RiskAssessment {
                level: RiskLevel::Medium,
                requires_confirmation: true,
                reason: "Agent confidence is low; clarifying intent.".to_string(),
            };
        }

        RiskAssessment {
            level,
            requires_confirmation: false,
            reason: "Safe to proceed.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_common::{Tool, RiskLevel};
    use serde_json::json;

    #[test]
    fn test_risk_assessment_high_capability() {
        let tool = Tool {
            name: "test.tool".to_string(),
            provider: "test".to_string(),
            risk_level: RiskLevel::Low,
            capabilities: vec!["financial".to_string()],
        };
        let assessment = RiskEngine::assess(&tool, &json!({}), &json!({"confidence": 1.0}));
        assert_eq!(assessment.level, RiskLevel::High);
        assert!(assessment.requires_confirmation);
    }

    #[test]
    fn test_risk_assessment_ambiguous() {
        let tool = Tool {
            name: "test.tool".to_string(),
            provider: "test".to_string(),
            risk_level: RiskLevel::Low,
            capabilities: vec![],
        };
        let assessment = RiskEngine::assess(&tool, &json!({}), &json!({"confidence": 0.5}));
        assert_eq!(assessment.level, RiskLevel::Medium);
        assert!(assessment.requires_confirmation);
    }
}

pub struct IntegrityManager;

impl IntegrityManager {
    pub fn sign_data(data: &str, key: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    pub fn verify_data(data: &str, signature: &str, key: &[u8]) -> bool {
        let expected = Self::sign_data(data, key);
        expected == signature
    }
}
