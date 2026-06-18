use healing_core::incident::FailureKind;
use healing_core::signal::HealthSignal;

pub struct Classifier;

impl Classifier {
    pub fn classify(signal: &HealthSignal) -> FailureKind {
        match signal.kind.as_str() {
            "HeartbeatMissed" => FailureKind::SilentAgent,
            "PodExited" => {
                if let Some(reason) = signal.payload.get("reason") {
                    if reason == "OOMKilled" {
                        return FailureKind::ResourceExhaustion;
                    }
                }
                FailureKind::CrashedAgent
            }
            "CrashLoopBackOff" => FailureKind::CrashLoop,
            "LeaseExpired" => FailureKind::LeaseExpired,
            "CheckpointCorrupt" => FailureKind::CheckpointCorruption,
            "DependencyFailed" => FailureKind::DependencyFailure,
            "PolicyBlocked" => FailureKind::PolicyBlocked,
            _ => FailureKind::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classifies_missed_heartbeat_as_silent_agent() {
        let signal = HealthSignal {
            signal_id: "1".into(),
            observed_at: "now".into(),
            source: "monitor".into(),
            agent_ref: None,
            run_ref: None,
            task_ref: None,
            graph_node_ref: None,
            kind: "HeartbeatMissed".into(),
            severity: "High".into(),
            payload: json!({}),
            trace_id: None,
        };

        assert_eq!(Classifier::classify(&signal), FailureKind::SilentAgent);
    }

    #[test]
    fn classifies_oom_killed_pod_as_resource_exhaustion() {
        let signal = HealthSignal {
            signal_id: "2".into(),
            observed_at: "now".into(),
            source: "k8s".into(),
            agent_ref: None,
            run_ref: None,
            task_ref: None,
            graph_node_ref: None,
            kind: "PodExited".into(),
            severity: "High".into(),
            payload: json!({"reason": "OOMKilled"}),
            trace_id: None,
        };

        assert_eq!(
            Classifier::classify(&signal),
            FailureKind::ResourceExhaustion
        );
    }
}
