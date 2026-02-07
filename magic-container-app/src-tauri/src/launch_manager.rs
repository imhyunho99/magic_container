use tauri::{AppHandle, Manager, path::BaseDirectory};
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use crate::models::ModelConfig;
use std::path::PathBuf;
use std::net::TcpListener;

pub struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

// Global state to hold the running python process
pub struct ServiceState {
    pub process: Arc<Mutex<Option<ChildGuard>>>,
}

fn get_python_path(app_data_dir: &PathBuf) -> PathBuf {
    let venv_dir = app_data_dir.join("venv");
    #[cfg(target_os = "windows")]
    return venv_dir.join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    return venv_dir.join("bin").join("python3");
}

fn get_free_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0").ok().and_then(|l| l.local_addr().ok()).map(|a| a.port())
}

pub async fn launch_model(app: AppHandle, model: ModelConfig, state: tauri::State<'_, ServiceState>) -> Result<String, String> {
    // 1. Check if a process is already running, if so, kill it (Drop will handle kill)
    {
        let mut process_guard = state.process.lock().map_err(|_| "Failed to lock mutex")?;
        *process_guard = None; // This drops the previous ChildGuard, killing the process
    }

    // 2. Resolve paths
    let resource_path = app.path().resolve("python_server/main.py", BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve server script: {}", e))?;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let model_path = app_data_dir.join("models").join(&model.id).join("weights").join(&model.source.filename);

    if !model_path.exists() {
        return Err(format!("Model file not found at: {:?}", model_path));
    }

    // 3. Determine python executable (from venv)
    let python_bin = get_python_path(&app_data_dir);

    if !python_bin.exists() {
        return Err(format!("Python venv not found at {:?}. Please reinstall the model.", python_bin));
    }

    // 4. Find free port
    let port = get_free_port().ok_or("Failed to find free port")?;

    // 5. Spawn process
    let child = Command::new(python_bin)
        .arg(resource_path)
        .arg("--model")
        .arg(model_path)
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .map_err(|e| format!("Failed to start python server: {}", e))?;

    // Store the child process wrapped in guard
    {
        let mut process_guard = state.process.lock().map_err(|_| "Failed to lock mutex")?;
        *process_guard = Some(ChildGuard(child));
    }

    // 6. Wait for health check
    let client = reqwest::Client::new();
    let health_url = format!("http://127.0.0.1:{}/health", port);
    
    for _ in 0..30 { // Try for 30 seconds
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Ok(res) = client.get(&health_url).send().await {
            if res.status().is_success() {
                // Return the port so UI can connect
                return Ok(format!("{}", port));
            }
        }
    }

    Err("Server timed out. Check logs.".to_string())
}
