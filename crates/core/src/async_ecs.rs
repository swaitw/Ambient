use ambient_ecs::{components, Entity, FnSystem, Resource, SystemGroup, World};
use ambient_sys::time::Instant;

components!("ecs", {
    @[Resource]
    async_run: AsyncRun,
});

pub type AsyncEcsAction = Box<dyn FnOnce(&mut World) + Sync + Send>;

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct AsyncRun(
    flume::Sender<AsyncEcsAction>,
    Option<flume::Receiver<AsyncEcsAction>>,
);
impl AsyncRun {
    fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        Self(tx, Some(rx))
    }
    pub fn run<F: FnOnce(&mut World) + Sync + Send + 'static>(&self, action: F) {
        self.0.send(Box::new(action)).ok();
    }
}
impl std::fmt::Debug for AsyncRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AsyncRun").finish()
    }
}

pub fn async_ecs_resources() -> Entity {
    Entity::new().with(async_run(), AsyncRun::new())
}

pub fn async_ecs_systems() -> SystemGroup {
    SystemGroup::new(
        "async_ecs_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let start = Instant::now();
            let rx = world.resource_mut(async_run()).1.take().unwrap();
            for action in rx.try_iter() {
                action(world);
                if Instant::now().duration_since(start).as_millis() > 50 {
                    let remaining = rx.len();
                    if remaining > 0 {
                        tracing::warn!("async ecs timeout. Remaining actions: {remaining}");
                    }
                    break;
                }
            }
            world.resource_mut(async_run()).1 = Some(rx);

            let dur = Instant::now().duration_since(start).as_millis();
            if dur > 100 {
                tracing::warn!("Async run ran for {dur}ms");
            }
        }))],
    )
}
