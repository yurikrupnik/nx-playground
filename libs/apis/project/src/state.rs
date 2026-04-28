/// ProjectState trait composed from HasDatabase and HasRedis
///
/// Any type that implements both HasDatabase and HasRedis automatically
/// implements ProjectState - no need for additional impl blocks!
pub trait ProjectState: app::state::HasDatabase + app::state::HasRedis {}

/// Blanket implementation: any type with HasDatabase + HasRedis gets ProjectState for free
impl<T> ProjectState for T where T: app::state::HasDatabase + app::state::HasRedis {}
