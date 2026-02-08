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
            id: "qwen2.5-1.5b-instruct-v2".to_string(), // Changed ID to force re-download
            name: "Qwen2.5 1.5B Instruct",
            description: "Best-in-class lightweight model. Excellent Korean support and reasoning. Runs smoothly on 4GB+ RAM laptops.",
            version: "Q4_K_M".to_string(),
            task_type: "text-generation".to_string(),
            requirements: ModelRequirements {
                min_ram: 4 * 1024 * 1024 * 1024, // 4 GB
                min_vram: 2 * 1024 * 1024 * 1024, // 2 GB (optional)
                disk_space: 1 * 1024 * 1024 * 1024, // ~1 GB
            },
            source: ModelSource {
                url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
                filename: "qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
            },
            python_packages: vec![
                "llama-cpp-python".to_string(),
                "uvicorn".to_string(),
                "fastapi".to_string()
            ],
        },
        ModelConfig {
            id: "gemma-2-2b-it-gguf".to_string(),
            name: "Google Gemma 2 2B".to_string(),
            description: "Google's latest lightweight open model. Strong logical reasoning and summarization. Good for office tasks.".to_string(),
            version: "Q4_K_M".to_string(),
            task_type: "text-generation".to_string(),
            requirements: ModelRequirements {
                min_ram: 4 * 1024 * 1024 * 1024, // 4 GB
                min_vram: 2 * 1024 * 1024 * 1024, // 2 GB (optional)
                disk_space: 2 * 1024 * 1024 * 1024, // ~1.7 GB
            },
            source: ModelSource {
                url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf".to_string(),
                filename: "gemma-2-2b-it-Q4_K_M.gguf".to_string(),
            },
            python_packages: vec![
                "llama-cpp-python".to_string(),
                "uvicorn".to_string(),
                "fastapi".to_string()
            ],
        },
        ModelConfig {
            id: "whisper-tiny".to_string(),
            name: "Whisper Tiny".to_string(),
            description: "OpenAI's lightweight speech recognition model. Converts voice to text very quickly.".to_string(),
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
        }
    ]
}
