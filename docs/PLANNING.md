# Planning: Agent Self-Healing

## Backlog Slice

### E0: Repo scaffold and local CI
**User Story:** As a contributor, I want the repository fully scaffolded with Nix and Cargo so that I can immediately run CI tests locally without fighting environment setup.
**Acceptance Criteria:**
- Cargo workspace with all 9 crates configured.
- Nix flakes set up with default development environment.
- CI commands (`cargo test`, `cargo clippy`) pass.
- `/healthz` endpoint stub returns 200.
**Test Names:**
- `test_workspace_builds`
- `test_flake_check_exposes_default_package`
- `test_server_healthz_returns_ok`

### E1: Core health/failure/recovery domain model
**User Story:** As the recovery engine, I need a strict internal domain model so that I can classify failures and plan actions predictably without side effects.
**Acceptance Criteria:**
- `HealthSignal`, `Incident`, `FailureKind`, `HealingAction`, `RecoveryPlan` modeled as structs/enums.
- Deterministic classification logic.
- Plan generation requiring an idempotency key.
**Test Names:**
- `classifies_missed_heartbeat_as_silent_agent`
- `classifies_oom_killed_pod_as_resource_exhaustion`
- `recovery_plan_requires_idempotency_key`
- `rollback_action_requires_verified_checkpoint`

### E2: SurrealDB persistence and migrations
**User Story:** As the self-healing supervisor, I need to persist incidents and checkpoints in SurrealDB to ensure I don't lose state on crash.
**Acceptance Criteria:**
- SurrealDB schema defined for `health_signal`, `checkpoint`, `incident`, and `recovery_action`.
- Atomic inserts and updates of incident actions.
**Test Names:**
- `inserts_health_signal`
- `creates_incident_once_by_idempotency_key`
- `latest_verified_checkpoint_wins`
- `corrupt_checkpoint_is_never_selected`

### E3: Scheduler adapter for AgentRun/AgentTask/AgentLease
**User Story:** As the self-healing system, I need to observe the Kubernetes `agent-scheduler` state to understand when leases expire or tasks are abandoned.
**Acceptance Criteria:**
- Watch/read implementation for `AgentRun`, `AgentTask`, and `AgentLease` CRDs.
- Proper mapping of scheduler status into `SchedulerObservation` structs.
**Test Names:**
- `maps_agentrun_running_with_active_job_to_active_agent`
- `maps_agenttask_claimed_with_expired_lease_to_scheduler_owned_reclaim`

### E4: Recovery planner, backoff, circuit breaker
**User Story:** As a robust supervisor, I want to backoff retries and open a circuit breaker when failures cascade so that I don't crash loop the entire cluster.
**Acceptance Criteria:**
- Exponential backoff implementation for actions.
- Circuit breaker trips when too many restarts fail within a time window.
**Test Names:**
- `exponential_backoff_is_monotonic_until_cap`
- `circuit_breaker_opens_after_repeated_failures`
- `planner_prefers_scheduler_owned_reclaim_for_expired_task_lease`

### E5: Checkpoint manager and rollback planning
**User Story:** As a resilient agent, I want to rollback to verified state hashes when data is poisoned so I can safely resume execution.
**Acceptance Criteria:**
- Support for verifying checkpoints via a state hash.
- Support for selecting a known good checkpoint.
**Test Names:**
- `writes_checkpoint_with_state_hash`
- `selects_latest_good_checkpoint_for_agent_and_node`
- `rollback_plan_contains_snapshot_ref_and_parent_checkpoint`

### E6: Graph-aware cascade safety
**User Story:** As a graph execution orchestrator, I want to pause downstream nodes when a parent node rolls back so that errors do not propagate.
**Acceptance Criteria:**
- Dependency checking mechanisms.
- Downstream topological querying of the graph to halt dependents.
**Test Names:**
- `downstream_nodes_are_paused_when_parent_checkpoint_rolls_back`
- `sibling_nodes_continue_when_failure_is_isolated`

### E7: Brains policy/evidence/experience integration
**User Story:** As an intelligent supervisor, I need to consult Brains for policy gating on destructive actions and emit experiences so the swarm can learn from the failure.
**Acceptance Criteria:**
- API integration points for `check_policy` and `store_memory`.
- Trace generation for `incident_opened`.
**Test Names:**
- `emits_brains_trace_for_incident_opened`
- `policy_denial_blocks_recovery_action`
- `stores_experience_after_successful_recovery`

### E8: MOM notifications and operator summaries
**User Story:** As a human operator, I need Slack/MOM notifications summarizing recovery actions so I know when my agents are self-healing.
**Acceptance Criteria:**
- Output handler integration via HTTP for MOM.
**Test Names:**
- `mom_receives_human_readable_recovery_summary`

### E9: Kubernetes deployment and RBAC
**User Story:** As a DevOps engineer, I want Kustomize overlays and Kubernetes manifests ready to deploy to GKE so I can manage the component with Flux.
**Acceptance Criteria:**
- Base manifests for Deployment, RBAC (ServiceAccount, Roles), and Service.
- Production and Staging overlays.

### E10: E2E chaos/conformance suite
**User Story:** As a Stevedores developer, I want e2e tests running locally that simulate OOM kills and missed heartbeats so that I can be confident the recovery engine works end-to-end.
**Acceptance Criteria:**
- Complete E2E harnesses utilizing local SurrealDB, fake MOM, and a mock CRD scheduler.
**Test Names:**
- `e2e_crashed_agent_restarts_from_checkpoint`
- `e2e_expired_lease_is_observed_then_reclaimed_by_scheduler`
