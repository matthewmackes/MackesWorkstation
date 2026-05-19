//! `mackesd` — CLI entry point for the Mesh control plane.
//!
//! Subcommands land alongside their backing Phase 12 substeps. Today
//! only `mackesd migrate` ships (Phase 12.2 store + migrations); the
//! rest follow as substeps complete. We deliberately do NOT register
//! stub commands here — every `mackesd X` either does X or is absent.

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mackesd",
    version,
    about = "Mesh control plane for Mackes XFCE Workstation"
)]
struct Cli {
    /// Override the default `SQLite` store path (defaults to
    /// `$MACKESD_HOME/mackesd.db` or `/var/lib/mackesd/mackesd.db`).
    #[arg(long, env = "MACKESD_DB")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Apply every pending `SQLite` migration against the store.
    ///
    /// Idempotent — running `mackesd migrate` against an up-to-date
    /// store is a no-op that exits 0.
    Migrate,

    /// Print store status: applied-migration count + db path.
    Status,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(mackesd_core::default_db_path);

    match cli.cmd {
        Cmd::Migrate => {
            let conn = mackesd_core::store::open(&db_path)
                .with_context(|| format!("opening store at {}", db_path.display()))?;
            let n = mackesd_core::store::applied_migration_count(&conn)?;
            tracing::info!("store at {} migrated (n={})", db_path.display(), n);
            println!("{n} migrations applied");
        }
        Cmd::Status => {
            let conn = mackesd_core::store::open(&db_path)
                .with_context(|| format!("opening store at {}", db_path.display()))?;
            let n = mackesd_core::store::applied_migration_count(&conn)?;
            println!("db:                 {}", db_path.display());
            println!("migrations applied: {n}");
        }
    }
    Ok(())
}
