use crate::common::RiskLevel;

pub struct RiskAssessment {
    pub level: RiskLevel,
    pub requires_confirmation: bool,
    pub reason: String,
}

pub struct RiskEngine;

impl RiskEngine {
    pub fn assess(tool_name: &str, _params: &serde_json::Value, context: &serde_json::Value) -> RiskAssessment {
        // 1. Deterministic Layer
        let base_risk = match tool_name {
            "financial.transfer" | "system.factory_reset" | "file.delete_all" => RiskLevel::High,
            "messages.send" | "contacts.modify" => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        // 2. AI Contextual Layer (Simulated)
        let is_ambiguous = context["confidence"].as_f64().unwrap_or(1.0) < 0.7;

        if let RiskLevel::High = base_risk {
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
            level: base_risk,
            requires_confirmation: false,
            reason: "Safe to proceed.".to_string(),
        }
    }
}
