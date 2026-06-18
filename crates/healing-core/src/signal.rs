use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSignal {
    pub signal_id: String,
    pub observed_at: String,
    pub source: String,
    pub agent_ref: Option<String>,
    pub run_ref: Option<String>,
    pub task_ref: Option<String>,
    pub graph_node_ref: Option<String>,
    pub kind: String,
    pub severity: String,
    pub payload: serde_json::Value,
    pub trace_id: Option<String>,
}
