use healing_core::action::{HealingAction, PlannedAction, RecoveryPlan};
use healing_core::incident::{FailureKind, Incident};
use healing_core::policy::PolicyDecision;

pub struct HealingPlanner;

impl HealingPlanner {
    pub fn plan_recovery(incident: &Incident) -> RecoveryPlan {
        let idempotency_key = format!("plan-{}", incident.incident_id);

        let actions = match incident.failure_kind {
            FailureKind::SilentAgent => vec![
                PlannedAction {
                    action: HealingAction::RestartAgent,
                    preconditions: vec![],
                    policy_decision: Some(PolicyDecision {
                        decision_id: "default-restart".to_string(),
                        is_approved: true,
                        rationale: "SilentAgent requires restart".to_string(),
                    }),
                    evidence_refs: vec![],
                    attempt_number: 1,
                    max_attempts: 1,
                },
                PlannedAction {
                    action: HealingAction::ObserveOnly,
                    preconditions: vec![],
                    policy_decision: None,
                    evidence_refs: vec![],
                    attempt_number: 1,
                    max_attempts: 1,
                },
            ],
            FailureKind::ResourceExhaustion => vec![PlannedAction {
                action: HealingAction::RestartAgent, // Need more memory? Let's just restart for now.
                preconditions: vec![],
                policy_decision: Some(PolicyDecision {
                    decision_id: "resource-restart".to_string(),
                    is_approved: true,
                    rationale: "ResourceExhaustion implies transient limit hit, restart"
                        .to_string(),
                }),
                evidence_refs: vec![],
                attempt_number: 1,
                max_attempts: 1,
            }],
            _ => vec![PlannedAction {
                action: HealingAction::EscalateToHuman,
                preconditions: vec![],
                policy_decision: None,
                evidence_refs: vec![],
                attempt_number: 1,
                max_attempts: 1,
            }],
        };

        RecoveryPlan::new(idempotency_key, incident.incident_id.clone(), actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_restart_for_silent_agent() {
        let incident = Incident {
            incident_id: "inc-1".to_string(),
            opened_at: "now".to_string(),
            closed_at: None,
            status: "open".to_string(),
            failure_kind: FailureKind::SilentAgent,
            severity: "high".to_string(),
            idempotency_key: "idem-1".to_string(),
        };

        let plan = HealingPlanner::plan_recovery(&incident);
        assert_eq!(plan.actions.len(), 2);
        assert_eq!(plan.actions[0].action, HealingAction::RestartAgent);
        assert_eq!(plan.actions[1].action, HealingAction::ObserveOnly);
    }

    #[test]
    fn plans_escalation_for_unknown() {
        let incident = Incident {
            incident_id: "inc-2".to_string(),
            opened_at: "now".to_string(),
            closed_at: None,
            status: "open".to_string(),
            failure_kind: FailureKind::Unknown,
            severity: "high".to_string(),
            idempotency_key: "idem-2".to_string(),
        };

        let plan = HealingPlanner::plan_recovery(&incident);
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].action, HealingAction::EscalateToHuman);
    }
}
