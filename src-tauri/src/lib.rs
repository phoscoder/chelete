mod commands;
mod database;
pub mod seed;
mod theme;

use database::DbState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db_state = database::init_database(&app_handle)?;
            app.handle().manage(DbState(db_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_accounts,
            commands::create_account,
            commands::update_account,
            commands::delete_account,
            commands::get_transactions,
            commands::create_transaction,
            commands::update_transaction,
            commands::delete_transaction,
            commands::get_categories,
            commands::create_category,
            commands::update_category,
            commands::delete_category,
            commands::get_overview,
            commands::get_omarchy_theme,
            seed::seed_database_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Chelete");
}
