import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface GpuInfo {
  name: string;
  vram_total: number;
  vram_used: number;
  driver_version?: string;
  cuda_version?: string;
}

interface SystemSpecs {
  os_name: string;
  os_version: string;
  cpu_model: string;
  cpu_cores: number;
  total_memory: number;
  used_memory: number;
  gpus: GpuInfo[];
}

interface ModelRequirements {
  min_ram: number;
  min_vram: number;
  disk_space: number;
}

interface ModelSource {
  url: string;
  filename: string;
}

interface ModelConfig {
  id: string;
  name: string;
  description: string;
  version: string;
  task_type: string;
  requirements: ModelRequirements;
  source: ModelSource;
  python_packages: string[];
}

interface ProgressPayload {
  model_id: string;
  status: "downloading" | "installing_deps" | "completed" | "error";
  progress: number;
  message: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<"system" | "models">("system");
  const [specs, setSpecs] = useState<SystemSpecs | null>(null);
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [loadingSpecs, setLoadingSpecs] = useState(true);
  const [installProgress, setInstallProgress] = useState<Record<string, ProgressPayload>>({});

  useEffect(() => {
    async function fetchData() {
      try {
        const specsData = await invoke<SystemSpecs>("get_system_specs");
        setSpecs(specsData);
        
        const modelsData = await invoke<ModelConfig[]>("get_models");
        setModels(modelsData);
      } catch (error) {
        console.error("Failed to fetch data:", error);
      } finally {
        setLoadingSpecs(false);
      }
    }
    fetchData();

    // Listen for install progress
    const unlisten = listen<ProgressPayload>("install-progress", (event) => {
      setInstallProgress((prev) => ({
        ...prev,
        [event.payload.model_id]: event.payload,
      }));
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const handleInstall = async (modelId: string) => {
    try {
      // Set initial state to avoid flicker
      setInstallProgress((prev) => ({
        ...prev,
        [modelId]: { model_id: modelId, status: "downloading", progress: 0, message: "Starting..." },
      }));
      await invoke("install_model_command", { modelId });
    } catch (error) {
      console.error("Install failed:", error);
      alert("Installation failed: " + error);
      // Reset progress on error
      setInstallProgress((prev) => {
        const newState = { ...prev };
        delete newState[modelId];
        return newState;
      });
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 GB";
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB";
  };

  const checkCompatibility = (requirements: ModelRequirements) => {
    if (!specs) return { compatible: false, reason: "System specs not loaded" };
    
    if (specs.total_memory < requirements.min_ram) {
      return { compatible: false, reason: `Insufficient RAM (Needs ${formatBytes(requirements.min_ram)})` };
    }

    if (requirements.min_vram > 0) {
        const totalVram = specs.gpus.reduce((sum, gpu) => sum + gpu.vram_total, 0);
        if (specs.gpus.length > 0 && totalVram > 0 && totalVram < requirements.min_vram) {
             return { compatible: false, reason: `Insufficient VRAM (Needs ${formatBytes(requirements.min_vram)})` };
        }
    }
    
    return { compatible: true };
  };

  return (
    <main className="container">
      <header className="app-header">
        <h1>Magic Container</h1>
        <nav className="tabs">
          <button 
            className={activeTab === "system" ? "active" : ""} 
            onClick={() => setActiveTab("system")}
          >
            System Info
          </button>
          <button 
            className={activeTab === "models" ? "active" : ""} 
            onClick={() => setActiveTab("models")}
          >
            Model Hub
          </button>
        </nav>
      </header>

      <div className="content">
        {loadingSpecs ? (
          <p>Loading system information...</p>
        ) : (
          <>
            {activeTab === "system" && specs && (
              <div className="specs-container">
                <h2>System Diagnostics</h2>
                <div className="spec-item">
                  <strong>OS:</strong> {specs.os_name} {specs.os_version}
                </div>
                <div className="spec-item">
                  <strong>CPU:</strong> {specs.cpu_model} ({specs.cpu_cores} Cores)
                </div>
                <div className="spec-item">
                  <strong>Memory:</strong> {formatBytes(specs.used_memory)} used / {formatBytes(specs.total_memory)} total
                </div>
                
                {specs.gpus.length > 0 ? (
                  <div className="gpu-section">
                    <h3>GPUs</h3>
                    {specs.gpus.map((gpu, index) => (
                      <div key={index} className="spec-item gpu-item">
                        <strong>{gpu.name}</strong>
                        <br />
                        VRAM: {formatBytes(gpu.vram_used)} / {formatBytes(gpu.vram_total)}
                        {gpu.driver_version && <div>Driver: {gpu.driver_version}</div>}
                        {gpu.cuda_version && <div>CUDA: {gpu.cuda_version}</div>}
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="spec-item">
                    <strong>GPU:</strong> No dedicated GPU detected.
                  </div>
                )}
              </div>
            )}

            {activeTab === "models" && (
              <div className="models-container">
                <h2>Available Models</h2>
                <div className="model-grid">
                  {models.map((model) => {
                    const { compatible, reason } = checkCompatibility(model.requirements);
                    const progress = installProgress[model.id];

                    return (
                      <div key={model.id} className={`model-card ${compatible ? "compatible" : "incompatible"}`}>
                        <div className="card-header">
                            <span className="task-badge">{model.task_type}</span>
                            <h3>{model.name}</h3>
                        </div>
                        <p className="description">{model.description}</p>
                        <div className="requirements">
                            <span>RAM: {formatBytes(model.requirements.min_ram)}</span>
                            {model.requirements.min_vram > 0 && <span>VRAM: {formatBytes(model.requirements.min_vram)}</span>}
                        </div>
                        <div className="action-area">
                            {progress ? (
                                <div className="progress-container">
                                    <div className="progress-bar">
                                        <div 
                                            className="progress-fill" 
                                            style={{ width: `${progress.progress}%` }}
                                        ></div>
                                    </div>
                                    <div className="progress-text">
                                        {progress.status === "completed" ? "Ready to Launch" : progress.message}
                                    </div>
                                    {progress.status === "completed" && (
                                         <button className="launch-btn" onClick={() => alert("Launching logic coming soon!")}>
                                            Launch
                                         </button>
                                    )}
                                </div>
                            ) : compatible ? (
                                <button className="install-btn" onClick={() => handleInstall(model.id)}>
                                    Download & Install
                                </button>
                            ) : (
                                <div className="warning">Incompatible: {reason}</div>
                            )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </main>
  );
}

export default App;
