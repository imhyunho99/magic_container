// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod specs;
mod models;
mod install_manager;
mod inference_manager;

use specs::SystemSpecs;
use models::ModelConfig;
use tauri::{AppHandle, Manager};
use inference_manager::InferenceState;

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

#[tauri::command]
async fn load_model_command(app: AppHandle, state: tauri::State<'_, InferenceState>, model_id: String) -> Result<String, String> {
    let models = models::get_available_models();
    if let Some(model) = models.into_iter().find(|m| m.id == model_id) {
        // Resolve full path to model file
        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let model_path = app_data_dir.join("models").join(&model.id).join("weights").join(&model.source.filename);
        
        inference_manager::load_model(model_path.to_string_lossy().to_string(), state).await.map_err(|e| e.to_string())
    } else {
        Err("Model not found".to_string())
    }
}

#[tauri::command]
async fn generate_command(app: AppHandle, state: tauri::State<'_, InferenceState>, prompt: String) -> Result<(), String> {
    inference_manager::generate(prompt, app, state).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(InferenceState::new())
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_system_specs, 
            get_models, 
            install_model_command,
            load_model_command,
            generate_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
