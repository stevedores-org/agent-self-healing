use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureKind {
    SilentAgent,
    CrashedAgent,
    CrashLoop,
    LeaseExpired,
    SchedulerLag,
    ResourceExhaustion,
    CheckpointCorruption,
    GraphNodeFailure,
    DependencyFailure,
    PolicyBlocked,
    RepeatedFailure,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub incident_id: String,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub status: String,
    pub failure_kind: FailureKind,
    pub severity: String,
    pub idempotency_key: String,
}
