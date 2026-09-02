use companion_types::PetState;

#[allow(dead_code)]
pub fn get_deterministic_dialogue(state: PetState, context: &str) -> String {
    let ctx_lower = context.to_lowercase();
    match (state, ctx_lower.as_str()) {
        (PetState::Distressed, c) if c.contains("cpu") || c.contains("highcpu") => {
            "Fans are screaming! What are we compiling?".to_string()
        }
        (PetState::Celebrating, c) if c.contains("commit") || c.contains("devcommit") => {
            "Commit pushed! Ship it to production!".to_string()
        }
        (PetState::Working, c) if c.contains("neovim") || c.contains("vim") || c.contains("code") => {
            "Coding hard! Remember to save often.".to_string()
        }
        (PetState::Sleeping, _) => "Zzz... Don't wake me up...".to_string(),
        (PetState::Walking, _) => "Just taking a little stroll around your desktop.".to_string(),
        (PetState::Idle, _) => "Meow! Standing by.".to_string(),
        (PetState::Distressed, _) => "Whoa, system pressure is rising!".to_string(),
        (PetState::Celebrating, _) => "Hooray! Great job!".to_string(),
        (PetState::Working, _) => "Focus mode activated!".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use companion_types::PetState;

    #[test]
    fn test_deterministic_dialogue() {
        assert_eq!(
            get_deterministic_dialogue(PetState::Distressed, "HighCPU"),
            "Fans are screaming! What are we compiling?"
        );
        assert_eq!(
            get_deterministic_dialogue(PetState::Celebrating, "DevCommit"),
            "Commit pushed! Ship it to production!"
        );
        assert_eq!(
            get_deterministic_dialogue(PetState::Sleeping, "anything"),
            "Zzz... Don't wake me up..."
        );
    }
}
