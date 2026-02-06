import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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

function App() {
  const [specs, setSpecs] = useState<SystemSpecs | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchSpecs() {
      try {
        const specs = await invoke<SystemSpecs>("get_system_specs");
        setSpecs(specs);
      } catch (error) {
        console.error("Failed to fetch system specs:", error);
      } finally {
        setLoading(false);
      }
    }
    fetchSpecs();
  }, []);

  const formatBytes = (bytes: number) => {
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB";
  };

  return (
    <main className="container">
      <h1>Magic Container</h1>
      <p>System Diagnostics</p>

      {loading ? (
        <p>Loading system specs...</p>
      ) : specs ? (
        <div className="specs-container">
          <div className="spec-item">
            <strong>OS:</strong> {specs.os_name} {specs.os_version}
          </div>
          <div className="spec-item">
            <strong>CPU:</strong> {specs.cpu_model} ({specs.cpu_cores} Cores)
          </div>
          <div className="spec-item">
            <strong>Memory:</strong> {formatBytes(specs.used_memory)} / {formatBytes(specs.total_memory)}
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
              <strong>GPU:</strong> No dedicated GPU detected (or not supported on this OS yet).
            </div>
          )}
        </div>
      ) : (
        <p>Could not load system specifications.</p>
      )}
    </main>
  );
}

export default App;
