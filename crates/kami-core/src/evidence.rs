use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One piece of evidence backing a project-state claim (§4, §25). Kami
/// never lets an LLM assert a strength, weakness, or size figure without
/// an `Evidence` record a later loop can trace the claim back to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    /// What the evidence is about (e.g. a strength id, "project-size").
    pub subject: String,
    /// The analysis subsystem that produced it (e.g. "kami-sizing::bart").
    pub source: String,
    pub timestamp: DateTime<Utc>,
    /// 0.0-1.0. Deterministic measurements (Bart output, a file walk)
    /// should report 1.0; anything derived from an LLM must report < 1.0.
    pub confidence: f64,
    pub data: serde_json::Value,
    /// Free-text description of how this evidence was obtained, e.g. a
    /// binary path and invocation, or a file path and line range.
    pub provenance: String,
}

impl Evidence {
    pub fn new(
        subject: impl Into<String>,
        source: impl Into<String>,
        provenance: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("ev-{}", uuid::Uuid::new_v4()),
            subject: subject.into(),
            source: source.into(),
            timestamp: Utc::now(),
            confidence: 1.0,
            data: serde_json::Value::Null,
            provenance: provenance.into(),
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}
