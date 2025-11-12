from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse
from pydantic import BaseModel
from typing import Optional, List, Dict, Any
import os
from pathlib import Path
from dotenv import load_dotenv

from src.query_parser import QueryParser
from src.context_builder import ContextBuilder
from src.refactoring_analyzer import RefactoringAnalyzer
from src.prompt_templates import PromptTemplates
from src.clients import ArchitectureDecoderClient

load_dotenv()

app = FastAPI(
    title="Architecture Decoder AI Assistant",
    description="AI-powered assistant for understanding codebase architecture and refactoring impact",
    version="0.1.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize clients
decoder_client = ArchitectureDecoderClient(
    base_url=os.getenv("ARCHITECTURE_DECODER_URL", "http://localhost:8080")
)

query_parser = QueryParser()
context_builder = ContextBuilder(decoder_client)
refactoring_analyzer = RefactoringAnalyzer(decoder_client)
prompt_templates = PromptTemplates()

# OpenAI client
try:
    from openai import OpenAI
    openai_client = OpenAI(api_key=os.getenv("OPENAI_API_KEY")) if os.getenv("OPENAI_API_KEY") else None
except ImportError:
    print("Warning: OpenAI package not installed. Install with: pip install openai")
    openai_client = None


class QueryRequest(BaseModel):
    repository_id: str
    query: str
    max_results: Optional[int] = 10
    include_graph: Optional[bool] = False


class RefactorAnalysisRequest(BaseModel):
    repository_id: str
    target_elements: List[str]
    proposed_changes: str


@app.post("/api/v1/ai/query")
async def ai_query(request: QueryRequest):
    """Natural language query endpoint"""
    try:
        # Parse query intent
        intent = query_parser.parse(request.query)

        # Build context from Architecture Decoder
        context = await context_builder.build(
            repository_id=request.repository_id,
            intent=intent,
            max_results=request.max_results,
            include_graph=request.include_graph
        )

        # Build prompt
        prompt = prompt_templates.build_query_prompt(
            query=request.query,
            context=context,
            intent=intent
        )

        # Call OpenAI if available, otherwise return structured response
        if openai_client:
            try:
                response = await call_openai(prompt)
            except Exception as e:
                print(f"Warning: OpenAI call failed: {e}")
                response = "AI service unavailable. Here's what I found:\n\n" + format_context_summary(context)
        else:
            response = format_context_summary(context)

        # Format response with sources
        graph_data = context.get("graph", {}) if request.include_graph else None
        graph_stats = None
        if graph_data and isinstance(graph_data, dict):
            # Extract statistics if available
            graph_stats = graph_data.get("statistics", {})
            if not graph_stats and "nodes" in graph_data:
                # Calculate basic stats if not provided
                nodes = graph_data.get("nodes", [])
                edges = graph_data.get("edges", [])
                node_types = {}
                for node in nodes:
                    node_type = node.get("type", "unknown")
                    node_types[node_type] = node_types.get(node_type, 0) + 1
                graph_stats = {
                    "total_nodes": len(nodes),
                    "total_edges": len(edges),
                    "node_types": node_types
                }
        
        return {
            "answer": response,
            "sources": context.get("sources", []),
            "graph_context": {
                "statistics": graph_stats,
                "nodes": graph_data.get("nodes", []) if graph_data else [],
                "edges": graph_data.get("edges", []) if graph_data else []
            } if graph_data else None,
            "related_entities": context.get("related", {}),
            "intent": intent.get("intent").value if hasattr(intent.get("intent"), "value") else str(intent.get("intent"))
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/ai/refactor-analysis")
async def refactor_analysis(request: RefactorAnalysisRequest):
    """Refactoring impact analysis endpoint"""
    try:
        # Analyze impact using graph traversal
        impact = await refactoring_analyzer.analyze(
            repository_id=request.repository_id,
            target_elements=request.target_elements,
            proposed_changes=request.proposed_changes
        )

        # Build prompt for AI analysis
        prompt = prompt_templates.build_refactoring_prompt(
            impact=impact,
            proposed_changes=request.proposed_changes
        )

        # Get AI recommendations if available
        if openai_client:
            try:
                ai_recommendations = await call_openai(prompt)
            except Exception as e:
                print(f"Warning: OpenAI call failed: {e}")
                ai_recommendations = None
        else:
            ai_recommendations = None

        # Combine structured analysis with AI insights
        return {
            "impact_analysis": impact,
            "ai_recommendations": ai_recommendations,
            "risk_level": impact.get("risk_level"),
            "recommendations": impact.get("recommendations", [])
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


async def call_openai(prompt: str) -> str:
    """Call OpenAI API"""
    if not openai_client:
        raise Exception("OpenAI client not initialized")

    response = openai_client.chat.completions.create(
        model=os.getenv("OPENAI_MODEL", "gpt-4"),
        messages=[
            {"role": "system", "content": "You are an AI assistant helping developers understand their codebase architecture."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.3,
        max_tokens=1500
    )
    return response.choices[0].message.content


def format_context_summary(context: Dict[str, Any]) -> str:
    """Format context as a summary when AI is unavailable"""
    sources = context.get("sources", [])
    repo = context.get("repository", {})

    summary = f"Found {len(sources)} relevant items in repository '{repo.get('name', 'Unknown')}':\n\n"

    code_elements = [s for s in sources if s.get("type") == "code_element"]
    if code_elements:
        summary += f"**Functions/Code Elements ({len(code_elements)}):**\n"
        for elem in code_elements[:5]:
            summary += f"- {elem.get('name')} ({elem.get('file_path', 'unknown location')})\n"
        summary += "\n"

    services = [s for s in sources if s.get("type") == "service"]
    if services:
        summary += f"**Services ({len(services)}):**\n"
        for svc in services[:5]:
            summary += f"- {svc.get('name')} ({svc.get('provider', 'unknown provider')})\n"
        summary += "\n"

    dependencies = [s for s in sources if s.get("type") == "dependency"]
    if dependencies:
        summary += f"**Dependencies ({len(dependencies)}):**\n"
        for dep in dependencies[:5]:
            summary += f"- {dep.get('name')} (v{dep.get('version', 'unknown')})\n"
        summary += "\n"

    return summary


@app.get("/health")
async def health():
    """Health check endpoint"""
    return {
        "status": "ok",
        "service": "ai-assistant",
        "decoder_url": os.getenv("ARCHITECTURE_DECODER_URL", "http://localhost:8080"),
        "openai_configured": openai_client is not None
    }


@app.get("/api/v1/repositories")
async def get_repositories():
    """Proxy endpoint to get repositories from Architecture Decoder"""
    try:
        repos = await decoder_client.client.get(
            f"{decoder_client.base_url}/api/v1/repositories"
        )
        repos.raise_for_status()
        return repos.json()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch repositories: {str(e)}")


# Serve static files
static_dir = Path(__file__).parent.parent / "static"
if static_dir.exists():
    app.mount("/static", StaticFiles(directory=str(static_dir)), name="static")

@app.get("/")
async def root():
    """Serve the chat UI"""
    static_file = Path(__file__).parent.parent / "static" / "index.html"
    if static_file.exists():
        return FileResponse(str(static_file))
    return {
        "service": "Architecture Decoder AI Assistant",
        "version": "0.1.0",
        "endpoints": {
            "query": "/api/v1/ai/query",
            "refactor_analysis": "/api/v1/ai/refactor-analysis",
            "health": "/health"
        },
        "docs": "/docs",
        "ui": "/static/index.html"
    }


# Cleanup on shutdown
@app.on_event("shutdown")
async def shutdown():
    await decoder_client.close()

