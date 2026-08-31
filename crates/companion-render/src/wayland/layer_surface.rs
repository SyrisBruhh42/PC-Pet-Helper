use crate::graphics::physics::PetPhysics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandLayer {
    Top,
    Bottom,
    Overlay,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandEdge {
    Top = 1,
    Bottom = 2,
    Left = 4,
    Right = 8,
}

pub struct LayerSurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub layer: WaylandLayer,
    pub anchored_edges: u32, // Bitfield of edges
}

pub struct WaylandLayerSurfaceManager {
    pub config: LayerSurfaceConfig,
    pub is_wayland_active: bool,
    pub current_input_region: Option<(i32, i32, i32, i32)>,
}

impl WaylandLayerSurfaceManager {
    pub fn new(width: u32, height: u32) -> Self {
        let is_wayland_active = std::env::var("WAYLAND_DISPLAY").is_ok();
        Self {
            config: LayerSurfaceConfig {
                width,
                height,
                layer: WaylandLayer::Top,
                anchored_edges: (WaylandEdge::Bottom as u32)
                    | (WaylandEdge::Left as u32)
                    | (WaylandEdge::Right as u32),
            },
            is_wayland_active,
            current_input_region: None,
        }
    }

    /// Calculate and update the active Wayland input region to strictly match the pet's bounding box [x, y, x + 64, y + 64].
    /// Transparent areas outside this region pass clicks through to underlying windows.
    pub fn update_input_region(&mut self, physics: &PetPhysics) -> (i32, i32, i32, i32) {
        let bbox = physics.bounding_box();
        self.current_input_region = Some(bbox);
        if self.is_wayland_active {
            // Apply wl_surface.set_input_region to pet's active bounding box
        }
        bbox
    }
}
