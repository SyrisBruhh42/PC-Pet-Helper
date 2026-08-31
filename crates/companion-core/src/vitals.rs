use companion_types::{PetState, VitalsVector};

#[derive(Debug, Clone, PartialEq)]
pub struct VitalsEngine {
    vitals: VitalsVector,
}

impl Default for VitalsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VitalsEngine {
    pub fn new() -> Self {
        Self {
            vitals: VitalsVector::default(),
        }
    }

    pub fn from_vector(vitals: VitalsVector) -> Self {
        let mut engine = Self { vitals };
        engine.clamp();
        engine
    }

    pub fn vector(&self) -> &VitalsVector {
        &self.vitals
    }

    pub fn vector_mut(&mut self) -> &mut VitalsVector {
        &mut self.vitals
    }

    pub fn apply_tick(&mut self, dt_secs: f32, state: PetState, cpu_usage: f32) {
        let dt = dt_secs.max(0.0).min(60.0);

        // Hunger: + 0.05 * dt
        self.vitals.hunger += 0.05 * dt;

        // Affection: - 0.02 * dt
        self.vitals.affection -= 0.02 * dt;

        // Energy
        if state == PetState::Sleeping {
            self.vitals.energy += 0.50 * dt;
        } else {
            self.vitals.energy -= 0.05 * dt;
        }

        // Focus
        if state == PetState::Working {
            self.vitals.focus += 0.10 * dt;
        } else {
            self.vitals.focus -= 0.05 * dt;
        }

        // Stress: cpu_usage > 85.0 or hunger > 80.0
        if cpu_usage > 85.0 || self.vitals.hunger > 80.0 {
            self.vitals.stress += 0.20 * dt;
        } else {
            self.vitals.stress -= 0.10 * dt;
        }

        self.clamp();
    }

    pub fn apply_user_click(&mut self) {
        self.vitals.affection += 5.0;
        self.vitals.energy += 2.0;
        self.vitals.stress -= 5.0;
        self.clamp();
    }

    fn clamp(&mut self) {
        self.vitals.energy = self.vitals.energy.clamp(0.0, 100.0);
        self.vitals.hunger = self.vitals.hunger.clamp(0.0, 100.0);
        self.vitals.focus = self.vitals.focus.clamp(0.0, 100.0);
        self.vitals.affection = self.vitals.affection.clamp(0.0, 100.0);
        self.vitals.stress = self.vitals.stress.clamp(0.0, 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_vector() {
        let engine = VitalsEngine::new();
        assert_eq!(engine.vector().energy, 100.0);
        assert_eq!(engine.vector().hunger, 0.0);
        assert_eq!(engine.vector().focus, 50.0);
        assert_eq!(engine.vector().affection, 80.0);
        assert_eq!(engine.vector().stress, 0.0);
    }

    #[test]
    fn test_tick_idle() {
        let mut engine = VitalsEngine::new();
        engine.apply_tick(1.0, PetState::Idle, 10.0);
        assert!((engine.vector().energy - 99.95).abs() < 1e-4);
        assert!((engine.vector().hunger - 0.05).abs() < 1e-4);
        assert!((engine.vector().focus - 49.95).abs() < 1e-4);
        assert!((engine.vector().affection - 79.98).abs() < 1e-4);
        assert!((engine.vector().stress - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_user_click() {
        let mut engine = VitalsEngine::new();
        engine.vector_mut().stress = 10.0;
        engine.vector_mut().energy = 50.0;
        engine.apply_user_click();
        assert_eq!(engine.vector().affection, 85.0);
        assert_eq!(engine.vector().energy, 52.0);
        assert_eq!(engine.vector().stress, 5.0);
    }

    #[test]
    fn test_dt_cap() {
        let mut engine = VitalsEngine::new();
        engine.apply_tick(120.0, PetState::Idle, 10.0);
        // Effective dt should be capped at 60.0s
        assert!((engine.vector().hunger - 3.0).abs() < 1e-4);
    }
}
