pub mod graphics;
pub mod ipc;
pub mod wayland;

pub use graphics::physics::PetPhysics;
pub use graphics::sprite_animator::{SpriteAnimator, SpriteSheet};
pub use ipc::{IpcClient, STATE_SOCKET_PATH, TELEMETRY_SOCKET_PATH};
pub use wayland::layer_surface::{
    LayerSurfaceConfig, WaylandEdge, WaylandLayer, WaylandLayerSurfaceManager,
};
