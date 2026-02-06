// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod specs;
mod models;
mod install_manager;

use specs::SystemSpecs;
use models::ModelConfig;
use tauri::AppHandle;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_system_specs() -> SystemSpecs {
    specs::get_specs()
}

#[tauri::command]
fn get_models() -> Vec<ModelConfig> {
    models::get_available_models()
}

#[tauri::command]
async fn install_model_command(app: AppHandle, model_id: String) -> Result<(), String> {
    let models = models::get_available_models();
    if let Some(model) = models.into_iter().find(|m| m.id == model_id) {
        install_manager::install_model(app, model).await
    } else {
        Err("Model not found".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_system_specs, 
            get_models,
            install_model_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
