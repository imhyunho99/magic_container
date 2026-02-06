use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelRequirements {
    pub min_ram: u64,  // bytes
    pub min_vram: u64, // bytes
    pub disk_space: u64, // bytes
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSource {
    pub url: String, // HuggingFace URL or direct link
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub task_type: String, // e.g., "text-generation", "speech-to-text"
    pub requirements: ModelRequirements,
    pub source: ModelSource,
    pub python_packages: Vec<String>,
}

// Hardcoded initial model list for testing
pub fn get_available_models() -> Vec<ModelConfig> {
    vec![
        ModelConfig {
            id: "whisper-tiny".to_string(),
            name: "Whisper Tiny".to_string(),
            description: "OpenAI's lightweight speech recognition model. Extremely fast and runs on almost any CPU. Great for testing.".to_string(),
            version: "tiny".to_string(),
            task_type: "speech-to-text".to_string(),
            requirements: ModelRequirements {
                min_ram: 1 * 1024 * 1024 * 1024, // 1 GB
                min_vram: 0,
                disk_space: 100 * 1024 * 1024, // ~100 MB
            },
            source: ModelSource {
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/master/ggml-tiny.bin".to_string(),
                filename: "ggml-tiny.bin".to_string(),
            },
            python_packages: vec![
                "openai-whisper".to_string(),
                "soundfile".to_string()
            ],
        },
        ModelConfig {
            id: "llama-2-7b-chat-gguf".to_string(),
            name: "Llama 2 7B Chat".to_string(),
            description: "A quantized LLM optimized for chat. Good balance of performance and resource usage.".to_string(),
            version: "Q4_K_M".to_string(),
            task_type: "text-generation".to_string(),
            requirements: ModelRequirements {
                min_ram: 8 * 1024 * 1024 * 1024, // 8 GB
                min_vram: 6 * 1024 * 1024 * 1024, // 6 GB recommended
                disk_space: 5 * 1024 * 1024 * 1024, // ~5 GB
            },
            source: ModelSource {
                url: "https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_K_M.gguf".to_string(),
                filename: "llama-2-7b-chat.Q4_K_M.gguf".to_string(),
            },
            python_packages: vec![
                "llama-cpp-python".to_string(),
                "uvicorn".to_string(),
                "fastapi".to_string()
            ],
        }
    ]
}
