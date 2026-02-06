use tauri::{AppHandle, Manager, Emitter}; // Use Emitter for v2
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use reqwest::Client;
use futures_util::StreamExt;
use crate::models::ModelConfig;

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    model_id: String,
    status: String, // "downloading", "installing_deps", "completed", "error"
    progress: u64,  // 0-100
    message: String,
}

pub async fn install_model(app: AppHandle, model: ModelConfig) -> Result<(), String> {
    // 1. Setup directories
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let model_dir = app_data_dir.join("models").join(&model.id);
    let weights_dir = model_dir.join("weights");

    fs::create_dir_all(&weights_dir).map_err(|e| format!("Failed to create dirs: {}", e))?;

    let file_path = weights_dir.join(&model.source.filename);

    // Check if file already exists
    if file_path.exists() {
        let _ = app.emit("install-progress", ProgressPayload {
            model_id: model.id.clone(),
            status: "completed".to_string(),
            progress: 100,
            message: "Model already installed.".to_string(),
        });
        return Ok(());
    }

    // 2. Download Model File
    let _ = app.emit("install-progress", ProgressPayload {
        model_id: model.id.clone(),
        status: "downloading".to_string(),
        progress: 0,
        message: "Starting download...".to_string(),
    });

    let client = Client::new();
    let res = client
        .get(&model.source.url)
        .send()
        .await
        .map_err(|e| format!("Failed to request model: {}", e))?;

    let total_size = res.content_length().unwrap_or(0);
    let mut stream = res.bytes_stream();
    let mut file = fs::File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Chunk error: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
        
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = (downloaded * 100 / total_size);
            // Emit progress every 1% or so to avoid spamming events? 
            // For now, let's emit freely but maybe debounce in frontend or limit here.
            // A simple way is to emit only when percent changes.
            let _ = app.emit("install-progress", ProgressPayload {
                model_id: model.id.clone(),
                status: "downloading".to_string(),
                progress: percent,
                message: format!("{:.2} MB / {:.2} MB", downloaded as f64 / 1024.0 / 1024.0, total_size as f64 / 1024.0 / 1024.0),
            });
        }
    }

    // 3. Setup Python Environment (Placeholder)
    let _ = app.emit("install-progress", ProgressPayload {
        model_id: model.id.clone(),
        status: "installing_deps".to_string(),
        progress: 90,
        message: "Setting up Python environment... (Skipped in this version)".to_string(),
    });

    // TODO: venv creation logic here

    // 4. Finish
    let _ = app.emit("install-progress", ProgressPayload {
        model_id: model.id.clone(),
        status: "completed".to_string(),
        progress: 100,
        message: "Installation finished!".to_string(),
    });

    Ok(())
}
