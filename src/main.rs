use chelete_lib::database;

fn main() {
    // The native UI arrives in later commits; for now just make sure the
    // database opens and migrates.
    let db = database::init_database().expect("failed to open database");
    drop(db);
}
