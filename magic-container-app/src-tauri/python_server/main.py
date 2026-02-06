import argparse
import uvicorn
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from llama_cpp import Llama
import os

app = FastAPI()
model = None

class ChatRequest(BaseModel):
    message: str
    max_tokens: int = 512
    temperature: float = 0.7

@app.post("/chat")
async def chat(request: ChatRequest):
    global model
    if model is None:
        raise HTTPException(status_code=500, detail="Model not loaded")
    
    # Simple prompt format for Llama 2 / TinyLlama
    # For better quality, we should use specific chat templates (e.g. [INST] ... [/INST])
    # But for a generic test, raw prompting is okay.
    prompt = f"User: {request.message}\nAssistant: "
    
    output = model(
        prompt,
        max_tokens=request.max_tokens,
        stop=["User:", "\nUser"],
        temperature=request.temperature,
        echo=False
    )
    
    text = output['choices'][0]['text']
    return {"reply": text.strip()}

@app.get("/health")
def health():
    return {"status": "ok"}

def load_model(path: str):
    global model
    print(f"Loading model from: {path}")
    # n_gpu_layers=-1 means offload all to GPU if supported (metal/cuda)
    # n_ctx=2048 default context window
    model = Llama(model_path=path, n_gpu_layers=-1, n_ctx=2048, verbose=True)
    print("Model loaded successfully!")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", type=str, required=True, help="Path to the GGUF model file")
    parser.add_argument("--port", type=int, default=8000, help="Port to run the server on")
    args = parser.parse_args()

    if not os.path.exists(args.model):
        print(f"Error: Model file not found at {args.model}")
        exit(1)

    load_model(args.model)
    
    uvicorn.run(app, host="127.0.0.1", port=args.port)
