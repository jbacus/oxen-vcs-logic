use anyhow::{Context, Result};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tracing::info;

/// Initialize the database and run migrations
pub async fn init_database(database_url: &str) -> Result<SqlitePool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        info!("Creating database: {}", database_url);
        Sqlite::create_database(database_url)
            .await
            .context("Failed to create database")?;
    }

    // Connect to database
    let pool = SqlitePool::connect(database_url)
        .await
        .context("Failed to connect to database")?;

    // Run migrations
    info!("Running database migrations...");
    run_migrations(&pool).await?;

    info!("Database initialized successfully");
    Ok(pool)
}

/// Run database migrations
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Read and execute migration files
    let migrations_dir = std::path::Path::new("migrations");

    if !migrations_dir.exists() {
        info!("No migrations directory found, skipping migrations");
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "sql")
                .unwrap_or(false)
        })
        .collect();

    // Sort migrations by filename
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();

        info!("Running migration: {}", filename);

        let sql = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read migration: {}", filename))?;

        sqlx::raw_sql(&sql)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to execute migration: {}", filename))?;
    }

    Ok(())
}
