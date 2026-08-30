use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    let db_path = find_database();

    if !db_path.exists() {
        eprintln!("Database not found at: {}", db_path.display());
        eprintln!("Run the app first to create the database.");
        std::process::exit(1);
    }

    println!("Seeding database at: {}", db_path.display());

    let conn = Connection::open(&db_path).expect("failed to open database");
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();

    match chelete_lib::seed::seed_database(&conn) {
        Ok(true) => println!("Database seeded successfully."),
        Ok(false) => println!("Database already contains data. Skipping seed."),
        Err(e) => {
            eprintln!("Seed failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn find_database() -> PathBuf {
    // Try common Tauri app data locations
    let candidates = vec![
        // Linux: ~/.local/share/com.chelete.app/chelete.db
        dirs::data_dir()
            .map(|d| d.join("com.chelete.app").join("chelete.db")),
        // Linux fallback: ~/.local/share/chelete/chelete.db
        dirs::data_dir()
            .map(|d| d.join("chelete").join("chelete.db")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return candidate;
        }
    }

    // Default path (first candidate or panic)
    dirs::data_dir()
        .expect("cannot determine data directory")
        .join("com.chelete.app")
        .join("chelete.db")
}
