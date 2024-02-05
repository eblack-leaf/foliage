use bevy_ecs::prelude::{IntoSystemConfigs, ResMut, Resource, Schedule, SystemSet, World};
use bevy_ecs::schedule::{ExecutorKind, ScheduleLabel};

/// Wrapper around a bevy_ecs::World
pub type Container = World;
/// Wrapper around a bevy_ecs::Schedule
pub type Task = Schedule;

/// State of a Job
#[derive(PartialEq)]
pub enum ExecutionState {
    Active,
    Suspended,
}

/// Idle hook
#[derive(Copy, Clone, Resource)]
pub struct Idle {
    pub can_idle: bool,
}

impl Default for Idle {
    fn default() -> Self {
        Self::new()
    }
}
impl Idle {
    pub fn new() -> Self {
        Self { can_idle: false }
    }
}

/// System for attempting to idle at the beginning of each loop
pub fn attempt_to_idle(mut idle: ResMut<Idle>) {
    idle.can_idle = true;
}

/// Exit hook
#[derive(Copy, Clone, Resource)]
pub struct Exit {
    pub exit_requested: bool,
}

impl Exit {
    pub fn new() -> Self {
        Self {
            exit_requested: false,
        }
    }
    #[allow(unused)]
    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }
}
impl Default for Exit {
    fn default() -> Self {
        Self::new()
    }
}
/// Extensible container + task runner
pub struct Job {
    pub execution_state: ExecutionState,
    pub container: Container,
    pub startup: Task,
    pub main: Task,
    pub teardown: Task,
}

/// SyncPoint for Job Idle behaviour
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum JobSyncPoint {
    Idle,
}
#[derive(ScheduleLabel, Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum ScheduleLabels {
    Startup,
    Main,
    Teardown,
}
impl Job {
    pub(crate) fn new() -> Self {
        Self {
            execution_state: ExecutionState::Suspended,
            container: {
                let mut container = Container::new();
                container.insert_resource(Exit::new());
                container.insert_resource(Idle::new());
                container
            },
            startup: Task::new(ScheduleLabels::Startup),
            main: {
                let mut task = Task::new(ScheduleLabels::Main);
                task.add_systems((attempt_to_idle.in_set(JobSyncPoint::Idle),));
                task
            },
            teardown: Task::new(ScheduleLabels::Teardown),
        }
    }
    pub fn startup(&mut self) -> &mut Task {
        &mut self.startup
    }
    pub fn main(&mut self) -> &mut Task {
        &mut self.main
    }
    pub fn teardown(&mut self) -> &mut Task {
        &mut self.teardown
    }
    pub fn exec_main(&mut self) {
        tracing::trace!("elm:exec-main");
        self.main
            .set_executor_kind(ExecutorKind::MultiThreaded)
            .run(&mut self.container);
    }
    pub fn exec_startup(&mut self) {
        self.startup
            .set_executor_kind(ExecutorKind::MultiThreaded)
            .run(&mut self.container);
    }
    pub fn exec_teardown(&mut self) {
        self.teardown
            .set_executor_kind(ExecutorKind::MultiThreaded)
            .run(&mut self.container);
    }
    pub fn suspend(&mut self) {
        self.execution_state = ExecutionState::Suspended;
    }
    pub fn resume(&mut self) {
        self.execution_state = ExecutionState::Active;
    }
    pub fn suspended(&self) -> bool {
        self.execution_state == ExecutionState::Suspended
    }
    pub fn resumed(&self) -> bool {
        self.execution_state == ExecutionState::Active
    }
    pub fn should_exit(&self) -> bool {
        return self
            .container
            .get_resource::<Exit>()
            .expect("no exit found")
            .exit_requested;
    }
    pub fn can_idle(&self) -> bool {
        return self
            .container
            .get_resource::<Idle>()
            .expect("no idle found")
            .can_idle;
    }
}
