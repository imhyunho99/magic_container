use tauri::{AppHandle, Manager, path::BaseDirectory};
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use crate::models::ModelConfig;
use std::path::PathBuf;

// Global state to hold the running python process
pub struct ServiceState {
    pub process: Arc<Mutex<Option<Child>>>,
}

pub async fn launch_model(app: AppHandle, model: ModelConfig, state: tauri::State<'_, ServiceState>) -> Result<String, String> {
    // 1. Check if a process is already running, if so, kill it
    {
        let mut process_guard = state.process.lock().map_err(|_| "Failed to lock mutex")?;
        if let Some(mut child) = process_guard.take() {
            let _ = child.kill(); // Kill previous model
        }
    }

    // 2. Resolve paths
    let resource_path = app.path().resolve("python_server/main.py", BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve server script: {}", e))?;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let model_path = app_data_dir.join("models").join(&model.id).join("weights").join(&model.source.filename);

    if !model_path.exists() {
        return Err(format!("Model file not found at: {:?}", model_path));
    }

    // 3. Determine python executable
    let python_bin = if cfg!(target_os = "windows") { "python" } else { "python3" };

    // 4. Spawn process
    let child = Command::new(python_bin)
        .arg(resource_path)
        .arg("--model")
        .arg(model_path)
        .arg("--port")
        .arg("8000")
        .spawn()
        .map_err(|e| format!("Failed to start python server: {}", e))?;

    // Store the child process
    {
        let mut process_guard = state.process.lock().map_err(|_| "Failed to lock mutex")?;
        *process_guard = Some(child);
    }

    // 5. Wait for health check
    let client = reqwest::Client::new();
    for _ in 0..30 { // Try for 30 seconds
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Ok(res) = client.get("http://localhost:8000/health").send().await {
            if res.status().is_success() {
                return Ok("Server started".to_string());
            }
        }
    }

    Err("Server timed out".to_string())
}
