mod db;
mod ssh;
mod terminal;
mod monitor;
mod commands;
mod crypto;

use db::DbConn;
use terminal::TerminalManager;
use commands::port_forward::PortForwardManager;
use commands::telnet::TelnetManager;
use commands::local_terminal::LocalTerminalManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            let app_dir = app_handle.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("myterm.db");
            app_handle.manage(DbConn::new(db_path));
            app_handle.manage(TerminalManager::new());
            app_handle.manage(PortForwardManager::new());
            app_handle.manage(TelnetManager::new());
            app_handle.manage(LocalTerminalManager::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connections
            commands::connections::get_groups,
            commands::connections::create_group,
            commands::connections::update_group,
            commands::connections::delete_group,
            commands::connections::get_connections,
            commands::connections::create_connection,
            commands::connections::update_connection,
            commands::connections::delete_connection,
            commands::connections::test_connection,
            commands::connections::search_connections,
            commands::connections::collect_server_info,
            // Terminal
            commands::terminal::connect_terminal,
            commands::terminal::disconnect_terminal,
            commands::terminal::terminal_write,
            commands::terminal::terminal_resize,
            // SFTP
            commands::sftp::sftp_list_dir,
            commands::sftp::sftp_read_file,
            commands::sftp::sftp_write_file,
            commands::sftp::sftp_remove_file,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_mkdir,
            // Monitor
            commands::monitor::get_monitor_data,
            // Notes
            commands::notes::get_notes,
            commands::notes::create_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            // AI
            commands::ai::get_ai_conversations,
            commands::ai::create_ai_conversation,
            commands::ai::delete_ai_conversation,
            commands::ai::get_ai_messages,
            commands::ai::save_ai_message,
            // Settings
            commands::settings::get_settings,
            commands::settings::set_setting,
            commands::settings::get_setting,
            // Port Forwarding
            commands::port_forward::create_port_forward,
            commands::port_forward::get_port_forwards,
            commands::port_forward::close_port_forward,
            // Ping
            commands::ping::ping_host,
            // RDP
            commands::rdp::connect_rdp,
            // Telnet
            commands::telnet::connect_telnet,
            commands::telnet::telnet_write,
            commands::telnet::disconnect_telnet,
            // Quick Commands
            commands::quick_commands::get_quick_commands,
            commands::quick_commands::create_quick_command,
            commands::quick_commands::update_quick_command,
            commands::quick_commands::delete_quick_command,
            // Import/Export
            commands::import_export::export_connections,
            commands::import_export::import_connections,
            // Local Terminal
            commands::local_terminal::open_local_terminal,
            commands::local_terminal::local_terminal_write,
            commands::local_terminal::close_local_terminal,
            // Local filesystem for SFTP two-pane view
            commands::local_fs::list_local_dir,
            commands::local_fs::write_local_file,
            commands::local_fs::remove_local_file,
            commands::local_fs::rename_local_file,
            commands::local_fs::create_local_dir,
            // Screenshot (dev)
            commands::screenshot::take_screenshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
