use companion_types::PetState;

#[derive(Debug, Clone)]
pub struct PetPhysics {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub sprite_size: f32,
    pub ground_level: f32,
}

impl PetPhysics {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let sprite_size = 64.0;
        let ground_level = 32.0; // In a 96px strip: ground level y=32 leaves 64px height (32..96)
        Self {
            x: (screen_width - sprite_size) / 2.0,
            y: ground_level,
            vx: 1.2,
            vy: 0.0,
            screen_width,
            screen_height,
            sprite_size,
            ground_level,
        }
    }

    pub fn tick(&mut self, state: PetState) {
        match state {
            PetState::Idle => {
                self.y = self.ground_level;
                self.vy = 0.0;
            }
            PetState::Walking => {
                self.y = self.ground_level;
                self.vy = 0.0;
                if self.vx == 0.0 {
                    self.vx = 1.2;
                }
                self.x += self.vx;
                let max_x = (self.screen_width - self.sprite_size).max(0.0);
                if self.x <= 0.0 {
                    self.x = 0.0;
                    self.vx = self.vx.abs();
                } else if self.x >= max_x {
                    self.x = max_x;
                    self.vx = -self.vx.abs();
                }
            }
            PetState::Working | PetState::Sleeping | PetState::Distressed => {
                self.y = self.ground_level;
                self.vy = 0.0;
            }
            PetState::Celebrating => {
                // Vertical jumping physics
                self.y += self.vy;
                self.vy += 0.5; // gravity g = 0.5 px/frame^2
                if self.y >= self.ground_level {
                    self.y = self.ground_level;
                    self.vy = -7.0; // jump velocity
                }
            }
        }
    }

    pub fn bounding_box(&self) -> (i32, i32, i32, i32) {
        let ix = self.x.round() as i32;
        let iy = self.y.round() as i32;
        let size = self.sprite_size as i32;
        (ix, iy, ix + size, iy + size)
    }

    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        let (x1, y1, x2, y2) = self.bounding_box();
        px >= x1 && px < x2 && py >= y1 && py < y2
    }
}
