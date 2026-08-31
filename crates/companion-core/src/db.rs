use companion_types::{PetState, VitalsVector};
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(home).join(".local/share/desktop-pet");
        fs::create_dir_all(&dir).ok();
        let db_path = dir.join("pet_state.db");
        Self::open(db_path)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS pet_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                current_state TEXT NOT NULL,
                energy REAL NOT NULL,
                hunger REAL NOT NULL,
                focus REAL NOT NULL,
                affection REAL NOT NULL,
                stress REAL NOT NULL,
                dialogue TEXT,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vitals_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                energy REAL NOT NULL,
                hunger REAL NOT NULL,
                focus REAL NOT NULL,
                affection REAL NOT NULL,
                stress REAL NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn save_pet_state(
        &self,
        state: PetState,
        vitals: &VitalsVector,
        dialogue: Option<&str>,
        timestamp: u64,
    ) -> Result<()> {
        let state_str = serde_json::to_string(&state).unwrap_or_else(|_| "Idle".to_string());

        self.conn.execute(
            "INSERT INTO pet_state (id, current_state, energy, hunger, focus, affection, stress, dialogue, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                current_state = excluded.current_state,
                energy = excluded.energy,
                hunger = excluded.hunger,
                focus = excluded.focus,
                affection = excluded.affection,
                stress = excluded.stress,
                dialogue = excluded.dialogue,
                updated_at = excluded.updated_at",
            params![
                state_str,
                vitals.energy,
                vitals.hunger,
                vitals.focus,
                vitals.affection,
                vitals.stress,
                dialogue,
                timestamp as i64
            ],
        )?;

        Ok(())
    }

    pub fn append_vitals_history(&self, vitals: &VitalsVector, timestamp: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO vitals_history (timestamp, energy, hunger, focus, affection, stress)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                timestamp as i64,
                vitals.energy,
                vitals.hunger,
                vitals.focus,
                vitals.affection,
                vitals.stress
            ],
        )?;

        Ok(())
    }

    pub fn load_latest_state(&self) -> Result<Option<(PetState, VitalsVector, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT current_state, energy, hunger, focus, affection, stress, dialogue FROM pet_state WHERE id = 1",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let state_str: String = row.get(0)?;
            let state: PetState = serde_json::from_str(&state_str).unwrap_or(PetState::Idle);
            let vitals = VitalsVector {
                energy: row.get(1)?,
                hunger: row.get(2)?,
                focus: row.get(3)?,
                affection: row.get(4)?,
                stress: row.get(5)?,
            };
            let dialogue: Option<String> = row.get(6)?;
            Ok(Some((state, vitals, dialogue)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_save_and_load() {
        let db = Database::open(":memory:").unwrap();
        let vitals = VitalsVector::default();
        let state = PetState::Working;

        db.save_pet_state(state, &vitals, Some("Working hard!"), 1000)
            .unwrap();

        let loaded = db.load_latest_state().unwrap().unwrap();
        assert_eq!(loaded.0, PetState::Working);
        assert_eq!(loaded.1, vitals);
        assert_eq!(loaded.2.as_deref(), Some("Working hard!"));
    }

    #[test]
    fn test_vitals_history() {
        let db = Database::open(":memory:").unwrap();
        let vitals = VitalsVector::default();
        db.append_vitals_history(&vitals, 1000).unwrap();

        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM vitals_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
