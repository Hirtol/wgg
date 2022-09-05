//! This build script takes care of ensuring our database entities are *always* up to date.
//! It does this by creating a temporary SQLite compile database, running the migrations, and then generating the DB entities
//! based on this temporary database.
//!
//! This entire sequence is re-done every time anything changes in the `CRATE_ROOT/migrations` directory.
use sea_orm_cli::DateTimeCrate;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::env;
use std::path::Path;
use std::time::Duration;

#[tokio::main]
pub async fn main() {
    // First establish re-run rules.
    let current_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed=../migrations");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let compile_db = Path::new(&out_dir).join("compile.db");
    // Delete old DB
    if compile_db.exists() {
        std::fs::remove_file(&compile_db).unwrap();
    }

    run(Path::new(&current_dir), &compile_db).await;
}

async fn run(current_dir: &Path, compile_db: &Path) {
    // Create the compile-database
    initialise_database(compile_db).await;

    // Run the entity generation.
    // Note, Cargo docs generally recommend *not* changing anything outside of OUT_DIR, so we're breaking that guideline here.
    // This is fine in this instance, as we're a binary crate anyway, so no one else should be affected.
    let wgg_db_entity_path = current_dir
        .parent()
        .unwrap()
        .join("wgg_db_entity")
        .join("src")
        .join("entity");

    let generate_cmd = sea_orm_cli::GenerateSubcommands::Entity {
        compact_format: false,
        expanded_format: true,
        include_hidden_tables: false,
        tables: None,
        ignore_tables: vec![],
        max_connections: 1,
        output_dir: wgg_db_entity_path.to_string_lossy().into_owned(),
        database_schema: "".to_string(),
        database_url: database_url(compile_db),
        with_serde: "none".to_string(),
        with_copy_enums: false,
        date_time_crate: DateTimeCrate::Chrono,
    };

    sea_orm_cli::commands::run_generate_command(generate_cmd, false)
        .await
        .unwrap()
}

async fn initialise_database(db_path: &Path) {
    std::fs::create_dir_all(&db_path.parent().unwrap()).unwrap();

    let options = database_url(db_path)
        .parse::<SqliteConnectOptions>()
        .unwrap()
        .journal_mode(SqliteJournalMode::Memory)
        .synchronous(SqliteSynchronous::Normal) // Since we're in WAL mode
        .busy_timeout(Duration::from_secs(10));

    let pool = SqlitePoolOptions::new()
        .max_connections(std::thread::available_parallelism().unwrap().get() as u32)
        .connect_with(options)
        .await
        .unwrap();

    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
}

fn database_url(db_path: &Path) -> String {
    format!(
        "sqlite:///{}?mode=rwc",
        db_path
            .to_str()
            .expect("Invalid database path specified in config or ENV")
    )
}
