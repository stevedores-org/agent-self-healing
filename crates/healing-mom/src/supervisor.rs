use healing_core::action::{HealingAction, RecoveryPlan};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SupervisorError {
    #[error("Action execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Action skipped due to unmet preconditions")]
    PreconditionsUnmet,
}

pub trait ActionExecutor: Debug {
    fn execute(&self, action: &HealingAction) -> Result<(), SupervisorError>;
}

#[derive(Debug)]
pub struct RecoverySupervisor<E: ActionExecutor> {
    executor: E,
}

impl<E: ActionExecutor> RecoverySupervisor<E> {
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    pub fn execute_plan(&self, plan: &RecoveryPlan) -> Result<(), SupervisorError> {
        for action in &plan.actions {
            self.executor.execute(&action.action)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use healing_core::action::PlannedAction;
    use healing_core::policy::PolicyDecision;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    struct MockExecutor {
        executed_actions: RefCell<Vec<HealingAction>>,
    }

    impl ActionExecutor for MockExecutor {
        fn execute(&self, action: &HealingAction) -> Result<(), SupervisorError> {
            self.executed_actions.borrow_mut().push(action.clone());
            Ok(())
        }
    }

    #[test]
    fn supervisor_executes_plan_sequentially() {
        let plan = RecoveryPlan::new(
            "idemp-1".to_string(),
            "incident-123".to_string(),
            vec![
                PlannedAction {
                    action: HealingAction::RestartAgent,
                    preconditions: vec![],
                    policy_decision: Some(PolicyDecision {
                        decision_id: "pol-1".to_string(),
                        is_approved: true,
                        rationale: "test".to_string(),
                    }),
                    evidence_refs: vec![],
                    attempt_number: 1,
                    max_attempts: 3,
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
        );

        let executor = MockExecutor::default();
        let supervisor = RecoverySupervisor::new(executor);

        let result = supervisor.execute_plan(&plan);
        assert!(result.is_ok());

        let executed = supervisor.executor.executed_actions.borrow();
        assert_eq!(executed.len(), 2);
        assert_eq!(executed[0], HealingAction::RestartAgent);
        assert_eq!(executed[1], HealingAction::ObserveOnly);
    }
}
