# AI Assistant for Architecture Decoder

AI-powered assistant that helps developers understand their codebase architecture and assess refactoring impact using natural language queries.

## Features

- **Natural Language Queries**: Ask questions about your codebase in plain English
- **Function Discovery**: Find available functions, tools, and services
- **Refactoring Impact Analysis**: Assess risks before making changes
- **Context-Aware Responses**: Uses knowledge graph data from Architecture Decoder

## Quick Start

### Prerequisites

- Python 3.8+
- Architecture Decoder running on `http://localhost:8080`
- OpenAI API key (optional, for AI-powered responses)

### Installation

```bash
# Install dependencies
pip install -r requirements.txt

# Copy environment template
cp .env.example .env

# Edit .env and add your OpenAI API key
# ARCHITECTURE_DECODER_URL=http://localhost:8080
# OPENAI_API_KEY=your_key_here
```

### Running

```bash
# Start the AI assistant
uvicorn src.main:app --reload --port 8000

# Or use Python directly
python -m uvicorn src.main:app --reload --port 8000
```

The service will be available at `http://localhost:8000`

## Usage

### Natural Language Query

```bash
curl -X POST http://localhost:8000/api/v1/ai/query \
  -H "Content-Type: application/json" \
  -d '{
    "repository_id": "repo-123",
    "query": "What functions use Firebase?",
    "max_results": 10
  }'
```

### Refactoring Impact Analysis

```bash
curl -X POST http://localhost:8000/api/v1/ai/refactor-analysis \
  -H "Content-Type: application/json" \
  -d '{
    "repository_id": "repo-123",
    "target_elements": ["func-123"],
    "proposed_changes": "Rename getAdminStorage to getFirebaseStorage"
  }'
```

### Example Queries

- "What functions are available in this repository?"
- "Which services does this codebase use?"
- "Show me all functions that use Firebase"
- "What dependencies are used by the authentication module?"
- "What build tools are configured?"
- "What would break if I rename `getAdminStorage`?"

## API Endpoints

### POST `/api/v1/ai/query`

Natural language query endpoint.

**Request:**
```json
{
  "repository_id": "repo-123",
  "query": "What functions use Firebase?",
  "max_results": 10,
  "include_graph": false
}
```

**Response:**
```json
{
  "answer": "Based on the codebase analysis...",
  "sources": [...],
  "related_entities": {...},
  "intent": "find_functions"
}
```

### POST `/api/v1/ai/refactor-analysis`

Refactoring impact analysis endpoint.

**Request:**
```json
{
  "repository_id": "repo-123",
  "target_elements": ["func-123"],
  "proposed_changes": "Rename function"
}
```

**Response:**
```json
{
  "impact_analysis": {
    "affected_functions": [...],
    "risk_level": "medium",
    "recommendations": [...]
  },
  "ai_recommendations": "..."
}
```

### GET `/health`

Health check endpoint.

## Configuration

Set environment variables in `.env`:

```bash
# Architecture Decoder API URL
ARCHITECTURE_DECODER_URL=http://localhost:8080

# OpenAI API Key (optional)
OPENAI_API_KEY=sk-...

# Server Configuration
PORT=8000
HOST=0.0.0.0

# OpenAI Model (optional, defaults to gpt-4)
OPENAI_MODEL=gpt-4
```

## How It Works

1. **Query Parsing**: Natural language query is parsed to identify intent
2. **Context Building**: Relevant data is fetched from Architecture Decoder API
3. **Prompt Engineering**: Context is formatted into a prompt for the AI
4. **AI Response**: OpenAI generates a contextual answer
5. **Response Formatting**: Answer is returned with source attribution

## Development

### Project Structure

```
ai-assistant/
├── src/
│   ├── main.py              # FastAPI application
│   ├── query_parser.py      # Parse natural language queries
│   ├── context_builder.py   # Build context from API
│   ├── refactoring_analyzer.py  # Analyze refactoring impact
│   ├── prompt_templates.py  # AI prompt templates
│   └── clients.py           # HTTP clients for Architecture Decoder
├── requirements.txt
├── .env.example
└── README.md
```

### Testing

```bash
# Test health endpoint
curl http://localhost:8000/health

# Test query endpoint
curl -X POST http://localhost:8000/api/v1/ai/query \
  -H "Content-Type: application/json" \
  -d '{"repository_id": "test-repo", "query": "test"}'
```

## Troubleshooting

### "OpenAI client not initialized"

- Make sure `OPENAI_API_KEY` is set in `.env`
- Install OpenAI package: `pip install openai`
- The service will still work without OpenAI, but responses will be less detailed

### "Could not connect to Architecture Decoder"

- Verify Architecture Decoder is running on the configured URL
- Check `ARCHITECTURE_DECODER_URL` in `.env`
- Test the Architecture Decoder API directly: `curl http://localhost:8080/api/v1/repositories`

### "Repository not found"

- Make sure the repository ID exists in Architecture Decoder
- List repositories: `curl http://localhost:8080/api/v1/repositories`

## License

MIT License - See LICENSE file for details.

