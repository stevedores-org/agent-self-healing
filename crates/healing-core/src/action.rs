use crate::checkpoint::CheckpointRef;
use crate::policy::PolicyDecision;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealingAction {
    Noop,
    ObserveOnly,
    RequeueTask,
    RestartAgent,
    RestartFromCheckpoint(CheckpointRef),
    RollbackToCheckpoint(CheckpointRef),
    PauseDependents,
    ResumeDependents,
    MarkFailed,
    EscalateToHuman,
    EmitExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub idempotency_key: String,
    pub incident_id: String,
    pub actions: Vec<PlannedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub action: HealingAction,
    pub preconditions: Vec<String>,
    pub policy_decision: Option<PolicyDecision>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub attempt_number: u32,
    pub max_attempts: u32,
}

impl RecoveryPlan {
    pub fn new(idempotency_key: String, incident_id: String, actions: Vec<PlannedAction>) -> Self {
        assert!(!idempotency_key.is_empty(), "Idempotency key is required");

        for pa in &actions {
            if let HealingAction::RollbackToCheckpoint(chk) = &pa.action {
                assert!(chk.is_verified, "Rollback requires a verified checkpoint");
            }
            if matches!(
                pa.action,
                HealingAction::RestartAgent
                    | HealingAction::RollbackToCheckpoint(_)
                    | HealingAction::MarkFailed
            ) {
                assert!(
                    pa.policy_decision.is_some()
                        && pa.policy_decision.as_ref().unwrap().is_approved,
                    "Destructive actions require policy decision"
                );
            }
        }

        Self {
            idempotency_key,
            incident_id,
            actions,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::CheckpointRef;

    #[test]
    #[should_panic(expected = "Idempotency key is required")]
    fn recovery_plan_requires_idempotency_key() {
        RecoveryPlan::new("".to_string(), "incident-123".to_string(), vec![]);
    }

    #[test]
    #[should_panic(expected = "Rollback requires a verified checkpoint")]
    fn rollback_action_requires_verified_checkpoint() {
        let chk = CheckpointRef {
            checkpoint_id: "chk-1".to_string(),
            is_verified: false,
        };

        RecoveryPlan::new(
            "idemp-1".to_string(),
            "incident-123".to_string(),
            vec![PlannedAction {
                action: HealingAction::RollbackToCheckpoint(chk),
                preconditions: vec![],
                policy_decision: None,
                evidence_refs: vec![],
                attempt_number: 1,
                max_attempts: 3,
            }],
        );
    }

    #[test]
    #[should_panic(expected = "Destructive actions require policy decision")]
    fn destructive_action_requires_policy_decision() {
        RecoveryPlan::new(
            "idemp-1".to_string(),
            "incident-123".to_string(),
            vec![PlannedAction {
                action: HealingAction::RestartAgent,
                preconditions: vec![],
                policy_decision: None,
                evidence_refs: vec![],
                attempt_number: 1,
                max_attempts: 3,
            }],
        );
    }
}
