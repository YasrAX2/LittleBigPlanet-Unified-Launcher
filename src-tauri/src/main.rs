#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod download;
mod metadata;
mod model;
mod release;
mod video;
use tauri::Manager;
fn main() {
    std::panic::set_hook(Box::new(|_| {
        if let Some(d) = std::env::var_os("LOCALAPPDATA") {
            let p = std::path::PathBuf::from(d).join("com.patchworkpod.launcher");
            let _ = std::fs::create_dir_all(&p);
            let _=std::fs::write(p.join("crash.txt"),"Patchwork Pod encountered an internal error. No private paths or account data were recorded.\n");
        }
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .setup(|app| {
            let dir = if cfg!(debug_assertions) {
                std::env::var_os("PATCHWORK_TEST_HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or(app.path().app_local_data_dir()?)
            } else {
                app.path().app_local_data_dir()?
            };
            let store = model::Store::open(dir).map_err(std::io::Error::other)?;
            app.manage(store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::preferences,
            commands::pick,
            commands::import_data,
            commands::clear_data,
            commands::import_engine,
            commands::game_settings,
            commands::validate_game,
            commands::discover_games,
            commands::check_release,
            commands::refresh_requirements,
            commands::install,
            commands::cancel_download,
            commands::maintenance,
            commands::launch,
            commands::open_location,
            commands::open_link,
            commands::backup_config,
            commands::restore_config,
            commands::import_art,
            commands::diagnostics
        ])
        .build(tauri::generate_context!())
        .expect("Unable to start Patchwork Pod")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let s = app.state::<model::Store>();
                if *s.busy.lock().unwrap() || !s.running.lock().unwrap().is_empty() {
                    api.prevent_exit();
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.eval("window.dispatchEvent(new CustomEvent('pod-busy-close'))");
                    }
                }
            }
        });
}
