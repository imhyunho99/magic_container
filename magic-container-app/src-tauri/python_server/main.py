import argparse
import uvicorn
import os
import json
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from llama_cpp import Llama

app = FastAPI()

# Enable CORS for local development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

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
    
    # Prompt engineering: Basic
    prompt = f"User: {request.message}\nAssistant: "
    
    def event_generator():
        try:
            stream = model(
                prompt,
                max_tokens=request.max_tokens,
                stop=["User:", "\nUser"],
                temperature=request.temperature,
                stream=True
            )
            for output in stream:
                text = output['choices'][0]['text']
                if text:
                    # Correct SSE format: data: <json>\n\n
                    payload = json.dumps({"token": text}, ensure_ascii=False)
                    yield f"data: {payload}\n\n"
            
            yield "data: [DONE]\n\n"
        except Exception as e:
            error_msg = json.dumps({"error": str(e)}, ensure_ascii=False)
            yield f"data: {error_msg}\n\n"

    return StreamingResponse(event_generator(), media_type="text/event-stream")

@app.get("/health")
def health():
    return {"status": "ok", "model_loaded": model is not None}

def load_model(path: str):
    global model
    print(f"Loading model from: {path}")
    try:
        # n_gpu_layers=-1 attempts to offload to Metal/CUDA
        model = Llama(model_path=path, n_gpu_layers=-1, n_ctx=2048, verbose=True)
        print("Model loaded successfully!")
    except Exception as e:
        print(f"Failed to load model: {e}")

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