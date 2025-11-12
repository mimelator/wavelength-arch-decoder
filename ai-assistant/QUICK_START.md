# Quick Start Guide

## 🚀 Getting Started in 3 Steps

### 1. Setup Environment

```bash
cd ai-assistant

# Copy environment template
cp .env.example .env

# Edit .env and add your OpenAI API key
# ARCHITECTURE_DECODER_URL=http://localhost:8080
# OPENAI_API_KEY=sk-your-key-here
```

### 2. Start Architecture Decoder

Make sure your Architecture Decoder is running:

```bash
# In the wavelength-arch-decoder directory
cargo run --release
```

It should be available at `http://localhost:8080`

### 3. Start AI Assistant

```bash
# Option A: Use the startup script (recommended)
./start.sh

# Option B: Manual start
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
uvicorn src.main:app --reload --port 8000
```

The AI Assistant will be available at `http://localhost:8000`

## 📝 Example Queries

### Using curl

```bash
# Query: What functions use Firebase?
curl -X POST http://localhost:8000/api/v1/ai/query \
  -H "Content-Type: application/json" \
  -d '{
    "repository_id": "your-repo-id",
    "query": "What functions use Firebase?",
    "max_results": 10
  }'

# Refactoring analysis
curl -X POST http://localhost:8000/api/v1/ai/refactor-analysis \
  -H "Content-Type: application/json" \
  -d '{
    "repository_id": "your-repo-id",
    "target_elements": ["func-123"],
    "proposed_changes": "Rename getAdminStorage to getFirebaseStorage"
  }'
```

### Using Python

```python
import httpx

# Query
response = httpx.post(
    "http://localhost:8000/api/v1/ai/query",
    json={
        "repository_id": "your-repo-id",
        "query": "What functions use Firebase?",
        "max_results": 10
    }
)
print(response.json()["answer"])
```

### Using the Web UI

Visit `http://localhost:8000/docs` for interactive API documentation (Swagger UI)

## 🔍 Finding Your Repository ID

First, get the list of repositories from Architecture Decoder:

```bash
curl http://localhost:8080/api/v1/repositories
```

Look for the `id` field of the repository you want to query.

## 💡 Example Queries

- "What functions are available in this repository?"
- "Which services does this codebase use?"
- "Show me all functions that use Firebase"
- "What dependencies are used by the authentication module?"
- "What build tools are configured?"
- "What would break if I rename `getAdminStorage`?"
- "Can I safely remove this function?"

## 🐛 Troubleshooting

### "Could not connect to Architecture Decoder"

- Verify Architecture Decoder is running: `curl http://localhost:8080/health`
- Check `ARCHITECTURE_DECODER_URL` in `.env`

### "Repository not found"

- List repositories: `curl http://localhost:8080/api/v1/repositories`
- Use the correct repository ID

### "OpenAI API error"

- Verify `OPENAI_API_KEY` is set in `.env`
- Check your OpenAI account has credits
- The service will still work without OpenAI, but responses will be less detailed

## 📚 Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check [AI_INTEGRATION_PROPOSAL.md](../docs/AI_INTEGRATION_PROPOSAL.md) for architecture details
- See [AI_INTEGRATION_EXAMPLE.md](../docs/AI_INTEGRATION_EXAMPLE.md) for code examples

