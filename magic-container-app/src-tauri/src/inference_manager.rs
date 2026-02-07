use tauri::{AppHandle, Emitter};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use llm::{Model, InferenceSession, InferenceParameters, InferenceRequest, InferenceResponse};

// Global state to hold the loaded model
pub struct InferenceState {
    pub model: Arc<Mutex<Option<Box<dyn Model>>>>,
}

impl InferenceState {
    pub fn new() -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
        }
    }
}

pub async fn load_model(path: String, state: tauri::State<'_, InferenceState>) -> Result<String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(anyhow!("Model file not found at {:?}", path));
    }

    // Determine model architecture from file or assume Llama for now
    // llm crate can auto-detect, but we usually need to specify architecture.
    // Our models are Llama/TinyLlama/Phi-2.
    // Llama 2 -> Llama
    // TinyLlama -> Llama
    // Phi-2 -> Phi
    
    // For simplicity in this generic container, we'll try Llama first.
    // Real implementation should pass architecture type from ModelConfig.
    let architecture = llm::ModelArchitecture::Llama;
    
    let model = llm::load_dynamic(
        Some(architecture),
        &path,
        llm::TokenizerSource::Embedded,
        llm::ModelParameters::default(),
        llm::load_progress_callback_stdout
    ).map_err(|e| anyhow!("Failed to load model: {}", e))?;

    {
        let mut m = state.model.lock().unwrap();
        *m = Some(model);
    }

    Ok("Model loaded successfully".to_string())
}

#[derive(serde::Serialize, Clone)]
struct TokenPayload {
    token: String,
}

pub async fn generate(prompt: String, app: AppHandle, state: tauri::State<'_, InferenceState>) -> Result<()> {
    // We need to clone the model arc to use it, but Model trait is not easily cloneable.
    // We have to lock and use it. 
    // Inference in llm crate is blocking usually. We should run it in a blocking task.
    
    let model_arc = state.model.clone();
    let prompt_clone = prompt.clone();
    let app_handle = app.clone();

    // Spawn a blocking task for inference
    std::thread::spawn(move || {
        let model_guard = model_arc.lock().unwrap();
        if let Some(model) = model_guard.as_ref() {
            let mut session = model.start_session(Default::default());
            
            let res = session.infer::<std::convert::Infallible>(
                model.as_ref(),
                &mut rand::thread_rng(),
                &llm::InferenceRequest {
                    prompt: (&prompt_clone).into(),
                    parameters: &llm::InferenceParameters::default(),
                    play_back_previous_tokens: false,
                    maximum_token_count: Some(200),
                },
                // Output request
                &mut Default::default(),
                // Callback
                |t| {
                    let _ = app_handle.emit("chat-token", TokenPayload { token: t.to_string() });
                    Ok(llm::InferenceFeedback::Continue)
                }
            );
            
            let _ = app_handle.emit("chat-finished", ());
        }
    });

    Ok(())
}