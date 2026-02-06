use tauri::{AppHandle, Emitter};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;

// Global state to hold the loaded model
pub struct InferenceState {
    pub backend: LlamaBackend,
    pub model: Arc<Mutex<Option<LlamaModel>>>,
    // Context is usually per-session, but for single user we can keep one.
    // However, LlamaContext is not Send/Sync easily without Mutex.
    // For simplicity, we will reload context or keep it protected.
    // Actually, storing Model is enough, we can create Context per chat session or keep one.
    // Let's keep one context for the active chat.
    pub context: Arc<Mutex<Option<LlamaContext>>>,
}

impl InferenceState {
    pub fn new() -> Self {
        Self {
            backend: LlamaBackend::init().unwrap(),
            model: Arc::new(Mutex::new(None)),
            context: Arc::new(Mutex::new(None)),
        }
    }
}

pub async fn load_model(path: String, state: tauri::State<'_, InferenceState>) -> Result<String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(anyhow!("Model file not found at {:?}", path));
    }

    // Load Model
    let params = LlamaModelParams::default();
    // Enable GPU offload if Metal/CUDA is enabled (n_gpu_layers > 0)
    // For now use default, library usually detects best settings or we can tune params.
    // params.n_gpu_layers = 99; // Example for GPU

    let model = LlamaModel::load_from_file(&state.backend, &path, &params)
        .map_err(|e| anyhow!("Failed to load model: {}", e))?;

    // Create Context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(2048); // 2k context window
    
    let context = model.new_context(&state.backend, ctx_params)
        .map_err(|e| anyhow!("Failed to create context: {}", e))?;

    // Store in state
    {
        let mut m = state.model.lock().unwrap();
        *m = Some(model);
    }
    {
        let mut c = state.context.lock().unwrap();
        *c = Some(context);
    }

    Ok("Model loaded successfully".to_string())
}

#[derive(serde::Serialize, Clone)]
struct TokenPayload {
    token: String,
}

pub async fn generate(prompt: String, app: AppHandle, state: tauri::State<'_, InferenceState>) -> Result<()> {
    let model_guard = state.model.lock().unwrap();
    let model = model_guard.as_ref().ok_or(anyhow!("No model loaded"))?;
    
    let mut ctx_guard = state.context.lock().unwrap();
    let ctx = ctx_guard.as_mut().ok_or(anyhow!("No context active"))?;

    // Tokenize prompt
    // Add special tokens for Llama 2 chat format if needed (BOS, etc.)
    // For simplicity, raw tokenize.
    let tokens_list = model.str_to_token(&prompt, true)
        .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

    // Clear context KV cache to start fresh? 
    // Or we should manage history. For now, let's just evaluate this prompt (stateless-ish).
    // Ideally we keep history. 
    // For this demo: Evaluate prompt -> Generate.
    
    // Clear KV cache (optional, if we want fresh start)
    ctx.clear_kv_cache();

    // Prepare batch
    let mut batch = LlamaBatch::new(512, 1);
    let last_index = tokens_list.len() as i32 - 1;

    for (i, token) in tokens_list.iter().enumerate() {
        let is_last = i as i32 == last_index;
        batch.add(*token, i as i32, &[0], is_last)?;
    }

    ctx.decode(&mut batch).map_err(|e| anyhow!("Decode failed: {}", e))?;

    // Generation loop
    let mut n_cur = batch.n_tokens();
    let max_tokens = 200; // Limit generation

    for i in 0..max_tokens {
        // Sample next token
        let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
        let candidates_p = LlamaTokenDataArray::from_iter(candidates, false);
        
        let new_token_id = ctx.sample_token_greedy(candidates_p);

        // Check for EOS
        if new_token_id == model.token_eos() {
            break;
        }

        // Decode token to string
        let token_str = model.token_to_str(new_token_id).unwrap_or("".to_string());

        // Emit to frontend
        let _ = app.emit("chat-token", TokenPayload { token: token_str.clone() });

        // Feed back into model
        batch.clear();
        batch.add(new_token_id, n_cur, &[0], true)?;
        n_cur += 1;

        ctx.decode(&mut batch).map_err(|e| anyhow!("Decode loop failed: {}", e))?;
    }

    let _ = app.emit("chat-finished", ());

    Ok(())
}
