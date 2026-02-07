use tauri::{AppHandle, Manager, Emitter};
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::Command;
use reqwest::Client;
use futures_util::StreamExt;
use crate::models::ModelConfig;

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    model_id: String,
    status: String,
    progress: u64,
    message: String,
}

// Helper to get venv paths
fn get_venv_paths(app_data_dir: &PathBuf) -> (PathBuf, PathBuf) {
    let venv_dir = app_data_dir.join("venv");
    
    #[cfg(target_os = "windows")]
    let python_executable = venv_dir.join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    let python_executable = venv_dir.join("bin").join("python3");

    #[cfg(target_os = "windows")]
    let pip_executable = venv_dir.join("Scripts").join("pip.exe");
    #[cfg(not(target_os = "windows"))]
    let pip_executable = venv_dir.join("bin").join("pip3");

    (python_executable, pip_executable)
}

pub async fn install_model(app: AppHandle, model: ModelConfig) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    
    // 0. Ensure Venv Exists
    let venv_dir = app_data_dir.join("venv");
    if !venv_dir.exists() {
        let _ = app.emit("install-progress", ProgressPayload {
            model_id: model.id.clone(),
            status: "installing_deps".to_string(),
            progress: 5,
            message: "Creating virtual environment...".to_string(),
        });

        let system_python = if cfg!(target_os = "windows") { "python" } else { "python3" };
        let output = Command::new(system_python)
            .arg("-m")
            .arg("venv")
            .arg(&venv_dir)
            .output()
            .map_err(|e| format!("Failed to create venv: {}", e))?;

        if !output.status.success() {
            return Err(format!("Venv creation failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
    }

    // 1. Setup Model Directories
    let model_dir = app_data_dir.join("models").join(&model.id);
    let weights_dir = model_dir.join("weights");
    fs::create_dir_all(&weights_dir).map_err(|e| format!("Failed to create dirs: {}", e))?;

    let file_path = weights_dir.join(&model.source.filename);

    // 2. Download Model File
    if !file_path.exists() {
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
                let percent = downloaded * 100 / total_size;
                if percent % 5 == 0 {
                    let _ = app.emit("install-progress", ProgressPayload {
                        model_id: model.id.clone(),
                        status: "downloading".to_string(),
                        progress: percent,
                        message: format!("{:.2} MB / {:.2} MB", downloaded as f64 / 1024.0 / 1024.0, total_size as f64 / 1024.0 / 1024.0),
                    });
                }
            }
        }
    }

    // 3. Install Python Dependencies (into venv)
    let _ = app.emit("install-progress", ProgressPayload {
        model_id: model.id.clone(),
        status: "installing_deps".to_string(),
        progress: 90,
        message: "Installing dependencies into venv...".to_string(),
    });

    let (_, pip_executable) = get_venv_paths(&app_data_dir);
    
    if let Err(e) = setup_python_env(&pip_executable, &model.python_packages) {
        let _ = app.emit("install-progress", ProgressPayload {
            model_id: model.id.clone(),
            status: "error".to_string(),
            progress: 0,
            message: format!("Dependency error: {}", e),
        });
        return Err(e);
    }

    // 4. Finish
    let _ = app.emit("install-progress", ProgressPayload {
        model_id: model.id.clone(),
        status: "completed".to_string(),
        progress: 100,
        message: "Installation finished! Ready to Launch.".to_string(),
    });

    Ok(())
}

fn setup_python_env(pip_path: &PathBuf, packages: &[String]) -> Result<(), String> {
    if packages.is_empty() {
        return Ok(());
    }

    if !pip_path.exists() {
        return Err(format!("Pip not found at {:?}. Venv creation might have failed.", pip_path));
    }

    let mut cmd = Command::new(pip_path);
    cmd.arg("install");
    
    for pkg in packages {
        cmd.arg(pkg);
    }
    
    // Fix for macOS specific Metal support or general upgrades
    // We can add --upgrade or specific index-url here if needed.
    // For now standard install.

    let output = cmd.output().map_err(|e| format!("Failed to run pip: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pip install failed: {}", stderr));
    }

    Ok(())
}
