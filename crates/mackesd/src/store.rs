//! `SQLite` persistence (Phase 12.2 — locked 2026-05-19 in 12.A.2).
//!
//! Owns connection lifecycle, migration application, and the helpers
//! every other module uses to read or write the store. WAL mode is
//! enabled in `0001_init.sql` so readers (the panel's in-process
//! library link) never block writers (the daemon's reconcile loop).

use std::path::Path;

use anyhow::Context;
use rusqlite::Connection;

use crate::Result;

/// Numbered migration. Run in order; once applied, the version is
/// recorded in `schema_migrations`.
struct Migration {
    version: i64,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: include_str!("../migrations/0001_init.sql"),
}];

/// Open the store at `path`, creating its parent directory if needed
/// and applying every pending migration before returning.
///
/// # Errors
///
/// Returns an error if the parent directory cannot be created, the
/// database cannot be opened (e.g. permission denied), or any
/// migration fails to apply.
pub fn open(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db parent dir {}", parent.display()))?;
    }
    let conn =
        Connection::open(path).with_context(|| format!("opening sqlite db {}", path.display()))?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open an in-memory store. Used by tests + dry-run paths so the real
/// `/var/lib/mackesd/mackesd.db` never gets clobbered.
///
/// # Errors
///
/// Returns an error if the in-memory connection can't open or migrate.
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory().context("opening in-memory sqlite")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Apply every pending migration. Idempotent — already-applied
/// versions are skipped.
///
/// # Errors
///
/// Returns an error if a migration's SQL fails to execute or if the
/// `schema_migrations` table can't be created.
pub fn migrate(conn: &Connection) -> Result<()> {
    // Bootstrap the tracking table so we can read its current state
    // even on a fresh database.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\
             version    INTEGER PRIMARY KEY,\
             applied_at TEXT NOT NULL\
         );",
    )
    .context("creating schema_migrations table")?;

    let applied: std::collections::HashSet<i64> = {
        let mut stmt = conn
            .prepare("SELECT version FROM schema_migrations")
            .context("listing applied migrations")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, i64>(0))
            .context("iterating applied migrations")?;
        rows.collect::<rusqlite::Result<_>>()
            .context("reading applied migration row")?
    };

    for m in MIGRATIONS {
        if applied.contains(&m.version) {
            continue;
        }
        conn.execute_batch(m.sql)
            .with_context(|| format!("applying migration {}", m.version))?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)",
            (m.version, chrono::Utc::now().to_rfc3339()),
        )
        .with_context(|| format!("recording migration {}", m.version))?;
    }
    Ok(())
}

/// Number of migrations that have run against this connection.
///
/// # Errors
///
/// Returns an error if the `schema_migrations` table can't be queried.
pub fn applied_migration_count(conn: &Connection) -> Result<i64> {
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .context("counting applied migrations")?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_applies_every_migration() {
        let conn = open_in_memory().expect("open");
        let n = applied_migration_count(&conn).expect("count");
        assert_eq!(usize::try_from(n).expect("count fits"), MIGRATIONS.len());
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = open_in_memory().expect("open");
        let before = applied_migration_count(&conn).expect("count");
        migrate(&conn).expect("re-migrate");
        let after = applied_migration_count(&conn).expect("count");
        assert_eq!(before, after, "re-running migrate must be a no-op");
    }

    #[test]
    fn nodes_table_accepts_only_known_roles() {
        let conn = open_in_memory().expect("open");
        // Bogus role rejected by CHECK constraint.
        let res = conn.execute(
            "INSERT INTO nodes (node_id, name, public_key, enrolled_at, role) \
             VALUES ('n1','one','pk','2026-01-01T00:00:00Z','grand-admiral')",
            [],
        );
        assert!(res.is_err(), "bogus role must violate CHECK constraint");
    }

    #[test]
    fn desired_config_state_machine_constraint() {
        let conn = open_in_memory().expect("open");
        let bad = conn.execute(
            "INSERT INTO desired_config (author, message, spec_json, state, created_at) \
             VALUES ('me','m','{}','rejected-by-policy','2026-01-01T00:00:00Z')",
            [],
        );
        assert!(
            bad.is_err(),
            "unknown deployment state must be rejected by CHECK"
        );
    }
}
