//! Ownership of asynchronous HuggingFace download work.
//!
//! Request futures prepare work, but this module owns every installed Tokio
//! handle. Installation is synchronous and gated so state and task custody can
//! be committed together before work starts. Opaque allocation identities
//! prevent an old task from observing or removing its successor.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::future::Future;
use std::panic::AssertUnwindSafe;
#[cfg(test)]
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use futures::FutureExt;
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;

use crate::model_library::download_recovery::DestinationIdentity;

type FallibleBlockingReceiver<T, E> =
    oneshot::Receiver<std::result::Result<std::result::Result<T, E>, String>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskRole {
    AdmissionTransition,
    RecoveryTransition,
    RelocationTransition,
    Worker,
    CancelFinalizer,
    TerminalProjection,
}

#[derive(Clone)]
pub(super) struct TaskGeneration(Arc<Notify>);

impl PartialEq for TaskGeneration {
    fn eq(&self, other: &Self) -> bool {
        self.matches(other)
    }
}

impl Eq for TaskGeneration {}

impl TaskGeneration {
    fn new() -> Self {
        Self(Arc::new(Notify::new()))
    }

    pub(super) fn wake_pause(&self) {
        self.0.notify_waiters();
    }

    pub(super) fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    fn key(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
}

impl fmt::Debug for TaskGeneration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("TaskGeneration")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}

#[derive(Clone)]
struct DestinationClaim {
    download_id: String,
    domain: DestinationDomain,
    generation: Option<TaskGeneration>,
    ready: Arc<Notify>,
}

impl DestinationClaim {
    fn matches(
        &self,
        download_id: &str,
        domain: DestinationDomain,
        generation: &TaskGeneration,
    ) -> bool {
        self.download_id == download_id
            && self.domain == domain
            && self
                .generation
                .as_ref()
                .is_some_and(|current| current.matches(generation))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(super) enum DestinationDomain {
    Ambient,
    Recovery,
}

#[derive(Default)]
struct DestinationQueue {
    claims: VecDeque<DestinationClaim>,
    // Retained until this runtime owner is dropped. Stale store snapshots may
    // outlive terminal UI state, so absence cannot authorize reconstruction.
    released: HashSet<(String, DestinationDomain)>,
}

/// Generation-scoped, task-owned serialization for filesystem authority at a
/// destination. The mutex protects only queue bookkeeping; no guard survives
/// an await, callback, broadcast, filesystem operation, or wake-up.
/// Display paths never authorize a queue position: callers retain the held
/// destination and supply its equality identity for every lifecycle operation.
#[derive(Default)]
pub(super) struct DestinationExecutionOwner {
    queues: Mutex<HashMap<DestinationIdentity, DestinationQueue>>,
}

impl DestinationExecutionOwner {
    /// A physical legacy-directory move owns both names. The target must have
    /// no incumbent; the source may only be taken by its current first owner.
    pub(super) fn reserve_relocation(
        &self,
        source: DestinationIdentity,
        target: DestinationIdentity,
        download_id: String,
        generation: TaskGeneration,
    ) -> bool {
        if source == target {
            return false;
        }
        let mut queues = self
            .queues
            .lock()
            .expect("HF destination-execution owner lock poisoned");
        if queues
            .get(&target)
            .is_some_and(|queue| !queue.claims.is_empty())
        {
            return false;
        }
        if queues.get(&source).is_some_and(|queue| {
            queue.claims.front().is_some_and(|claim| {
                claim.download_id != download_id || claim.domain != DestinationDomain::Ambient
            }) || queue
                .released
                .contains(&(download_id.clone(), DestinationDomain::Ambient))
        }) {
            return false;
        }
        let queue = queues.entry(source).or_default();
        if let Some(claim) = queue.claims.front_mut() {
            claim.generation = Some(generation.clone());
        } else {
            queue.claims.push_back(DestinationClaim {
                download_id: download_id.clone(),
                domain: DestinationDomain::Ambient,
                generation: Some(generation.clone()),
                ready: Arc::new(Notify::new()),
            });
        }
        let queue = queues.entry(target).or_default();
        queue
            .released
            .remove(&(download_id.clone(), DestinationDomain::Ambient));
        queue.claims.push_back(DestinationClaim {
            download_id,
            domain: DestinationDomain::Ambient,
            generation: Some(generation),
            ready: Arc::new(Notify::new()),
        });
        true
    }
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Reserves a destination position at the state/task commit point. A
    /// repeated reservation for the same download transfers that position to
    /// the successor lifecycle generation without reordering the queue.
    pub(super) fn reserve(
        &self,
        destination: DestinationIdentity,
        download_id: String,
        domain: DestinationDomain,
        generation: TaskGeneration,
    ) -> bool {
        let mut queues = self
            .queues
            .lock()
            .expect("HF destination-execution owner lock poisoned");
        let queue = queues.entry(destination).or_default();
        if queue.released.contains(&(download_id.clone(), domain)) {
            return false;
        }
        if let Some(claim) = queue
            .claims
            .iter_mut()
            .find(|claim| claim.download_id == download_id)
        {
            if claim.domain != domain {
                return false;
            }
            claim.generation = Some(generation);
            return true;
        }
        queue.claims.push_back(DestinationClaim {
            download_id,
            domain,
            generation: Some(generation),
            ready: Arc::new(Notify::new()),
        });
        true
    }

    /// Restores destination authority for resumable state that currently has
    /// no runnable lifecycle generation.
    pub(super) fn reserve_dormant(
        &self,
        destination: DestinationIdentity,
        download_id: String,
        domain: DestinationDomain,
    ) -> bool {
        let mut queues = self
            .queues
            .lock()
            .expect("HF destination-execution owner lock poisoned");
        let queue = queues.entry(destination).or_default();
        if queue.released.contains(&(download_id.clone(), domain)) {
            return false;
        }
        if let Some(claim) = queue
            .claims
            .iter()
            .find(|claim| claim.download_id == download_id)
        {
            return claim.domain == domain;
        }
        queue.claims.push_back(DestinationClaim {
            download_id,
            domain,
            generation: None,
            ready: Arc::new(Notify::new()),
        });
        true
    }

    pub(super) fn promote_domain(
        &self,
        destination: &DestinationIdentity,
        download_id: &str,
        expected: DestinationDomain,
        promoted: DestinationDomain,
        generation: &TaskGeneration,
    ) -> bool {
        let mut queues = self
            .queues
            .lock()
            .expect("HF destination-execution owner lock poisoned");
        let Some(claim) = queues.get_mut(destination).and_then(|queue| {
            queue
                .claims
                .iter_mut()
                .find(|claim| claim.download_id == download_id)
        }) else {
            return false;
        };
        if claim.domain != expected {
            return false;
        }
        claim.domain = promoted;
        claim.generation = Some(generation.clone());
        true
    }

    pub(super) async fn wait_for_turn(
        &self,
        destination: &DestinationIdentity,
        download_id: &str,
        domain: DestinationDomain,
        generation: &TaskGeneration,
    ) -> bool {
        loop {
            let notified = {
                let queues = self
                    .queues
                    .lock()
                    .expect("HF destination-execution owner lock poisoned");
                let Some(queue) = queues.get(destination) else {
                    return false;
                };
                let Some(index) = queue
                    .claims
                    .iter()
                    .position(|claim| claim.download_id == download_id)
                else {
                    return false;
                };
                let claim = &queue.claims[index];
                if !claim.matches(download_id, domain, generation) {
                    return false;
                }
                if index == 0 {
                    return true;
                }
                claim.ready.clone().notified_owned()
            };
            notified.await;
        }
    }

    pub(super) fn release(
        &self,
        destination: &DestinationIdentity,
        download_id: &str,
        domain: DestinationDomain,
        generation: &TaskGeneration,
    ) -> bool {
        match self.release_deferred(destination, download_id, domain, generation) {
            Some(release) => {
                release.wake();
                true
            }
            None => false,
        }
    }

    /// Commits exact queue bookkeeping without signaling under a caller's guard.
    pub(super) fn release_deferred(
        &self,
        destination: &DestinationIdentity,
        download_id: &str,
        domain: DestinationDomain,
        generation: &TaskGeneration,
    ) -> Option<DestinationRelease> {
        let next = {
            let mut queues = self
                .queues
                .lock()
                .expect("HF destination-execution owner lock poisoned");
            let queue = queues.get_mut(destination)?;
            let index = queue
                .claims
                .iter()
                .position(|claim| claim.matches(download_id, domain, generation))?;
            queue.claims.remove(index);
            queue.released.insert((download_id.to_string(), domain));
            let next = (index == 0)
                .then(|| queue.claims.front().map(|claim| claim.ready.clone()))
                .flatten();
            next
        };
        Some(DestinationRelease { next })
    }

    #[cfg(test)]
    pub(super) fn contains(
        &self,
        destination: &DestinationIdentity,
        download_id: &str,
        domain: DestinationDomain,
        generation: &TaskGeneration,
    ) -> bool {
        self.queues
            .lock()
            .expect("HF destination-execution owner lock poisoned")
            .get(destination)
            .is_some_and(|queue| {
                queue
                    .claims
                    .iter()
                    .any(|claim| claim.matches(download_id, domain, generation))
            })
    }

    #[cfg(test)]
    pub(super) fn claim_count(&self, destination: &DestinationIdentity) -> usize {
        self.queues
            .lock()
            .expect("HF destination-execution owner lock poisoned")
            .get(destination)
            .map(|queue| queue.claims.len())
            .unwrap_or(0)
    }
}

pub(super) struct DestinationRelease {
    next: Option<Arc<Notify>>,
}

impl DestinationRelease {
    pub(super) fn wake(self) {
        if let Some(next) = self.next {
            next.notify_waiters();
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TaskSnapshot {
    pub(super) role: TaskRole,
    pub(super) finished: bool,
    pub(super) outer_finished: bool,
    pub(super) started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum TaskStartState {
    Gated = 0,
    Running = 1,
    Abandoned = 2,
}

impl TaskStartState {
    fn load(state: &AtomicU8) -> Self {
        match state.load(Ordering::Acquire) {
            0 => Self::Gated,
            1 => Self::Running,
            _ => Self::Abandoned,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskTerminal {
    Completed,
    Cancelled,
    Panicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TaskObservation {
    pub(super) generation: TaskGeneration,
    pub(super) role: TaskRole,
    pub(super) terminal: TaskTerminal,
    pub(super) nested_failures: usize,
    pub(super) outer_finished_before_replacement: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProjectionOutcome {
    Pending,
    Committed,
    RolledBack,
    Failed,
    Panicked,
    Superseded,
}

#[derive(Debug)]
struct ProjectionCellState {
    predecessor_ready: bool,
    predecessor: Option<TaskObservation>,
    outcome: ProjectionOutcome,
    settled: bool,
    failed: bool,
    failure_projected: bool,
}

#[derive(Debug)]
struct ProjectionCell {
    state: Mutex<ProjectionCellState>,
    inherited: Mutex<Vec<Arc<ProjectionCell>>>,
    notify: Notify,
}

impl ProjectionCell {
    fn new(predecessor_ready: bool) -> Self {
        Self {
            state: Mutex::new(ProjectionCellState {
                predecessor_ready,
                predecessor: None,
                outcome: ProjectionOutcome::Pending,
                settled: false,
                failed: false,
                failure_projected: false,
            }),
            inherited: Mutex::new(Vec::new()),
            notify: Notify::new(),
        }
    }

    fn inherit(&self, cell: Arc<ProjectionCell>) {
        self.inherited
            .lock()
            .expect("HF inherited projection-cell lock poisoned")
            .push(cell);
    }

    fn record_predecessor(&self, observation: TaskObservation) {
        {
            let mut state = self.state.lock().expect("HF projection-cell lock poisoned");
            state.predecessor = Some(observation);
            state.predecessor_ready = true;
        }
        self.notify.notify_waiters();
    }

    async fn wait_for_predecessor(&self) -> Option<TaskObservation> {
        loop {
            let notified = self.notify.notified();
            let ready = {
                let state = self.state.lock().expect("HF projection-cell lock poisoned");
                state.predecessor_ready.then(|| state.predecessor.clone())
            };
            if let Some(predecessor) = ready {
                return predecessor;
            }
            notified.await;
        }
    }

    fn settle(&self, outcome: ProjectionOutcome) {
        {
            let mut state = self.state.lock().expect("HF projection-cell lock poisoned");
            if state.outcome != ProjectionOutcome::Pending {
                if matches!(
                    outcome,
                    ProjectionOutcome::Failed | ProjectionOutcome::Panicked
                ) {
                    state.failed = true;
                }
                return;
            }
            if matches!(
                outcome,
                ProjectionOutcome::Failed | ProjectionOutcome::Panicked
            ) {
                state.failed = true;
            }
            state.outcome = outcome;
        }
        self.notify.notify_waiters();
    }

    fn outcome(&self) -> ProjectionOutcome {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .outcome
    }

    async fn wait(&self) -> ProjectionOutcome {
        loop {
            let notified = self.notify.notified();
            let outcome = self.outcome();
            if outcome != ProjectionOutcome::Pending {
                return outcome;
            }
            notified.await;
        }
    }

    fn mark_settled(&self) {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .settled = true;
    }

    fn is_settled(&self) -> bool {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .settled
    }

    fn mark_failed(&self) {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .failed = true;
    }

    fn acknowledge_failure_projection(&self) {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .failure_projected = true;
        let inherited = self
            .inherited
            .lock()
            .expect("HF inherited projection-cell lock poisoned")
            .clone();
        for cell in inherited {
            cell.acknowledge_failure_projection();
            cell.mark_settled();
        }
        self.notify.notify_waiters();
    }

    fn is_ready_to_settle(&self) -> bool {
        let state = self.state.lock().expect("HF projection-cell lock poisoned");
        state.outcome != ProjectionOutcome::Pending && (!state.failed || state.failure_projected)
    }

    fn has_unprojected_failure(&self) -> bool {
        let state = self.state.lock().expect("HF projection-cell lock poisoned");
        state.outcome != ProjectionOutcome::Pending && state.failed && !state.failure_projected
    }

    #[cfg(test)]
    fn failure_projected(&self) -> bool {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .failure_projected
    }

    fn failed(&self) -> bool {
        self.state
            .lock()
            .expect("HF projection-cell lock poisoned")
            .failed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BlockingTaskError {
    StaleGeneration,
    Join(String),
    ResultChannelClosed,
}

impl fmt::Display for BlockingTaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleGeneration => formatter.write_str("task generation is no longer current"),
            Self::Join(detail) => write!(formatter, "blocking task failed: {detail}"),
            Self::ResultChannelClosed => formatter.write_str("blocking result channel closed"),
        }
    }
}

struct NestedTask {
    handle: JoinHandle<()>,
    completion: Arc<NestedCompletion>,
}

struct NestedCompletion {
    finished: AtomicBool,
    failed: AtomicBool,
    notify: Notify,
}

struct RetiredTask {
    observer: JoinHandle<()>,
}

struct TaskEntry {
    relocation_completion: Option<tokio::sync::watch::Receiver<bool>>,
    admission: Option<(PendingAdmissionIdentity, tokio::sync::watch::Receiver<bool>)>,
    generation: TaskGeneration,
    role: TaskRole,
    outer: JoinHandle<()>,
    nested: Vec<NestedTask>,
    nested_failures_archived: usize,
    projection: Option<Arc<ProjectionCell>>,
    starts: Vec<oneshot::Sender<()>>,
    superseded_projection: Option<Arc<ProjectionCell>>,
    abort_on_start: Vec<tokio::task::AbortHandle>,
    start_state: Arc<AtomicU8>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct PendingAdmissionIdentity {
    pub(super) destination: crate::model_library::download_recovery::DestinationIdentity,
    pub(super) repo_id: String,
    pub(super) files: Vec<(String, Option<u64>, Option<String>)>,
}

struct PreparedEntry {
    download_id: String,
    generation: TaskGeneration,
    role: TaskRole,
    start: oneshot::Sender<()>,
    outer: JoinHandle<()>,
    projection: Option<Arc<ProjectionCell>>,
    start_state: Arc<AtomicU8>,
}

impl TaskEntry {
    fn finished(&self) -> bool {
        self.outer.is_finished() && self.nested.iter().all(|nested| nested.handle.is_finished())
    }

    fn reap_completed_nested(&mut self) {
        let mut retained = Vec::with_capacity(self.nested.len());
        for mut nested in self.nested.drain(..) {
            let observed = if nested.handle.is_finished() {
                (&mut nested.handle).now_or_never()
            } else {
                None
            };
            if let Some(result) = observed {
                self.nested_failures_archived += usize::from(
                    result.is_err() || nested.completion.failed.load(Ordering::Acquire),
                );
            } else {
                retained.push(nested);
            }
        }
        self.nested = retained;
    }
}

#[cfg(test)]
type BlockingObserver = Arc<dyn Fn(&'static str) + Send + Sync>;

#[cfg(test)]
type TaskIdsObserver = Arc<dyn Fn() + Send + Sync>;

#[cfg(test)]
type DrainObserver = Arc<dyn Fn() + Send + Sync>;

#[cfg(test)]
type SnapshotObserver = Arc<dyn Fn(&str, Option<TaskSnapshot>) + Send + Sync>;

#[cfg(test)]
type CancellationCheckObserver = Arc<dyn Fn() + Send + Sync>;

#[cfg(test)]
type CancelReplacementObserver = Arc<dyn Fn() + Send + Sync>;

#[cfg(test)]
type WorkerProjectionObserver = Arc<dyn Fn(&'static str) + Send + Sync>;

#[cfg(test)]
type BlockingResultObserver = Arc<dyn Fn(&'static str) + Send + Sync>;

#[cfg(test)]
type BlockingFailureObserver = Arc<dyn Fn(&'static str) -> bool + Send + Sync>;

#[cfg(test)]
type AmbientAdmissionObserver = Arc<dyn Fn(&'static str, &str) + Send + Sync>;

#[cfg(test)]
type ProjectionObserver = Arc<dyn Fn(&'static str) + Send + Sync>;

#[derive(Default)]
pub(super) struct DownloadTaskOwner {
    tasks: Mutex<HashMap<String, TaskEntry>>,
    prepared: Mutex<HashMap<usize, PreparedEntry>>,
    retired: Mutex<Vec<RetiredTask>>,
    retired_observations: AtomicUsize,
    #[cfg(test)]
    blocking_observer: Mutex<Option<BlockingObserver>>,
    #[cfg(test)]
    ids_observer: Mutex<Option<TaskIdsObserver>>,
    #[cfg(test)]
    drain_observer: Mutex<Option<DrainObserver>>,
    #[cfg(test)]
    snapshot_observer: Mutex<Option<SnapshotObserver>>,
    #[cfg(test)]
    cancellation_check_observer: Mutex<Option<CancellationCheckObserver>>,
    #[cfg(test)]
    cancel_replacement_observer: Mutex<Option<CancelReplacementObserver>>,
    #[cfg(test)]
    worker_projection_observer: Mutex<Option<WorkerProjectionObserver>>,
    #[cfg(test)]
    blocking_result_observer: Mutex<Option<BlockingResultObserver>>,
    #[cfg(test)]
    blocking_failure_observer: Mutex<Option<BlockingFailureObserver>>,
    #[cfg(test)]
    ambient_admission_observer: Mutex<Option<AmbientAdmissionObserver>>,
    #[cfg(test)]
    projection_observer: Mutex<Option<ProjectionObserver>>,
}

impl fmt::Debug for DownloadTaskOwner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self
            .tasks
            .lock()
            .map(|tasks| tasks.len())
            .unwrap_or_default();
        formatter
            .debug_struct("DownloadTaskOwner")
            .field("task_count", &count)
            .finish()
    }
}

pub(super) struct TaskContext {
    owner: Weak<DownloadTaskOwner>,
    download_id: String,
    generation: TaskGeneration,
    projection_failure: Option<Arc<ProjectionCell>>,
}

impl Clone for TaskContext {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            download_id: self.download_id.clone(),
            generation: self.generation.clone(),
            projection_failure: self.projection_failure.clone(),
        }
    }
}

pub(super) struct PreparedTask {
    owner: Weak<DownloadTaskOwner>,
    download_id: String,
    generation: TaskGeneration,
    role: TaskRole,
    projection: Option<Arc<ProjectionCell>>,
    start_state: Arc<AtomicU8>,
    armed: bool,
}

impl fmt::Debug for PreparedTask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedTask")
            .field("download_id", &self.download_id)
            .field("generation", &self.generation)
            .field("role", &self.role)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(super) struct InstalledTask {
    owner: Arc<DownloadTaskOwner>,
    download_id: String,
    generation: TaskGeneration,
    start_state: Arc<AtomicU8>,
}

#[derive(Debug)]
pub(super) struct PreparedProjection {
    task: PreparedTask,
    cell: Arc<ProjectionCell>,
}

pub(super) struct InstalledProjection {
    task: InstalledTask,
    ticket: ProjectionTicket,
}

#[derive(Clone)]
pub(super) struct ProjectionTicket {
    download_id: String,
    generation: TaskGeneration,
    cell: Arc<ProjectionCell>,
}

pub(super) enum ProjectionTransition {
    Started(InstalledProjection),
    Existing(InstalledProjection),
    NotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProjectionSettlement {
    Pending,
    FailureUnprojected,
    Settled,
    AlreadySettled,
    StaleGeneration,
    Missing,
}

#[derive(Debug)]
pub(super) enum CancelTransition {
    Relocating(tokio::sync::watch::Receiver<bool>),
    Started(InstalledTask),
    Existing(InstalledTask),
    AlreadyRunning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CancelPredecessor {
    Absent,
    Observed(TaskObservation),
}

impl DownloadTaskOwner {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn prepare<F, Fut>(
        self: &Arc<Self>,
        download_id: String,
        role: TaskRole,
        work: F,
    ) -> PreparedTask
    where
        F: FnOnce(TaskContext) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let generation = TaskGeneration::new();
        let context = TaskContext {
            owner: Arc::downgrade(self),
            download_id: download_id.clone(),
            generation: generation.clone(),
            projection_failure: None,
        };
        let (start, started) = oneshot::channel();
        let outer = tokio::spawn(async move {
            if started.await.is_ok() {
                work(context).await;
            }
        });
        let start_state = Arc::new(AtomicU8::new(TaskStartState::Gated as u8));
        self.prepared
            .lock()
            .expect("HF prepared-task owner lock poisoned")
            .insert(
                generation.key(),
                PreparedEntry {
                    download_id: download_id.clone(),
                    generation: generation.clone(),
                    role,
                    start,
                    outer,
                    projection: None,
                    start_state: start_state.clone(),
                },
            );
        PreparedTask {
            owner: Arc::downgrade(self),
            download_id,
            generation,
            role,
            projection: None,
            start_state,
            armed: true,
        }
    }

    pub(super) fn prepare_projection<F, Fut, P, PFut>(
        self: &Arc<Self>,
        download_id: String,
        project: F,
        project_panic: P,
    ) -> PreparedProjection
    where
        F: FnOnce(TaskContext, Option<TaskObservation>) -> Fut + Send + 'static,
        Fut: Future<Output = ProjectionOutcome> + Send + 'static,
        P: FnOnce(TaskContext) -> PFut + Send + 'static,
        PFut: Future<Output = ProjectionOutcome> + Send + 'static,
    {
        let cell = Arc::new(ProjectionCell::new(true));
        let project_cell = cell.clone();
        let mut task = self.prepare(
            download_id,
            TaskRole::TerminalProjection,
            move |mut context| async move {
                context.projection_failure = Some(project_cell.clone());
                let predecessor = project_cell.wait_for_predecessor().await;
                let project_context = context.clone();
                let outcome =
                    match AssertUnwindSafe(
                        async move { project(project_context, predecessor).await },
                    )
                    .catch_unwind()
                    .await
                    {
                        Ok(outcome) => outcome,
                        Err(_) => {
                            project_cell.mark_failed();
                            let fallback =
                                AssertUnwindSafe(async move { project_panic(context).await })
                                    .catch_unwind()
                                    .await;
                            if matches!(fallback, Ok(ProjectionOutcome::Failed)) {
                                project_cell.acknowledge_failure_projection();
                            }
                            ProjectionOutcome::Panicked
                        }
                    };
                if outcome == ProjectionOutcome::Failed {
                    project_cell.mark_failed();
                }
                if project_cell.failed()
                    && matches!(
                        outcome,
                        ProjectionOutcome::Committed | ProjectionOutcome::Failed
                    )
                {
                    project_cell.acknowledge_failure_projection();
                }
                project_cell.settle(outcome);
            },
        );
        task.projection = Some(cell.clone());
        self.prepared
            .lock()
            .expect("HF prepared-task owner lock poisoned")
            .get_mut(&task.generation.key())
            .expect("prepared projection remains owner-registered")
            .projection = Some(cell.clone());
        PreparedProjection { task, cell }
    }

    /// Installs a prepared task while its start gate remains closed.
    ///
    /// Dropping the returned token only marks its owner-held start lease as
    /// abandoned. Callers rescue it after releasing any outer state guard.
    pub(super) fn install_gated(
        self: &Arc<Self>,
        mut prepared: PreparedTask,
    ) -> std::result::Result<InstalledTask, PreparedTask> {
        if !prepared
            .owner
            .upgrade()
            .is_some_and(|owner| Arc::ptr_eq(&owner, self))
        {
            return Err(prepared);
        }
        let Some(entry) = self
            .prepared
            .lock()
            .expect("HF prepared-task owner lock poisoned")
            .remove(&prepared.generation.key())
        else {
            return Err(prepared);
        };
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        if tasks.contains_key(&prepared.download_id) {
            drop(tasks);
            self.prepared
                .lock()
                .expect("HF prepared-task owner lock poisoned")
                .insert(prepared.generation.key(), entry);
            return Err(prepared);
        }
        let download_id = entry.download_id.clone();
        let generation = entry.generation.clone();
        tasks.insert(
            download_id.clone(),
            TaskEntry {
                admission: None,
                relocation_completion: None,
                generation: generation.clone(),
                role: entry.role,
                outer: entry.outer,
                nested: Vec::new(),
                nested_failures_archived: 0,
                projection: entry.projection,
                starts: vec![entry.start],
                superseded_projection: None,
                abort_on_start: Vec::new(),
                start_state: entry.start_state,
            },
        );
        drop(tasks);
        prepared.armed = false;
        Ok(InstalledTask {
            owner: self.clone(),
            download_id,
            generation,
            start_state: prepared.start_state.clone(),
        })
    }

    pub(super) fn install_projection_gated(
        self: &Arc<Self>,
        prepared: PreparedProjection,
    ) -> std::result::Result<InstalledProjection, PreparedProjection> {
        let cell = prepared.cell.clone();
        match self.install_gated(prepared.task) {
            Ok(task) => {
                let ticket = ProjectionTicket {
                    download_id: task.download_id.clone(),
                    generation: task.generation.clone(),
                    cell,
                };
                Ok(InstalledProjection { task, ticket })
            }
            Err(task) => Err(PreparedProjection { task, cell }),
        }
    }

    pub(super) fn snapshot(&self, download_id: &str) -> Option<TaskSnapshot> {
        let snapshot = self
            .tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .map(|entry| TaskSnapshot {
                role: entry.role,
                finished: entry.finished(),
                outer_finished: entry.outer.is_finished(),
                started: TaskStartState::load(&entry.start_state) == TaskStartState::Running,
            });
        #[cfg(test)]
        let observer = self
            .snapshot_observer
            .lock()
            .expect("HF task snapshot observer lock poisoned")
            .clone();
        #[cfg(test)]
        if let Some(observer) = observer {
            observer(download_id, snapshot.clone());
        }
        snapshot
    }

    pub(super) fn active_worker_generation(&self, download_id: &str) -> Option<TaskGeneration> {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .filter(|entry| {
                entry.role == TaskRole::Worker
                    && TaskStartState::load(&entry.start_state) == TaskStartState::Running
                    && !entry.outer.is_finished()
            })
            .map(|entry| entry.generation.clone())
    }

    #[cfg(test)]
    fn generation_for_test(&self, download_id: &str) -> Option<TaskGeneration> {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .map(|entry| entry.generation.clone())
    }

    #[cfg(test)]
    fn nested_count_for_test(&self, download_id: &str) -> Option<usize> {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .map(|entry| entry.nested.len())
    }

    #[cfg(test)]
    pub(super) fn outer_finished_for_test(&self, download_id: &str) -> bool {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .is_some_and(|entry| entry.outer.is_finished())
    }

    #[cfg(test)]
    fn prepared_count_for_test(&self) -> usize {
        self.prepared
            .lock()
            .expect("HF prepared-task owner lock poisoned")
            .len()
    }

    pub(super) fn contains(&self, download_id: &str) -> bool {
        self.snapshot(download_id).is_some()
    }

    pub(super) fn generation_is_current(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
    ) -> bool {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .is_some_and(|entry| entry.generation.matches(generation))
    }

    fn generation_has_role(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
        role: TaskRole,
    ) -> bool {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .is_some_and(|entry| entry.generation.matches(generation) && entry.role == role)
    }

    /// Called under the download-state commit lock after durable confirmation.
    pub(super) fn promote_admission(&self, download_id: &str, generation: &TaskGeneration) -> bool {
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        let Some(entry) = tasks.get_mut(download_id) else {
            return false;
        };
        if !entry.generation.matches(generation) || entry.role != TaskRole::AdmissionTransition {
            return false;
        }
        entry.role = TaskRole::Worker;
        true
    }

    pub(super) fn bind_pending_admission(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
        identity: PendingAdmissionIdentity,
        completed: tokio::sync::watch::Receiver<bool>,
    ) {
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        if let Some(entry) = tasks
            .get_mut(download_id)
            .filter(|entry| entry.generation.matches(generation))
        {
            entry.admission = Some((identity, completed));
        }
    }

    pub(super) fn bind_relocation_completion(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
        completion: tokio::sync::watch::Receiver<bool>,
    ) -> bool {
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        let Some(entry) = tasks.get_mut(download_id) else {
            return false;
        };
        if entry.role != TaskRole::RelocationTransition || !entry.generation.matches(generation) {
            return false;
        }
        entry.relocation_completion = Some(completion);
        true
    }

    pub(super) fn relocation_completion(
        &self,
        download_id: &str,
    ) -> Option<tokio::sync::watch::Receiver<bool>> {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .get(download_id)
            .filter(|entry| entry.role == TaskRole::RelocationTransition)
            .and_then(|entry| entry.relocation_completion.clone())
    }

    pub(super) fn pending_admission(
        &self,
        identity: &PendingAdmissionIdentity,
    ) -> Option<(String, tokio::sync::watch::Receiver<bool>)> {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .iter()
            .find_map(|(id, entry)| {
                if entry.role != TaskRole::AdmissionTransition {
                    return None;
                }
                entry
                    .admission
                    .as_ref()
                    .filter(|(key, _)| key == identity)
                    .map(|(_, completion)| (id.clone(), completion.clone()))
            })
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .is_empty()
    }

    pub(super) fn ids(&self) -> Vec<String> {
        let ids = self
            .tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .keys()
            .cloned()
            .collect();
        #[cfg(test)]
        let observer = self
            .ids_observer
            .lock()
            .expect("HF task IDs observer lock poisoned")
            .clone();
        #[cfg(test)]
        if let Some(observer) = observer {
            observer();
        }
        ids
    }

    /// Starts a caller-independent finalizer after synchronously replacing and
    /// aborting the current generation. The finalizer observes the outer task
    /// and all registered blocking work before it runs `finish`.
    pub(super) fn begin_cancel<F, Fut>(
        self: &Arc<Self>,
        download_id: &str,
        finish: F,
    ) -> CancelTransition
    where
        F: FnOnce(TaskContext, CancelPredecessor) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        #[cfg(test)]
        {
            let observer = self
                .cancel_replacement_observer
                .lock()
                .expect("HF cancel-replacement observer lock poisoned")
                .clone();
            if let Some(observer) = observer {
                observer();
            }
        }
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        if let Some(current) = tasks.get_mut(download_id) {
            if current.role == TaskRole::RelocationTransition {
                if let Some(completion) = current.relocation_completion.as_ref() {
                    if !*completion.borrow() {
                        return CancelTransition::Relocating(completion.clone());
                    }
                }
            }
            if current.role == TaskRole::CancelFinalizer && !current.finished() {
                return if TaskStartState::load(&current.start_state) == TaskStartState::Running {
                    CancelTransition::AlreadyRunning
                } else {
                    CancelTransition::Existing(InstalledTask {
                        owner: self.clone(),
                        download_id: download_id.to_string(),
                        generation: current.generation.clone(),
                        start_state: current.start_state.clone(),
                    })
                };
            }
        }
        let mut current = tasks.remove(download_id);
        let outer_finished_before_replacement = current
            .as_ref()
            .is_some_and(|entry| entry.outer.is_finished());
        let abort_on_start = current
            .as_ref()
            .filter(|entry| entry.role != TaskRole::RelocationTransition)
            .map(|entry| entry.outer.abort_handle())
            .into_iter()
            .collect();
        let superseded_projection = current.as_ref().and_then(|entry| {
            entry
                .projection
                .clone()
                .or_else(|| entry.superseded_projection.clone())
        });
        let predecessor_starts = current
            .as_mut()
            .map(|entry| std::mem::take(&mut entry.starts))
            .unwrap_or_default();

        let generation = TaskGeneration::new();
        let context = TaskContext {
            owner: Arc::downgrade(self),
            download_id: download_id.to_string(),
            generation: generation.clone(),
            projection_failure: superseded_projection.clone(),
        };
        let (start, started) = oneshot::channel();
        let start_state = Arc::new(AtomicU8::new(TaskStartState::Gated as u8));
        let outer = tokio::spawn(async move {
            if started.await.is_err() {
                return;
            }
            let predecessor = match current {
                Some(current) => {
                    let role = current.role;
                    CancelPredecessor::Observed(
                        observe_entry(current, role, outer_finished_before_replacement).await,
                    )
                }
                None => CancelPredecessor::Absent,
            };
            finish(context, predecessor).await;
        });
        tasks.insert(
            download_id.to_string(),
            TaskEntry {
                admission: None,
                relocation_completion: None,
                generation: generation.clone(),
                role: TaskRole::CancelFinalizer,
                outer,
                nested: Vec::new(),
                nested_failures_archived: 0,
                projection: None,
                starts: predecessor_starts
                    .into_iter()
                    .chain(std::iter::once(start))
                    .collect(),
                superseded_projection,
                abort_on_start,
                start_state: start_state.clone(),
            },
        );
        drop(tasks);
        CancelTransition::Started(InstalledTask {
            owner: self.clone(),
            download_id: download_id.to_string(),
            generation,
            start_state,
        })
    }

    #[cfg(test)]
    pub(super) async fn observe_finished(&self, download_id: &str) -> Option<TaskObservation> {
        let entry = {
            let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
            if !tasks.get(download_id).is_some_and(TaskEntry::finished) {
                return None;
            }
            tasks.remove(download_id)
        }?;
        let role = entry.role;
        Some(observe_entry(entry, role, false).await)
    }

    pub(super) async fn observe_finished_generation(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
    ) -> Option<TaskObservation> {
        let entry = {
            let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
            if !tasks
                .get(download_id)
                .is_some_and(|entry| entry.generation.matches(generation) && entry.finished())
            {
                return None;
            }
            tasks.remove(download_id)
        }?;
        let role = entry.role;
        Some(observe_entry(entry, role, false).await)
    }

    pub(super) fn finished_or_projecting_ids(&self) -> Vec<String> {
        self.tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .iter()
            .filter_map(|(download_id, entry)| {
                (entry.role == TaskRole::TerminalProjection || entry.finished())
                    .then_some(download_id.clone())
            })
            .collect()
    }

    /// Replaces one fully finished generation with a start-gated projection
    /// owner under the same download ID. The predecessor is observed by a
    /// nested owner task, so its failure remains visible if cancellation
    /// supersedes the projector before state projection begins.
    pub(super) fn begin_finished_projection<F, Fut, P, PFut>(
        self: &Arc<Self>,
        download_id: &str,
        inherit_failure: bool,
        project: F,
        project_panic: P,
    ) -> ProjectionTransition
    where
        F: FnOnce(TaskContext, Option<TaskObservation>) -> Fut + Send + 'static,
        Fut: Future<Output = ProjectionOutcome> + Send + 'static,
        P: FnOnce(TaskContext) -> PFut + Send + 'static,
        PFut: Future<Output = ProjectionOutcome> + Send + 'static,
    {
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        let Some(current) = tasks.get_mut(download_id) else {
            return ProjectionTransition::NotReady;
        };
        if current.role == TaskRole::TerminalProjection {
            let cell = current
                .projection
                .clone()
                .expect("terminal projection owns a projection cell");
            let generation = current.generation.clone();
            return ProjectionTransition::Existing(InstalledProjection {
                task: InstalledTask {
                    owner: self.clone(),
                    download_id: download_id.to_string(),
                    generation: generation.clone(),
                    start_state: current.start_state.clone(),
                },
                ticket: ProjectionTicket {
                    download_id: download_id.to_string(),
                    generation,
                    cell,
                },
            });
        }
        if !current.finished() {
            return ProjectionTransition::NotReady;
        }

        let predecessor = tasks
            .remove(download_id)
            .expect("finished predecessor remained present");
        let predecessor_role = predecessor.role;
        let inherited_projection = predecessor.superseded_projection.clone();
        let generation = TaskGeneration::new();
        let cell = Arc::new(ProjectionCell::new(false));
        if let Some(inherited) = inherited_projection {
            if inherited.failed() {
                cell.mark_failed();
            }
            cell.inherit(inherited);
        }
        if inherit_failure {
            cell.mark_failed();
        }
        let context = TaskContext {
            owner: Arc::downgrade(self),
            download_id: download_id.to_string(),
            generation: generation.clone(),
            projection_failure: Some(cell.clone()),
        };
        let start_state = Arc::new(AtomicU8::new(TaskStartState::Gated as u8));

        let predecessor_cell = cell.clone();
        let predecessor_completion = Arc::new(NestedCompletion {
            finished: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            notify: Notify::new(),
        });
        let predecessor_completion_task = predecessor_completion.clone();
        let (predecessor_start, predecessor_started) = oneshot::channel();
        let predecessor_observer = tokio::spawn(async move {
            if predecessor_started.await.is_err() {
                return;
            }
            let observation = observe_entry(predecessor, predecessor_role, false).await;
            if observation.terminal == TaskTerminal::Panicked || observation.nested_failures > 0 {
                predecessor_completion_task
                    .failed
                    .store(true, Ordering::Release);
            }
            predecessor_cell.record_predecessor(observation);
            predecessor_completion_task
                .finished
                .store(true, Ordering::Release);
            predecessor_completion_task.notify.notify_waiters();
        });

        let project_cell = cell.clone();
        let (project_start, project_started) = oneshot::channel();
        let outer = tokio::spawn(async move {
            if project_started.await.is_err() {
                return;
            }
            let predecessor = project_cell.wait_for_predecessor().await;
            let project_context = context.clone();
            let outcome =
                match AssertUnwindSafe(async move { project(project_context, predecessor).await })
                    .catch_unwind()
                    .await
                {
                    Ok(outcome) => outcome,
                    Err(_) => {
                        project_cell.mark_failed();
                        let fallback =
                            AssertUnwindSafe(async move { project_panic(context).await })
                                .catch_unwind()
                                .await;
                        if matches!(fallback, Ok(ProjectionOutcome::Failed)) {
                            project_cell.acknowledge_failure_projection();
                        }
                        ProjectionOutcome::Panicked
                    }
                };
            if outcome == ProjectionOutcome::Failed {
                project_cell.mark_failed();
            }
            if project_cell.failed()
                && matches!(
                    outcome,
                    ProjectionOutcome::Committed | ProjectionOutcome::Failed
                )
            {
                project_cell.acknowledge_failure_projection();
            }
            project_cell.settle(outcome);
        });

        tasks.insert(
            download_id.to_string(),
            TaskEntry {
                admission: None,
                relocation_completion: None,
                generation: generation.clone(),
                role: TaskRole::TerminalProjection,
                outer,
                nested: vec![NestedTask {
                    handle: predecessor_observer,
                    completion: predecessor_completion,
                }],
                nested_failures_archived: 0,
                projection: Some(cell.clone()),
                starts: vec![predecessor_start, project_start],
                superseded_projection: None,
                abort_on_start: Vec::new(),
                start_state: start_state.clone(),
            },
        );
        drop(tasks);

        let ticket = ProjectionTicket {
            download_id: download_id.to_string(),
            generation: generation.clone(),
            cell,
        };
        ProjectionTransition::Started(InstalledProjection {
            task: InstalledTask {
                owner: self.clone(),
                download_id: download_id.to_string(),
                generation,
                start_state,
            },
            ticket,
        })
    }

    pub(super) fn settle_projection(&self, ticket: &ProjectionTicket) -> ProjectionSettlement {
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        let Some(entry) = tasks.get(&ticket.download_id) else {
            return if ticket.cell.is_settled() {
                ProjectionSettlement::AlreadySettled
            } else {
                ProjectionSettlement::Missing
            };
        };
        let matches = entry.role == TaskRole::TerminalProjection
            && entry.generation.matches(&ticket.generation);
        if !matches {
            return ProjectionSettlement::StaleGeneration;
        }
        if ticket.cell.has_unprojected_failure() {
            return ProjectionSettlement::FailureUnprojected;
        }
        if !ticket.cell.is_ready_to_settle() {
            return ProjectionSettlement::Pending;
        }
        if !entry.finished() {
            return ProjectionSettlement::Pending;
        }
        ticket.cell.mark_settled();
        let _ = tasks.remove(&ticket.download_id);
        ProjectionSettlement::Settled
    }

    /// Requests shutdown without claiming that task or filesystem work has
    /// already drained. Full client shutdown observation belongs to M4.
    pub(super) fn abort_all(&self) {
        let tasks = self
            .tasks
            .lock()
            .expect("HF task owner lock poisoned")
            .drain()
            .map(|(_, entry)| entry)
            .collect::<Vec<_>>();
        for entry in tasks {
            entry.outer.abort();
            for nested in entry.nested {
                nested.handle.abort();
            }
        }
    }

    fn promote_generation(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
        role: TaskRole,
    ) -> bool {
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        let Some(entry) = tasks.get_mut(download_id) else {
            return false;
        };
        if !entry.generation.matches(generation) {
            return false;
        }
        entry.role = role;
        true
    }

    fn start_generation(&self, download_id: &str, generation: &TaskGeneration) -> bool {
        let (aborts, superseded_projection, starts) = {
            let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
            let Some(entry) = tasks.get_mut(download_id) else {
                return false;
            };
            if !entry.generation.matches(generation) {
                return false;
            }
            entry
                .start_state
                .store(TaskStartState::Running as u8, Ordering::Release);
            (
                std::mem::take(&mut entry.abort_on_start),
                entry.superseded_projection.clone(),
                std::mem::take(&mut entry.starts),
            )
        };
        for abort in aborts {
            abort.abort();
        }
        if let Some(cell) = superseded_projection {
            cell.settle(ProjectionOutcome::Superseded);
        }
        for start in starts {
            let _ = start.send(());
        }
        true
    }

    /// Runs after outer state locks are released. Abandoned projectors and
    /// finalizers are safe to start because they retain required predecessor
    /// custody; abandoned workers are removed and aborted without claiming
    /// that they ever ran.
    pub(super) fn rescue_abandoned(&self) {
        self.reap_retired();
        let abandoned_prepared = {
            let mut prepared = self
                .prepared
                .lock()
                .expect("HF prepared-task owner lock poisoned");
            let abandoned = prepared
                .iter()
                .filter_map(|(key, entry)| {
                    (TaskStartState::load(&entry.start_state) == TaskStartState::Abandoned)
                        .then_some(*key)
                })
                .collect::<Vec<_>>();
            abandoned
                .into_iter()
                .filter_map(|key| prepared.remove(&key))
                .collect::<Vec<_>>()
        };
        for entry in abandoned_prepared {
            let abort = entry.outer.abort_handle();
            abort.abort();
            drop(entry.start);
            self.retain_retired(entry.outer, Vec::new());
        }
        let (to_start, to_abort) = {
            let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
            let mut to_start = Vec::new();
            let mut abort_ids = Vec::new();
            for (download_id, entry) in tasks.iter_mut() {
                if TaskStartState::load(&entry.start_state) != TaskStartState::Abandoned {
                    continue;
                }
                if matches!(
                    entry.role,
                    TaskRole::TerminalProjection | TaskRole::CancelFinalizer
                ) {
                    entry
                        .start_state
                        .store(TaskStartState::Running as u8, Ordering::Release);
                    to_start.push((
                        std::mem::take(&mut entry.abort_on_start),
                        entry.superseded_projection.clone(),
                        std::mem::take(&mut entry.starts),
                    ));
                } else {
                    abort_ids.push(download_id.clone());
                }
            }
            let to_abort = abort_ids
                .into_iter()
                .filter_map(|download_id| tasks.remove(&download_id))
                .collect::<Vec<_>>();
            (to_start, to_abort)
        };
        for (aborts, superseded_projection, starts) in to_start {
            for abort in aborts {
                abort.abort();
            }
            if let Some(cell) = superseded_projection {
                cell.settle(ProjectionOutcome::Superseded);
            }
            for start in starts {
                let _ = start.send(());
            }
        }
        for entry in to_abort {
            let TaskEntry {
                outer,
                nested,
                starts,
                ..
            } = entry;
            let outer_abort = outer.abort_handle();
            let nested_aborts = nested
                .iter()
                .map(|nested| nested.handle.abort_handle())
                .collect::<Vec<_>>();
            outer_abort.abort();
            for abort in nested_aborts {
                abort.abort();
            }
            drop(starts);
            self.retain_retired(outer, nested);
        }
    }

    fn retain_retired(&self, outer: JoinHandle<()>, nested: Vec<NestedTask>) {
        let observer = tokio::spawn(async move {
            let _ = outer.await;
            let _ = observe_nested(nested).await;
        });
        self.retired
            .lock()
            .expect("HF retired-task owner lock poisoned")
            .push(RetiredTask { observer });
    }

    fn reap_retired(&self) {
        loop {
            let task = {
                let mut retired = self
                    .retired
                    .lock()
                    .expect("HF retired-task owner lock poisoned");
                retired
                    .iter()
                    .position(|task| task.observer.is_finished())
                    .map(|index| retired.swap_remove(index))
            };
            let Some(mut task) = task else {
                break;
            };
            if (&mut task.observer).now_or_never().is_some() {
                self.retired_observations.fetch_add(1, Ordering::AcqRel);
            } else {
                self.retired
                    .lock()
                    .expect("HF retired-task owner lock poisoned")
                    .push(task);
                break;
            }
        }
    }

    #[cfg(test)]
    pub(super) fn outstanding_retired_for_test(&self) -> usize {
        self.reap_retired();
        self.retired
            .lock()
            .expect("HF retired-task owner lock poisoned")
            .len()
    }

    #[cfg(test)]
    fn retired_observations_for_test(&self) -> usize {
        self.retired_observations.load(Ordering::Acquire)
    }

    fn register_blocking<T, F>(
        self: &Arc<Self>,
        download_id: &str,
        generation: &TaskGeneration,
        operation: &'static str,
        function: F,
    ) -> std::result::Result<oneshot::Receiver<std::result::Result<T, String>>, BlockingTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        self.register_blocking_with_failure(download_id, generation, operation, function, |_| false)
    }

    fn register_fallible_blocking<T, E, F>(
        self: &Arc<Self>,
        download_id: &str,
        generation: &TaskGeneration,
        operation: &'static str,
        function: F,
    ) -> std::result::Result<FallibleBlockingReceiver<T, E>, BlockingTaskError>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: FnOnce() -> std::result::Result<T, E> + Send + 'static,
    {
        self.register_blocking_with_failure(
            download_id,
            generation,
            operation,
            function,
            std::result::Result::is_err,
        )
    }

    fn register_blocking_with_failure<T, F, C>(
        self: &Arc<Self>,
        download_id: &str,
        generation: &TaskGeneration,
        operation: &'static str,
        function: F,
        failed: C,
    ) -> std::result::Result<oneshot::Receiver<std::result::Result<T, String>>, BlockingTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
        C: FnOnce(&T) -> bool + Send + 'static,
    {
        #[cfg(not(test))]
        let _ = operation;
        let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
        let Some(entry) = tasks.get_mut(download_id) else {
            return Err(BlockingTaskError::StaleGeneration);
        };
        if !entry.generation.matches(generation) {
            return Err(BlockingTaskError::StaleGeneration);
        }
        entry.reap_completed_nested();

        #[cfg(test)]
        let blocking_observer = self
            .blocking_observer
            .lock()
            .expect("HF blocking observer lock poisoned")
            .clone();
        let (start_sender, start_receiver) = oneshot::channel();
        let (result_sender, result_receiver) = oneshot::channel();
        let completion = Arc::new(NestedCompletion {
            finished: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            notify: Notify::new(),
        });
        let completion_in_observer = completion.clone();
        #[cfg(test)]
        let result_observer = self
            .blocking_result_observer
            .lock()
            .expect("HF blocking-result observer lock poisoned")
            .clone();
        let observer = tokio::spawn(async move {
            let result = if start_receiver.await.is_ok() {
                tokio::task::spawn_blocking(move || {
                    #[cfg(test)]
                    if let Some(observer) = blocking_observer {
                        observer(operation);
                    }
                    function()
                })
                .await
            } else {
                return;
            }
            .map_err(|error| {
                completion_in_observer.failed.store(true, Ordering::Release);
                error.to_string()
            });
            if result.as_ref().is_ok_and(failed) {
                completion_in_observer.failed.store(true, Ordering::Release);
            }
            #[cfg(test)]
            if let Some(observer) = result_observer {
                observer(operation);
            }
            let _ = result_sender.send(result);
            completion_in_observer
                .finished
                .store(true, Ordering::Release);
            completion_in_observer.notify.notify_waiters();
        });
        entry.nested.push(NestedTask {
            handle: observer,
            completion,
        });
        drop(tasks);
        let _ = start_sender.send(());
        Ok(result_receiver)
    }

    #[cfg(test)]
    pub(super) fn set_blocking_observer(&self, observer: Option<BlockingObserver>) {
        *self
            .blocking_observer
            .lock()
            .expect("HF blocking observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_ids_observer(&self, observer: Option<TaskIdsObserver>) {
        *self
            .ids_observer
            .lock()
            .expect("HF task IDs observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_drain_observer(&self, observer: Option<DrainObserver>) {
        *self
            .drain_observer
            .lock()
            .expect("HF task drain observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_snapshot_observer(&self, observer: Option<SnapshotObserver>) {
        *self
            .snapshot_observer
            .lock()
            .expect("HF task snapshot observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_cancellation_check_observer(
        &self,
        observer: Option<CancellationCheckObserver>,
    ) {
        *self
            .cancellation_check_observer
            .lock()
            .expect("HF cancellation-check observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_cancel_replacement_observer(
        &self,
        observer: Option<CancelReplacementObserver>,
    ) {
        *self
            .cancel_replacement_observer
            .lock()
            .expect("HF cancel-replacement observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_worker_projection_observer(
        &self,
        observer: Option<WorkerProjectionObserver>,
    ) {
        *self
            .worker_projection_observer
            .lock()
            .expect("HF worker-projection observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_blocking_result_observer(&self, observer: Option<BlockingResultObserver>) {
        *self
            .blocking_result_observer
            .lock()
            .expect("HF blocking-result observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_blocking_failure_observer(&self, observer: Option<BlockingFailureObserver>) {
        *self
            .blocking_failure_observer
            .lock()
            .expect("HF blocking-failure observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_ambient_admission_observer(
        &self,
        observer: Option<AmbientAdmissionObserver>,
    ) {
        *self
            .ambient_admission_observer
            .lock()
            .expect("HF ambient-admission observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn set_projection_observer(&self, observer: Option<ProjectionObserver>) {
        *self
            .projection_observer
            .lock()
            .expect("HF projection observer lock poisoned") = observer;
    }

    #[cfg(test)]
    pub(super) fn observe_ambient_admission(&self, operation: &'static str, download_id: &str) {
        let observer = self
            .ambient_admission_observer
            .lock()
            .expect("HF ambient-admission observer lock poisoned")
            .clone();
        if let Some(observer) = observer {
            observer(operation, download_id);
        }
    }

    async fn drain_blocking_generation(
        &self,
        download_id: &str,
        generation: &TaskGeneration,
    ) -> std::result::Result<usize, BlockingTaskError> {
        let (_archived_failures, completions) = {
            let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
            let Some(entry) = tasks.get_mut(download_id) else {
                return Err(BlockingTaskError::StaleGeneration);
            };
            if !entry.generation.matches(generation) {
                return Err(BlockingTaskError::StaleGeneration);
            }
            entry.reap_completed_nested();
            (
                entry.nested_failures_archived,
                entry
                    .nested
                    .iter()
                    .map(|nested| nested.completion.clone())
                    .collect::<Vec<_>>(),
            )
        };
        #[cfg(test)]
        let observer = self
            .drain_observer
            .lock()
            .expect("HF task drain observer lock poisoned")
            .clone();
        #[cfg(test)]
        if let Some(observer) = observer {
            observer();
        }
        let _ = wait_for_nested(&completions).await;
        loop {
            let completed = {
                let mut tasks = self.tasks.lock().expect("HF task owner lock poisoned");
                let Some(entry) = tasks.get_mut(download_id) else {
                    return Err(BlockingTaskError::StaleGeneration);
                };
                if !entry.generation.matches(generation) {
                    return Err(BlockingTaskError::StaleGeneration);
                }
                entry.reap_completed_nested();
                if entry.nested.is_empty() {
                    Some(entry.nested_failures_archived)
                } else {
                    None
                }
            };
            if let Some(failures) = completed {
                return Ok(failures);
            }
            tokio::task::yield_now().await;
        }
    }
}

impl TaskContext {
    pub(super) async fn pause_requested(&self, pause_flag: &AtomicBool) {
        loop {
            let notified = self.generation.0.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if pause_flag.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }

    pub(super) fn download_id(&self) -> &str {
        &self.download_id
    }

    pub(super) fn generation(&self) -> &TaskGeneration {
        &self.generation
    }

    pub(super) fn is_current_role(&self, role: TaskRole) -> bool {
        self.owner.upgrade().is_some_and(|owner| {
            owner.generation_has_role(&self.download_id, &self.generation, role)
        })
    }

    pub(super) fn promote_role(&self, role: TaskRole) -> bool {
        self.owner.upgrade().is_some_and(|owner| {
            owner.promote_generation(&self.download_id, &self.generation, role)
        })
    }

    /// Completes custody transferred from a superseded terminal projector.
    /// A failed cell is acknowledged only after the finalizer has published
    /// its fail-closed terminal state.
    pub(super) fn complete_transferred_projection(&self, failure_projected: bool) -> bool {
        let Some(cell) = &self.projection_failure else {
            return false;
        };
        if cell.failed() {
            if !failure_projected {
                return false;
            }
            cell.acknowledge_failure_projection();
        }
        cell.mark_settled();
        true
    }

    #[cfg(test)]
    pub(super) async fn run_blocking<T, F>(
        &self,
        function: F,
    ) -> std::result::Result<T, BlockingTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        self.run_blocking_named("unnamed", function).await
    }

    #[cfg(test)]
    pub(super) fn register_blocking_without_wait_for_test<T, F>(
        &self,
        operation: &'static str,
        function: F,
    ) -> std::result::Result<(), BlockingTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let owner = self
            .owner
            .upgrade()
            .ok_or(BlockingTaskError::StaleGeneration)?;
        let receiver =
            owner.register_blocking(&self.download_id, &self.generation, operation, function)?;
        drop(receiver);
        Ok(())
    }

    /// Tracks completion and panics when the caller classifies richer domain
    /// outcomes itself (for example definite refusal versus uncertain move).
    /// Such callers must retain custody for any unresolved outcome.
    pub(super) async fn run_blocking_named<T, F>(
        &self,
        operation: &'static str,
        function: F,
    ) -> std::result::Result<T, BlockingTaskError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let owner = self
            .owner
            .upgrade()
            .ok_or(BlockingTaskError::StaleGeneration)?;
        let receiver =
            owner.register_blocking(&self.download_id, &self.generation, operation, function)?;
        receiver
            .await
            .map_err(|_| BlockingTaskError::ResultChannelClosed)?
            .map_err(BlockingTaskError::Join)
    }

    pub(super) async fn run_fallible_blocking_named<T, E, F>(
        &self,
        operation: &'static str,
        function: F,
    ) -> std::result::Result<std::result::Result<T, E>, BlockingTaskError>
    where
        T: Send + 'static,
        E: Send + 'static,
        F: FnOnce() -> std::result::Result<T, E> + Send + 'static,
    {
        let owner = self
            .owner
            .upgrade()
            .ok_or(BlockingTaskError::StaleGeneration)?;
        let receiver = owner.register_fallible_blocking(
            &self.download_id,
            &self.generation,
            operation,
            function,
        )?;
        receiver
            .await
            .map_err(|_| BlockingTaskError::ResultChannelClosed)?
            .map_err(BlockingTaskError::Join)
    }

    pub(super) async fn drain_blocking(&self) -> std::result::Result<usize, BlockingTaskError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(BlockingTaskError::StaleGeneration)?;
        owner
            .drain_blocking_generation(&self.download_id, &self.generation)
            .await
    }

    #[cfg(test)]
    pub(super) fn should_fail_blocking_operation(&self, operation: &'static str) -> bool {
        self.owner.upgrade().is_some_and(|owner| {
            owner
                .blocking_failure_observer
                .lock()
                .expect("HF blocking-failure observer lock poisoned")
                .as_ref()
                .is_some_and(|observer| observer(operation))
        })
    }

    #[cfg(test)]
    pub(super) fn observe_projection(&self, projection: &'static str) {
        if let Some(owner) = self.owner.upgrade() {
            let observer = owner
                .projection_observer
                .lock()
                .expect("HF projection observer lock poisoned")
                .clone();
            if let Some(observer) = observer {
                observer(projection);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn observe_cancellation_check(&self) {
        if let Some(owner) = self.owner.upgrade() {
            let observer = owner
                .cancellation_check_observer
                .lock()
                .expect("HF cancellation-check observer lock poisoned")
                .clone();
            if let Some(observer) = observer {
                observer();
            }
        }
    }

    #[cfg(test)]
    pub(super) fn observe_worker_projection(&self, projection: &'static str) {
        if let Some(owner) = self.owner.upgrade() {
            let observer = owner
                .worker_projection_observer
                .lock()
                .expect("HF worker-projection observer lock poisoned")
                .clone();
            if let Some(observer) = observer {
                observer(projection);
            }
        }
    }
}

impl InstalledTask {
    pub(super) fn generation(&self) -> &TaskGeneration {
        &self.generation
    }

    pub(super) fn start(self) {
        let _ = self
            .owner
            .start_generation(&self.download_id, &self.generation);
    }
}

impl Drop for InstalledTask {
    fn drop(&mut self) {
        let _ = self.start_state.compare_exchange(
            TaskStartState::Gated as u8,
            TaskStartState::Abandoned as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

impl Drop for PreparedTask {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = self.start_state.compare_exchange(
            TaskStartState::Gated as u8,
            TaskStartState::Abandoned as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

impl InstalledProjection {
    pub(super) fn start(self) -> ProjectionTicket {
        let Self { task, ticket } = self;
        task.start();
        ticket
    }
}

impl ProjectionTicket {
    pub(super) async fn wait(&self) -> ProjectionOutcome {
        self.cell.wait().await
    }

    #[cfg(test)]
    pub(super) fn failure_projected_for_test(&self) -> bool {
        self.cell.failure_projected()
    }

    #[cfg(test)]
    pub(super) fn settled_for_test(&self) -> bool {
        self.cell.is_settled()
    }
}

async fn observe_entry(
    mut entry: TaskEntry,
    role: TaskRole,
    outer_finished_before_replacement: bool,
) -> TaskObservation {
    let generation = entry.generation.clone();
    let terminal = match entry.outer.await {
        Ok(()) => TaskTerminal::Completed,
        Err(error) if error.is_cancelled() => TaskTerminal::Cancelled,
        Err(_) => TaskTerminal::Panicked,
    };
    let projection_failed = entry.projection.as_ref().is_some_and(|cell| cell.failed())
        || entry
            .superseded_projection
            .as_ref()
            .is_some_and(|cell| cell.failed());
    let nested_failures = entry.nested_failures_archived
        + observe_nested(entry.nested.drain(..).collect()).await
        + usize::from(projection_failed);
    TaskObservation {
        generation,
        role,
        terminal,
        nested_failures,
        outer_finished_before_replacement,
    }
}

async fn observe_nested(nested: Vec<NestedTask>) -> usize {
    let mut failures = 0;
    for nested in nested {
        let join_failed = nested.handle.await.is_err();
        if join_failed || nested.completion.failed.load(Ordering::Acquire) {
            failures += 1;
        }
    }
    failures
}

async fn wait_for_nested(completions: &[Arc<NestedCompletion>]) -> usize {
    for completion in completions {
        loop {
            let notified = completion.notify.notified();
            if completion.finished.load(Ordering::Acquire) {
                break;
            }
            notified.await;
        }
    }
    completions
        .iter()
        .filter(|completion| completion.failed.load(Ordering::Acquire))
        .count()
}

#[cfg(test)]
mod tests {
    fn queue_destination() -> (tempfile::TempDir, DestinationIdentity) {
        let root = tempfile::TempDir::new().unwrap();
        let authority =
            crate::model_library::download_recovery::DownloadDestinationRoot::open(root.path())
                .unwrap();
        let destination = authority.resolve(Path::new("model")).unwrap().identity();
        (root, destination)
    }

    #[test]
    fn released_destination_claim_cannot_be_resurrected_from_stale_inventory() {
        let owner = super::DestinationExecutionOwner::new();
        let (_root, path) = queue_destination();
        let first = super::TaskGeneration::new();
        let second = super::TaskGeneration::new();
        assert!(owner.reserve(
            path.clone(),
            "first".into(),
            super::DestinationDomain::Ambient,
            first.clone()
        ));
        assert!(owner.release(&path, "first", super::DestinationDomain::Ambient, &first));
        assert!(!owner.reserve_dormant(
            path.clone(),
            "first".into(),
            super::DestinationDomain::Ambient
        ));
        assert!(!owner.reserve(
            path.clone(),
            "first".into(),
            super::DestinationDomain::Ambient,
            second
        ));
        assert_eq!(owner.claim_count(&path), 0);
    }
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Duration;

    fn start_cancel(transition: CancelTransition) -> bool {
        match transition {
            CancelTransition::Started(finalizer) | CancelTransition::Existing(finalizer) => {
                finalizer.start();
            }
            CancelTransition::AlreadyRunning => {}
            CancelTransition::Relocating(_) => return false,
        }
        true
    }

    async fn acknowledge_failed_projection_through_cancel(
        owner: &Arc<DownloadTaskOwner>,
        download_id: &str,
        ticket: &ProjectionTicket,
    ) {
        assert_eq!(
            owner.settle_projection(ticket),
            ProjectionSettlement::FailureUnprojected
        );
        assert!(owner.contains(download_id));
        let (acknowledged_sender, acknowledged) = oneshot::channel();
        let transition = owner.begin_cancel(download_id, move |context, predecessor| async move {
            let CancelPredecessor::Observed(observation) = predecessor else {
                panic!("failed projector must remain predecessor custody");
            };
            assert!(observation.nested_failures > 0);
            assert!(context.complete_transferred_projection(true));
            let _ = acknowledged_sender.send(());
        });
        let finalizer = match transition {
            CancelTransition::Started(finalizer) => finalizer,
            _ => panic!("failed projector must be replaced by one finalizer"),
        };
        finalizer.start();
        assert_eq!(
            owner.settle_projection(ticket),
            ProjectionSettlement::StaleGeneration
        );
        tokio::time::timeout(Duration::from_secs(1), acknowledged)
            .await
            .expect("finalizer must acknowledge transferred failure")
            .unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot(download_id)
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("acknowledging finalizer must reach terminal Join state");
        assert!(ticket.failure_projected_for_test());
        assert!(ticket.settled_for_test());
        assert!(owner.observe_finished(download_id).await.is_some());
        assert!(!owner.contains(download_id));
        assert_eq!(
            owner.settle_projection(ticket),
            ProjectionSettlement::AlreadySettled
        );
    }

    #[tokio::test]
    async fn prepared_task_runs_only_after_owned_install_and_start() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let ran = Arc::new(AtomicBool::new(false));
        let ran_in_task = ran.clone();
        let prepared = owner.prepare(
            "download".to_string(),
            TaskRole::Worker,
            move |_| async move {
                ran_in_task.store(true, Ordering::SeqCst);
            },
        );

        tokio::task::yield_now().await;
        assert!(!ran.load(Ordering::SeqCst));

        let installed = owner.install_gated(prepared).unwrap();
        let generation = installed.generation().clone();
        assert!(owner.snapshot("download").is_some_and(|snapshot| {
            owner
                .generation_for_test("download")
                .is_some_and(|current| current.matches(&generation))
                && snapshot.role == TaskRole::Worker
                && !snapshot.finished
        }));
        assert!(!ran.load(Ordering::SeqCst));

        installed.start();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !ran.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("installed task should start");
    }

    #[tokio::test]
    async fn dropping_unstarted_install_generation_matches_cleanup() {
        for role in [TaskRole::Worker, TaskRole::RecoveryTransition] {
            let owner = Arc::new(DownloadTaskOwner::new());
            let ran = Arc::new(AtomicBool::new(false));
            let ran_in_task = ran.clone();
            let prepared = owner.prepare("download".to_string(), role, move |_| async move {
                ran_in_task.store(true, Ordering::SeqCst);
            });
            let installed = owner.install_gated(prepared).unwrap();
            assert!(owner.contains("download"));
            drop(installed);
            assert!(owner.contains("download"));
            assert!(!ran.load(Ordering::SeqCst));

            owner.rescue_abandoned();
            assert!(!owner.contains("download"));
            tokio::time::timeout(Duration::from_secs(1), async {
                while owner.outstanding_retired_for_test() != 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("abandoned installed wrapper must be observed to terminal");
            assert_eq!(owner.retired_observations_for_test(), 1);
            assert!(!ran.load(Ordering::SeqCst));
        }
    }

    #[tokio::test]
    async fn abandoned_projection_is_rescued_without_signalling_from_drop() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let ran = Arc::new(AtomicBool::new(false));
        let ran_in_projection = ran.clone();
        let prepared = owner.prepare_projection(
            "download".to_string(),
            move |_, predecessor| {
                let ran = ran_in_projection.clone();
                async move {
                    assert!(predecessor.is_none());
                    ran.store(true, Ordering::SeqCst);
                    ProjectionOutcome::Committed
                }
            },
            |_| async { ProjectionOutcome::Failed },
        );
        let InstalledProjection { task, ticket } =
            owner.install_projection_gated(prepared).unwrap();

        // Token destruction is CAS-only. It cannot release the gate while an
        // outer state guard may still be unwinding.
        drop(task);
        tokio::task::yield_now().await;
        assert!(!ran.load(Ordering::SeqCst));
        assert!(owner.contains("download"));

        owner.rescue_abandoned();
        assert_eq!(ticket.wait().await, ProjectionOutcome::Committed);
        assert!(ran.load(Ordering::SeqCst));
        while owner.settle_projection(&ticket) == ProjectionSettlement::Pending {
            tokio::task::yield_now().await;
        }
        assert_eq!(
            owner.settle_projection(&ticket),
            ProjectionSettlement::AlreadySettled
        );
    }

    #[tokio::test]
    async fn abandoned_prepared_collision_is_inert_until_explicit_rescue() {
        for rejected_role in [TaskRole::Worker, TaskRole::RecoveryTransition] {
            let owner = Arc::new(DownloadTaskOwner::new());
            let first = owner.prepare("download".to_string(), TaskRole::Worker, |_| async {
                std::future::pending::<()>().await;
            });
            owner.install_gated(first).unwrap().start();

            let ran = Arc::new(AtomicBool::new(false));
            let ran_in_task = ran.clone();
            let second =
                owner.prepare("download".to_string(), rejected_role, move |_| async move {
                    ran_in_task.store(true, Ordering::SeqCst);
                });
            let rejected = owner.install_gated(second).unwrap_err();
            drop(rejected);
            tokio::task::yield_now().await;
            assert!(!ran.load(Ordering::SeqCst));
            assert_eq!(owner.prepared_count_for_test(), 1);

            owner.rescue_abandoned();
            assert_eq!(owner.prepared_count_for_test(), 0);
            tokio::time::timeout(Duration::from_secs(1), async {
                while owner.outstanding_retired_for_test() != 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("abandoned prepared wrapper must be observed to terminal");
            assert_eq!(owner.retired_observations_for_test(), 1);
            assert!(!ran.load(Ordering::SeqCst));
        }
    }

    #[tokio::test]
    async fn projection_settlement_distinguishes_duplicate_stale_and_missing() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let prepared = owner.prepare_projection(
            "projection".to_string(),
            |_, _| async { ProjectionOutcome::Committed },
            |_| async { ProjectionOutcome::Failed },
        );
        let ticket = owner.install_projection_gated(prepared).unwrap().start();
        assert_eq!(
            owner.settle_projection(&ticket),
            ProjectionSettlement::Pending
        );
        assert_eq!(ticket.wait().await, ProjectionOutcome::Committed);
        while owner.settle_projection(&ticket) == ProjectionSettlement::Pending {
            tokio::task::yield_now().await;
        }
        assert_eq!(
            owner.settle_projection(&ticket),
            ProjectionSettlement::AlreadySettled
        );

        let missing_cell = Arc::new(ProjectionCell::new(true));
        missing_cell.settle(ProjectionOutcome::Committed);
        let missing = ProjectionTicket {
            download_id: "missing".to_string(),
            generation: TaskGeneration::new(),
            cell: missing_cell,
        };
        assert_eq!(
            owner.settle_projection(&missing),
            ProjectionSettlement::Missing
        );

        let stale_cell = Arc::new(ProjectionCell::new(true));
        stale_cell.settle(ProjectionOutcome::Committed);
        let stale = ProjectionTicket {
            download_id: "successor".to_string(),
            generation: TaskGeneration::new(),
            cell: stale_cell,
        };
        let successor = owner.prepare("successor".to_string(), TaskRole::Worker, |_| async {
            std::future::pending::<()>().await;
        });
        owner.install_gated(successor).unwrap().start();
        assert_eq!(
            owner.settle_projection(&stale),
            ProjectionSettlement::StaleGeneration
        );
        assert!(owner.contains("successor"));
    }

    #[tokio::test]
    async fn projection_catches_call_and_poll_panics_for_both_constructors() {
        let owner = Arc::new(DownloadTaskOwner::new());

        let call = owner.prepare_projection(
            "ownerless-call".to_string(),
            |_, _| {
                panic!("call-time projection panic");
                #[allow(unreachable_code)]
                std::future::ready(ProjectionOutcome::Committed)
            },
            |_| async { ProjectionOutcome::Failed },
        );
        let call_ticket = owner.install_projection_gated(call).unwrap().start();
        assert_eq!(call_ticket.wait().await, ProjectionOutcome::Panicked);

        let poll = owner.prepare_projection(
            "ownerless-poll".to_string(),
            |_, _| async {
                panic!("poll-time projection panic");
            },
            |_| async { ProjectionOutcome::Failed },
        );
        let poll_ticket = owner.install_projection_gated(poll).unwrap().start();
        assert_eq!(poll_ticket.wait().await, ProjectionOutcome::Panicked);

        for (id, call_time) in [("finished-call", true), ("finished-poll", false)] {
            let predecessor = owner.prepare(id.to_string(), TaskRole::Worker, |_| async {});
            owner.install_gated(predecessor).unwrap().start();
            tokio::time::timeout(Duration::from_secs(1), async {
                while !owner.snapshot(id).is_some_and(|task| task.finished) {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            let transition = if call_time {
                owner.begin_finished_projection(
                    id,
                    false,
                    |_, _| {
                        panic!("call-time finished projection panic");
                        #[allow(unreachable_code)]
                        std::future::ready(ProjectionOutcome::Committed)
                    },
                    |_| async { ProjectionOutcome::Failed },
                )
            } else {
                owner.begin_finished_projection(
                    id,
                    false,
                    |_, _| async {
                        panic!("poll-time finished projection panic");
                    },
                    |_| async { ProjectionOutcome::Failed },
                )
            };
            let ProjectionTransition::Started(projection) = transition else {
                panic!("finished predecessor should install a projector");
            };
            let ticket = projection.start();
            assert_eq!(ticket.wait().await, ProjectionOutcome::Panicked);
        }
    }

    #[tokio::test]
    async fn fallback_panics_remain_owned_until_cancel_acknowledges_failure() {
        let owner = Arc::new(DownloadTaskOwner::new());

        let ownerless = owner.prepare_projection(
            "ownerless-double-panic".to_string(),
            |_, _| {
                panic!("call-time primary projection panic");
                #[allow(unreachable_code)]
                std::future::ready(ProjectionOutcome::Committed)
            },
            |_| {
                panic!("call-time fallback projection panic");
                #[allow(unreachable_code)]
                std::future::ready(ProjectionOutcome::Failed)
            },
        );
        let InstalledProjection { task, ticket } =
            owner.install_projection_gated(ownerless).unwrap();
        let abandoned_waiter_ticket = ticket.clone();
        let abandoned_waiter = tokio::spawn(async move {
            let _ = abandoned_waiter_ticket.wait().await;
        });
        abandoned_waiter.abort();
        let _ = abandoned_waiter.await;
        task.start();
        assert_eq!(ticket.wait().await, ProjectionOutcome::Panicked);
        acknowledge_failed_projection_through_cancel(&owner, "ownerless-double-panic", &ticket)
            .await;

        let predecessor = owner.prepare(
            "finished-double-panic".to_string(),
            TaskRole::Worker,
            |_| async {},
        );
        owner.install_gated(predecessor).unwrap().start();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot("finished-double-panic")
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let transition = owner.begin_finished_projection(
            "finished-double-panic",
            false,
            |_, _| async {
                panic!("poll-time primary projection panic");
            },
            |_| async {
                panic!("poll-time fallback projection panic");
            },
        );
        let ProjectionTransition::Started(projection) = transition else {
            panic!("finished predecessor should install a projector");
        };
        let ticket = projection.start();
        assert_eq!(ticket.wait().await, ProjectionOutcome::Panicked);
        acknowledge_failed_projection_through_cancel(&owner, "finished-double-panic", &ticket)
            .await;
    }

    #[tokio::test]
    async fn superseded_failed_projection_is_unacked_until_finalizer_projection() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let (fallback_reached_sender, fallback_reached) = oneshot::channel();
        let (_fallback_release_sender, fallback_release) = oneshot::channel::<()>();
        let prepared = owner.prepare_projection(
            "transferred-failure".to_string(),
            |_, _| async {
                panic!("primary projection panic");
            },
            move |_| async move {
                let _ = fallback_reached_sender.send(());
                let _ = fallback_release.await;
                ProjectionOutcome::RolledBack
            },
        );
        let ticket = owner.install_projection_gated(prepared).unwrap().start();
        tokio::time::timeout(Duration::from_secs(1), fallback_reached)
            .await
            .expect("primary panic must enter fallback")
            .unwrap();
        assert!(!ticket.failure_projected_for_test());

        let (finalizer_reached_sender, finalizer_reached) = oneshot::channel();
        let (allow_projection_sender, allow_projection) = oneshot::channel();
        let transition = owner.begin_cancel(
            "transferred-failure",
            move |context, predecessor| async move {
                let CancelPredecessor::Observed(observation) = predecessor else {
                    panic!("superseded projector must remain predecessor custody");
                };
                assert!(observation.nested_failures > 0);
                let _ = finalizer_reached_sender.send(());
                let _ = allow_projection.await;
                assert!(context.complete_transferred_projection(true));
            },
        );
        let CancelTransition::Started(finalizer) = transition else {
            panic!("projection must be replaced by one finalizer");
        };
        finalizer.start();
        assert_eq!(ticket.wait().await, ProjectionOutcome::Superseded);
        tokio::time::timeout(Duration::from_secs(1), finalizer_reached)
            .await
            .expect("finalizer must observe the failed projector")
            .unwrap();
        assert!(!ticket.failure_projected_for_test());
        assert!(!ticket.settled_for_test());
        allow_projection_sender.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !ticket.failure_projected_for_test() || !ticket.settled_for_test() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("finalizer must acknowledge only after its terminal projection");
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot("transferred-failure")
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(owner
            .observe_finished("transferred-failure")
            .await
            .is_some());
        assert!(!owner.contains("transferred-failure"));
    }

    #[tokio::test]
    async fn finished_and_panicked_tasks_are_observed_once() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let prepared = owner.prepare("panic".to_string(), TaskRole::Worker, |_| async {
            panic!("sentinel panic");
        });
        owner.install_gated(prepared).unwrap().start();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot("panic")
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let observation = owner.observe_finished("panic").await.unwrap();
        assert_eq!(observation.role, TaskRole::Worker);
        assert_eq!(observation.terminal, TaskTerminal::Panicked);
        assert_eq!(observation.nested_failures, 0);
        assert!(owner.observe_finished("panic").await.is_none());
    }

    #[tokio::test]
    async fn cancel_finalizer_drains_registered_blocking_work_before_terminal_callback() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let (blocking_started_sender, blocking_started) = oneshot::channel();
        let (release_sender, release) = std::sync::mpsc::channel();
        let prepared = owner.prepare(
            "download".to_string(),
            TaskRole::Worker,
            move |context| async move {
                let _ = context
                    .run_blocking(move || {
                        let _ = blocking_started_sender.send(());
                        let _ = release.recv();
                    })
                    .await;
            },
        );
        owner.install_gated(prepared).unwrap().start();
        blocking_started.await.unwrap();

        let finalized = Arc::new(AtomicBool::new(false));
        let finalized_in_task = finalized.clone();
        assert!(start_cancel(owner.begin_cancel(
            "download",
            move |_context, predecessor| async move {
                let CancelPredecessor::Observed(observation) = predecessor else {
                    panic!("installed worker must be observed");
                };
                assert_eq!(observation.terminal, TaskTerminal::Cancelled);
                finalized_in_task.store(true, Ordering::SeqCst);
            },
        )));
        tokio::task::yield_now().await;
        assert!(!finalized.load(Ordering::SeqCst));

        release_sender.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !finalized.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("finalizer should wait for nested blocking work");
    }

    #[tokio::test]
    async fn stale_generation_cannot_observe_or_remove_cancel_successor() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let prepared = owner.prepare("download".to_string(), TaskRole::Worker, |_| async {
            std::future::pending::<()>().await;
        });
        let installed = owner.install_gated(prepared).unwrap();
        let stale_generation = installed.generation().clone();
        installed.start();
        let (finish_sender, finish_receiver) = oneshot::channel();
        let transition = owner.begin_cancel("download", move |_context, _| async move {
            let _ = finish_receiver.await;
        });
        let CancelTransition::Started(finalizer) = transition else {
            panic!("worker should transition to a finalizer");
        };
        finalizer.start();
        let successor = owner.generation_for_test("download").unwrap();

        assert!(owner
            .observe_finished_generation("download", &stale_generation)
            .await
            .is_none());
        assert!(owner.snapshot("download").is_some_and(|snapshot| {
            owner
                .generation_for_test("download")
                .is_some_and(|current| current.matches(&successor))
                && snapshot.role == TaskRole::CancelFinalizer
        }));
        finish_sender.send(()).unwrap();
    }

    #[tokio::test]
    async fn repeated_cancel_keeps_one_finalizer_owner() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let prepared = owner.prepare("download".to_string(), TaskRole::Worker, |_| async {
            std::future::pending::<()>().await;
        });
        owner.install_gated(prepared).unwrap().start();
        let count = Arc::new(AtomicUsize::new(0));
        let count_in_finalizer = count.clone();
        let (finish_sender, finish_receiver) = oneshot::channel();
        let first = owner.begin_cancel("download", move |_context, _| async move {
            count_in_finalizer.fetch_add(1, Ordering::SeqCst);
            let _ = finish_receiver.await;
        });
        let CancelTransition::Started(finalizer) = first else {
            panic!("first cancellation should install a finalizer");
        };
        finalizer.start();
        let first_generation = owner.generation_for_test("download").unwrap();
        let second = owner.begin_cancel("download", |_, _| async {});
        let CancelTransition::AlreadyRunning = second else {
            panic!("repeat cancellation must not replace the finalizer");
        };
        let second_generation = owner.generation_for_test("download").unwrap();
        assert!(first_generation.matches(&second_generation));
        finish_sender.send(()).unwrap();
        tokio::task::yield_now().await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn blocking_panic_is_retained_when_outer_receiver_is_cancelled() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let (blocking_started_sender, blocking_started) = oneshot::channel();
        let (release_sender, release) = std::sync::mpsc::channel();
        let prepared = owner.prepare(
            "download".to_string(),
            TaskRole::Worker,
            move |context| async move {
                let _ = context
                    .run_blocking(move || {
                        let _ = blocking_started_sender.send(());
                        let _ = release.recv();
                        panic!("nested sentinel panic");
                    })
                    .await;
            },
        );
        owner.install_gated(prepared).unwrap().start();
        blocking_started.await.unwrap();

        let (observed_sender, observed) = oneshot::channel();
        assert!(start_cancel(owner.begin_cancel(
            "download",
            move |_context, predecessor| async move {
                let _ = observed_sender.send(predecessor);
            },
        )));
        release_sender.send(()).unwrap();
        let CancelPredecessor::Observed(observation) = observed.await.unwrap() else {
            panic!("installed worker must be observed");
        };
        assert_eq!(observation.terminal, TaskTerminal::Cancelled);
        assert_eq!(observation.nested_failures, 1);
    }

    #[tokio::test]
    async fn finished_unobserved_finalizer_is_replaced_and_observed_once() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let prepared = owner.prepare("download".to_string(), TaskRole::Worker, |_| async {});
        owner.install_gated(prepared).unwrap().start();
        while !owner
            .snapshot("download")
            .is_some_and(|snapshot| snapshot.finished)
        {
            tokio::task::yield_now().await;
        }
        let count = Arc::new(AtomicUsize::new(0));
        let count_in_finalizer = count.clone();
        assert!(start_cancel(owner.begin_cancel(
            "download",
            move |_, _| async move {
                count_in_finalizer.fetch_add(1, Ordering::SeqCst);
            },
        )));
        while !owner
            .snapshot("download")
            .is_some_and(|snapshot| snapshot.finished)
        {
            tokio::task::yield_now().await;
        }

        let (predecessor_sender, predecessor) = oneshot::channel();
        let replacement = owner.begin_cancel("download", move |_, predecessor| async move {
            let _ = predecessor_sender.send(predecessor);
        });
        let CancelTransition::Started(replacement) = replacement else {
            panic!("finished finalizer must be replaced by an observing finalizer");
        };
        replacement.start();
        let CancelPredecessor::Observed(observation) = predecessor.await.unwrap() else {
            panic!("replacement must observe the finished finalizer");
        };
        assert_eq!(observation.role, TaskRole::CancelFinalizer);
        assert_eq!(observation.terminal, TaskTerminal::Completed);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot("download")
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            owner.observe_finished("download").await.unwrap().role,
            TaskRole::CancelFinalizer
        );
        assert!(owner.observe_finished("download").await.is_none());
    }

    #[tokio::test]
    async fn cancelling_an_outer_drain_keeps_nested_custody_with_the_finalizer() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let (drain_sender, drain_receiver) = oneshot::channel();
        let prepared = owner.prepare(
            "download".to_string(),
            TaskRole::Worker,
            move |context| async move {
                let _ = drain_receiver.await;
                let _ = context.drain_blocking().await;
            },
        );
        let installed = owner.install_gated(prepared).unwrap();
        let generation = installed.generation().clone();
        installed.start();

        let (blocking_started_sender, blocking_started) = oneshot::channel();
        let (release_sender, release) = std::sync::mpsc::channel();
        let result = owner
            .register_blocking(
                "download",
                &generation,
                "drain cancellation sentinel",
                move || {
                    let _ = blocking_started_sender.send(());
                    let _ = release.recv();
                },
            )
            .unwrap();
        drop(result);
        blocking_started.await.unwrap();
        let (drain_started_sender, drain_started) = oneshot::channel();
        let drain_started_sender = Arc::new(Mutex::new(Some(drain_started_sender)));
        owner.set_drain_observer(Some(Arc::new(move || {
            if let Some(sender) = drain_started_sender.lock().unwrap().take() {
                let _ = sender.send(());
            }
        })));
        drain_sender.send(()).unwrap();
        drain_started.await.unwrap();
        assert_eq!(owner.nested_count_for_test("download"), Some(1));

        let terminal = Arc::new(AtomicBool::new(false));
        let terminal_in_finalizer = terminal.clone();
        assert!(start_cancel(owner.begin_cancel(
            "download",
            move |_, _| async move {
                terminal_in_finalizer.store(true, Ordering::SeqCst);
            },
        )));
        tokio::task::yield_now().await;
        let terminal_before_release = terminal.load(Ordering::SeqCst);
        release_sender.send(()).unwrap();
        owner.set_drain_observer(None);

        assert!(
            !terminal_before_release,
            "cancelling a drain must not detach its registered blocking owner"
        );
        tokio::time::timeout(Duration::from_secs(1), async {
            while !terminal.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the same runtime must drive the finalizer after nested release");
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot("download")
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("finalizer should reach an observable terminal outcome");
        let observation = owner.observe_finished("download").await.unwrap();
        assert_eq!(observation.role, TaskRole::CancelFinalizer);
        assert_eq!(observation.terminal, TaskTerminal::Completed);
        assert_eq!(observation.nested_failures, 0);
    }

    #[tokio::test]
    async fn finished_outer_is_not_observable_until_registered_blocking_work_finishes() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let (outer_release_sender, outer_release) = oneshot::channel();
        let prepared = owner.prepare(
            "download".to_string(),
            TaskRole::Worker,
            move |_| async move {
                let _ = outer_release.await;
            },
        );
        let installed = owner.install_gated(prepared).unwrap();
        let generation = installed.generation().clone();
        let (blocking_started_sender, blocking_started) = oneshot::channel();
        let (release_sender, release) = std::sync::mpsc::channel();
        // Register the real held blocking work synchronously through the
        // owner/generation before allowing the outer future to complete.
        let nested_result = owner
            .register_blocking(
                "download",
                &generation,
                "finished outer held operation",
                move || {
                    let _ = blocking_started_sender.send(());
                    let _ = release.recv();
                },
            )
            .unwrap();
        installed.start();
        tokio::time::timeout(Duration::from_secs(1), blocking_started)
            .await
            .expect("registered blocking work should start")
            .unwrap();
        outer_release_sender.send(()).unwrap();
        tokio::task::yield_now().await;

        assert!(owner
            .snapshot("download")
            .is_some_and(|snapshot| { !snapshot.finished && snapshot.role == TaskRole::Worker }));
        assert!(owner.observe_finished("download").await.is_none());
        assert!(owner.contains("download"));

        release_sender.send(()).unwrap();
        nested_result.await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !owner
                .snapshot("download")
                .is_some_and(|snapshot| snapshot.finished)
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("nested completion should make the owner observable");
        let observation = owner
            .observe_finished("download")
            .await
            .expect("finished nested work must be observable");
        assert_eq!(observation.role, TaskRole::Worker);
        assert_eq!(observation.terminal, TaskTerminal::Completed);
        assert_eq!(observation.nested_failures, 0);
        assert!(!owner.contains("download"));
    }

    #[tokio::test]
    async fn completed_nested_work_is_reaped_without_losing_failure_evidence() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let owner_in_task = owner.clone();
        let (metrics_sender, metrics) = oneshot::channel();
        let prepared = owner.prepare(
            "download".to_string(),
            TaskRole::Worker,
            move |context| async move {
                let mut retained_max = 0;
                for index in 0..512 {
                    let result = context
                        .run_blocking(move || {
                            if index == 0 {
                                panic!("archived nested failure sentinel");
                            }
                        })
                        .await;
                    if index == 0 {
                        assert!(matches!(result, Err(BlockingTaskError::Join(_))));
                    } else {
                        result.unwrap();
                    }
                    retained_max = retained_max.max(
                        owner_in_task
                            .nested_count_for_test("download")
                            .unwrap_or_default(),
                    );
                }
                let drained_failures = context.drain_blocking().await.unwrap();
                let _ = metrics_sender.send((retained_max, drained_failures));
                std::future::pending::<()>().await;
            },
        );
        owner.install_gated(prepared).unwrap().start();

        let (retained_max, drained_failures) = metrics.await.unwrap();
        assert!(
            retained_max <= 2,
            "sequential blocking operations must retain only the current and terminalizing observer"
        );
        assert_eq!(drained_failures, 1);

        let (predecessor_sender, predecessor) = oneshot::channel();
        assert!(start_cancel(owner.begin_cancel(
            "download",
            move |_, predecessor| async move {
                let _ = predecessor_sender.send(predecessor);
            },
        )));
        let CancelPredecessor::Observed(observation) = predecessor.await.unwrap() else {
            panic!("the worker remains owned until cancellation");
        };
        assert_eq!(observation.nested_failures, 1);
    }

    #[tokio::test]
    async fn state_only_cancellation_does_not_fabricate_a_worker_observation() {
        let owner = Arc::new(DownloadTaskOwner::new());
        let owner_in_finalizer = owner.clone();
        let (observation_sender, observation) = oneshot::channel();
        assert!(start_cancel(owner.begin_cancel(
            "state-only",
            move |_, observation| async move {
                assert!(owner_in_finalizer.contains("state-only"));
                let _ = observation_sender.send(observation);
            },
        )));

        assert_eq!(observation.await.unwrap(), CancelPredecessor::Absent);
    }

    #[tokio::test]
    async fn destination_reservations_follow_admission_order_not_poll_order() {
        let owner = Arc::new(DestinationExecutionOwner::new());
        let (_root, path) = queue_destination();
        let first = TaskGeneration::new();
        let second = TaskGeneration::new();
        assert!(owner.reserve(
            path.clone(),
            "first".to_string(),
            DestinationDomain::Ambient,
            first.clone(),
        ));
        assert!(owner.reserve(
            path.clone(),
            "second".to_string(),
            DestinationDomain::Ambient,
            second.clone(),
        ));

        let second_owner = owner.clone();
        let second_path = path.clone();
        let second_generation = second.clone();
        let mut second_waiter = tokio::spawn(async move {
            second_owner
                .wait_for_turn(
                    &second_path,
                    "second",
                    DestinationDomain::Ambient,
                    &second_generation,
                )
                .await
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(25), &mut second_waiter)
                .await
                .is_err(),
            "the later reservation must not win by polling first"
        );
        assert!(
            owner
                .wait_for_turn(&path, "first", DestinationDomain::Ambient, &first,)
                .await
        );
        assert!(owner.release(&path, "first", DestinationDomain::Ambient, &first,));
        assert!(tokio::time::timeout(Duration::from_secs(1), second_waiter)
            .await
            .expect("the next admitted reservation must be woken")
            .expect("destination waiter must join"));
    }

    #[tokio::test]
    async fn dormant_destination_claim_blocks_then_promotes_in_place() {
        let owner = Arc::new(DestinationExecutionOwner::new());
        let (_root, path) = queue_destination();
        let resumed = TaskGeneration::new();
        let follower = TaskGeneration::new();
        assert!(owner.reserve_dormant(
            path.clone(),
            "paused".to_string(),
            DestinationDomain::Ambient,
        ));
        assert!(owner.reserve(
            path.clone(),
            "follower".to_string(),
            DestinationDomain::Ambient,
            follower.clone(),
        ));
        assert!(owner.reserve(
            path.clone(),
            "paused".to_string(),
            DestinationDomain::Ambient,
            resumed.clone(),
        ));

        assert!(
            owner
                .wait_for_turn(&path, "paused", DestinationDomain::Ambient, &resumed,)
                .await
        );
        assert_eq!(owner.claim_count(&path), 2);
        assert!(owner.contains(&path, "paused", DestinationDomain::Ambient, &resumed,));
        assert!(owner.release(&path, "paused", DestinationDomain::Ambient, &resumed,));
        assert!(
            owner
                .wait_for_turn(&path, "follower", DestinationDomain::Ambient, &follower,)
                .await
        );
    }

    #[test]
    fn destination_domain_changes_only_through_explicit_promotion() {
        let owner = DestinationExecutionOwner::new();
        let (_root, path) = queue_destination();
        let transition = TaskGeneration::new();
        assert!(owner.reserve_dormant(
            path.clone(),
            "download".to_string(),
            DestinationDomain::Ambient,
        ));
        assert!(!owner.reserve(
            path.clone(),
            "download".to_string(),
            DestinationDomain::Recovery,
            transition.clone(),
        ));
        assert!(owner.reserve(
            path.clone(),
            "download".to_string(),
            DestinationDomain::Ambient,
            transition.clone(),
        ));
        assert!(owner.promote_domain(
            &path,
            "download",
            DestinationDomain::Ambient,
            DestinationDomain::Recovery,
            &transition,
        ));
        assert!(owner.contains(&path, "download", DestinationDomain::Recovery, &transition,));
    }
}
