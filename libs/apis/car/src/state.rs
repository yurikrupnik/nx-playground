/// CarState trait composed from HasMongoDB and HasRedis
///
/// Any type that implements both HasMongoDB and HasRedis automatically
/// implements CarState - no need for additional impl blocks!
///
/// This demonstrates the composition pattern: APIs mix-and-match traits they need.
pub trait CarState: app::state::HasMongoDB + app::state::HasRedis {}

/// Blanket implementation: any type with HasMongoDB + HasRedis gets CarState for free
impl<T> CarState for T where T: app::state::HasMongoDB + app::state::HasRedis {}
