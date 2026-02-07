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

pub async fn install_model(app: AppHandle, model: ModelConfig) -> Result<(), String> {
    // 1. Setup directories
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
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
                if percent % 5 == 0 { // Emit less frequently
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

    // 3. Setup Python Environment (Install Packages)
    let _ = app.emit("install-progress", ProgressPayload {
        model_id: model.id.clone(),
        status: "installing_deps".to_string(),
        progress: 90,
        message: "Installing Python dependencies... This may take a while.".to_string(),
    });

    if let Err(e) = setup_python_env(&model.python_packages) {
        let _ = app.emit("install-progress", ProgressPayload {
            model_id: model.id.clone(),
            status: "error".to_string(),
            progress: 0,
            message: format!("Failed to install dependencies: {}", e),
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

fn setup_python_env(packages: &[String]) -> Result<(), String> {
    if packages.is_empty() {
        return Ok(());
    }

    let python_bin = if cfg!(target_os = "windows") { "python" } else { "python3" };
    let pip_bin = if cfg!(target_os = "windows") { "pip" } else { "pip3" };

    // Check if pip exists
    if Command::new(python_bin).arg("-m").arg("pip").arg("--version").output().is_err() {
        return Err("Python or pip not found. Please install Python 3.10+".to_string());
    }

    // Install packages
    // Use --user to avoid permission issues if not in venv, or rely on system.
    // Ideally we use a venv, but for now global/user install is simpler for "just works".
    let mut cmd = Command::new(python_bin);
    cmd.arg("-m").arg("pip").arg("install").arg("--user"); // Install to user site
    
    // Add extra index url for torch/cuda if needed on windows? No, llama-cpp-python handles it mostly.
    
    for pkg in packages {
        cmd.arg(pkg);
    }

    // For Mac, ensure we upgrade to get metal support if needed?
    // llama-cpp-python usually compiles from source, so cmake is needed.
    // If cmake is missing (which we fixed), it should work.
    // To be safe, force upgrade
    cmd.arg("--upgrade");

    let output = cmd.output().map_err(|e| format!("Failed to run pip: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pip install failed: {}", stderr));
    }

    Ok(())
}
