# AI Integration Example Implementation

This document shows a concrete example of how to implement the AI assistant integration.

## Quick Start: Python AI Assistant Service

### Directory Structure

```
ai-assistant/
├── src/
│   ├── __init__.py
│   ├── main.py              # FastAPI application
│   ├── query_parser.py      # Parse natural language queries
│   ├── context_builder.py   # Build context from Architecture Decoder API
│   ├── refactoring_analyzer.py  # Analyze refactoring impact
│   ├── prompt_templates.py  # AI prompt templates
│   └── clients.py           # HTTP clients for Architecture Decoder
├── requirements.txt
├── .env.example
└── README.md
```

### Implementation

#### `src/main.py` - FastAPI Application

```python
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Optional, List, Dict, Any
import os
from dotenv import load_dotenv

from src.query_parser import QueryParser
from src.context_builder import ContextBuilder
from src.refactoring_analyzer import RefactoringAnalyzer
from src.prompt_templates import PromptTemplates
from src.clients import ArchitectureDecoderClient

load_dotenv()

app = FastAPI(title="Architecture Decoder AI Assistant")

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

# OpenAI client (or Anthropic)
import openai
openai.api_key = os.getenv("OPENAI_API_KEY")


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

        # Call OpenAI
        response = await call_openai(prompt)

        # Format response with sources
        return {
            "answer": response,
            "sources": context.get("sources", []),
            "graph_context": context.get("graph", {}) if request.include_graph else None,
            "related_entities": context.get("related", {})
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

        # Get AI recommendations
        ai_recommendations = await call_openai(prompt)

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
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[
            {"role": "system", "content": "You are an AI assistant helping developers understand their codebase architecture."},
            {"role": "user", "content": prompt}
        ],
        temperature=0.3,
        max_tokens=1000
    )
    return response.choices[0].message.content


@app.get("/health")
async def health():
    return {"status": "ok", "service": "ai-assistant"}
```

#### `src/query_parser.py` - Query Intent Parsing

```python
import re
from typing import Dict, Any
from enum import Enum

class QueryIntent(Enum):
    FIND_FUNCTIONS = "find_functions"
    LIST_SERVICES = "list_services"
    FIND_DEPENDENCIES = "find_dependencies"
    REFACTORING_IMPACT = "refactoring_impact"
    TOOL_DISCOVERY = "tool_discovery"
    GENERAL = "general"

class QueryParser:
    """Parse natural language queries to identify intent"""

    INTENT_PATTERNS = {
        QueryIntent.FIND_FUNCTIONS: [
            r"what functions",
            r"show.*functions",
            r"list.*functions",
            r"functions.*available",
            r"how.*function",
        ],
        QueryIntent.LIST_SERVICES: [
            r"what services",
            r"which services",
            r"services.*used",
            r"show.*services",
        ],
        QueryIntent.FIND_DEPENDENCIES: [
            r"what dependencies",
            r"which dependencies",
            r"dependencies.*used",
        ],
        QueryIntent.REFACTORING_IMPACT: [
            r"what would break",
            r"what.*impact",
            r"refactor",
            r"rename",
            r"remove",
            r"change",
        ],
        QueryIntent.TOOL_DISCOVERY: [
            r"what tools",
            r"build tools",
            r"test.*tools",
            r"linter",
        ],
    }

    def parse(self, query: str) -> Dict[str, Any]:
        """Parse query and return intent and extracted information"""
        query_lower = query.lower()

        # Identify intent
        intent = QueryIntent.GENERAL
        for intent_type, patterns in self.INTENT_PATTERNS.items():
            for pattern in patterns:
                if re.search(pattern, query_lower):
                    intent = intent_type
                    break
            if intent != QueryIntent.GENERAL:
                break

        # Extract entities (function names, service names, etc.)
        entities = self._extract_entities(query)

        # Extract topics
        topics = self._extract_topics(query)

        return {
            "intent": intent,
            "entities": entities,
            "topics": topics,
            "original_query": query
        }

    def _extract_entities(self, query: str) -> List[str]:
        """Extract potential entity names from query"""
        # Look for quoted strings
        entities = re.findall(r'"([^"]+)"', query)

        # Look for function-like names (camelCase, snake_case)
        entities.extend(re.findall(r'\b([a-z][a-zA-Z0-9_]*)\b', query))

        return list(set(entities))

    def _extract_topics(self, query: str) -> List[str]:
        """Extract topics/keywords from query"""
        # Common topics
        topics = []
        topic_keywords = [
            "authentication", "auth", "storage", "database", "api",
            "firebase", "aws", "stripe", "payment", "email", "notification"
        ]

        query_lower = query.lower()
        for keyword in topic_keywords:
            if keyword in query_lower:
                topics.append(keyword)

        return topics
```

#### `src/context_builder.py` - Build Context from API

```python
from typing import Dict, Any, List
from src.clients import ArchitectureDecoderClient
from src.query_parser import QueryIntent

class ContextBuilder:
    """Build context from Architecture Decoder API"""

    def __init__(self, client: ArchitectureDecoderClient):
        self.client = client

    async def build(
        self,
        repository_id: str,
        intent: Dict[str, Any],
        max_results: int = 10,
        include_graph: bool = False
    ) -> Dict[str, Any]:
        """Build context based on query intent"""
        context = {
            "repository_id": repository_id,
            "sources": [],
            "related": {}
        }

        intent_type = intent.get("intent")
        topics = intent.get("topics", [])
        entities = intent.get("entities", [])

        # Get repository info
        repo_info = await self.client.get_repository(repository_id)
        context["repository"] = repo_info

        # Build context based on intent
        if intent_type == QueryIntent.FIND_FUNCTIONS:
            context.update(await self._build_function_context(
                repository_id, topics, entities, max_results
            ))
        elif intent_type == QueryIntent.LIST_SERVICES:
            context.update(await self._build_service_context(
                repository_id, topics, max_results
            ))
        elif intent_type == QueryIntent.FIND_DEPENDENCIES:
            context.update(await self._build_dependency_context(
                repository_id, topics, max_results
            ))
        elif intent_type == QueryIntent.TOOL_DISCOVERY:
            context.update(await self._build_tool_context(
                repository_id, max_results
            ))
        else:
            # General query - get all relevant data
            context.update(await self._build_general_context(
                repository_id, topics, entities, max_results
            ))

        # Include graph if requested
        if include_graph:
            context["graph"] = await self.client.get_graph(repository_id)

        return context

    async def _build_function_context(
        self, repo_id: str, topics: List[str], entities: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for function discovery"""
        # Get all code elements
        code_elements = await self.client.get_code_elements(repo_id)

        # Filter by topics/entities
        filtered = []
        for element in code_elements:
            if element.get("element_type") != "function":
                continue

            name = element.get("name", "").lower()
            file_path = element.get("file_path", "").lower()

            # Check if matches topics or entities
            matches = False
            if topics:
                matches = any(topic.lower() in name or topic.lower() in file_path
                             for topic in topics)
            if entities:
                matches = matches or any(entity.lower() in name for entity in entities)
            if not topics and not entities:
                matches = True

            if matches:
                filtered.append(element)

        # Get relationships for filtered functions
        sources = []
        for element in filtered[:max_results]:
            relationships = await self.client.get_code_relationships(
                repo_id, code_element_id=element["id"]
            )

            sources.append({
                "type": "code_element",
                "id": element["id"],
                "name": element["name"],
                "signature": element.get("signature"),
                "file_path": element.get("file_path"),
                "line": element.get("line"),
                "relationships": relationships
            })

        # Get related services and dependencies
        related = await self._get_related_entities(repo_id, sources)

        return {
            "sources": sources,
            "related": related
        }

    async def _build_service_context(
        self, repo_id: str, topics: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for service discovery"""
        services = await self.client.get_services(repo_id)

        # Filter by topics
        if topics:
            filtered = [
                s for s in services
                if any(topic.lower() in s.get("name", "").lower()
                      or topic.lower() in s.get("provider", "").lower()
                      for topic in topics)
            ]
        else:
            filtered = services

        sources = [
            {
                "type": "service",
                "id": s["id"],
                "name": s["name"],
                "provider": s.get("provider"),
                "service_type": s.get("service_type"),
                "file_path": s.get("file_path")
            }
            for s in filtered[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_dependency_context(
        self, repo_id: str, topics: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for dependency discovery"""
        dependencies = await self.client.get_dependencies(repo_id)

        # Filter by topics
        if topics:
            filtered = [
                d for d in dependencies
                if any(topic.lower() in d.get("name", "").lower()
                      for topic in topics)
            ]
        else:
            filtered = dependencies

        sources = [
            {
                "type": "dependency",
                "id": d["id"],
                "name": d["name"],
                "version": d.get("version"),
                "package_manager": d.get("package_manager")
            }
            for d in filtered[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_tool_context(
        self, repo_id: str, max_results: int
    ) -> Dict[str, Any]:
        """Build context for tool discovery"""
        tools = await self.client.get_tools(repo_id)

        sources = [
            {
                "type": "tool",
                "id": t["id"],
                "name": t["name"],
                "tool_type": t.get("tool_type"),
                "config_file": t.get("config_file")
            }
            for t in tools[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_general_context(
        self, repo_id: str, topics: List[str], entities: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build general context for any query"""
        # Get a mix of code elements, services, dependencies
        code_elements = await self.client.get_code_elements(repo_id)
        services = await self.client.get_services(repo_id)
        dependencies = await self.client.get_dependencies(repo_id)

        sources = []

        # Add relevant code elements
        for element in code_elements[:max_results//3]:
            if topics or entities:
                name = element.get("name", "").lower()
                if any(topic.lower() in name for topic in topics) or \
                   any(entity.lower() in name for entity in entities):
                    sources.append({
                        "type": "code_element",
                        "id": element["id"],
                        "name": element["name"],
                        "file_path": element.get("file_path")
                    })

        # Add relevant services
        for service in services[:max_results//3]:
            sources.append({
                "type": "service",
                "id": service["id"],
                "name": service["name"],
                "provider": service.get("provider")
            })

        # Add relevant dependencies
        for dep in dependencies[:max_results//3]:
            sources.append({
                "type": "dependency",
                "id": dep["id"],
                "name": dep["name"],
                "version": dep.get("version")
            })

        return {
            "sources": sources,
            "related": {}
        }

    async def _get_related_entities(
        self, repo_id: str, sources: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Get related services and dependencies for code elements"""
        related_services = set()
        related_dependencies = set()

        for source in sources:
            if source.get("relationships"):
                for rel in source["relationships"]:
                    if rel.get("target_type") == "service":
                        related_services.add(rel.get("target_name"))
                    elif rel.get("target_type") == "dependency":
                        related_dependencies.add(rel.get("target_name"))

        return {
            "services": list(related_services),
            "dependencies": list(related_dependencies)
        }
```

#### `src/clients.py` - Architecture Decoder API Client

```python
import httpx
from typing import Dict, Any, List, Optional

class ArchitectureDecoderClient:
    """Client for Architecture Decoder API"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.client = httpx.AsyncClient(timeout=30.0)

    async def get_repository(self, repo_id: str) -> Dict[str, Any]:
        """Get repository details"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}"
        )
        response.raise_for_status()
        return response.json()

    async def get_code_elements(
        self, repo_id: str, element_type: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get code elements for repository"""
        url = f"{self.base_url}/api/v1/repositories/{repo_id}/code/elements"
        if element_type:
            url += f"?type={element_type}"
        response = await self.client.get(url)
        response.raise_for_status()
        return response.json()

    async def get_code_relationships(
        self, repo_id: str, code_element_id: Optional[str] = None,
        target_type: Optional[str] = None, target_id: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get code relationships"""
        url = f"{self.base_url}/api/v1/repositories/{repo_id}/code/relationships"
        params = {}
        if code_element_id:
            params["code_element_id"] = code_element_id
        if target_type:
            params["target_type"] = target_type
        if target_id:
            params["target_id"] = target_id

        response = await self.client.get(url, params=params)
        response.raise_for_status()
        return response.json()

    async def get_services(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get services for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/services"
        )
        response.raise_for_status()
        return response.json()

    async def get_dependencies(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get dependencies for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/dependencies"
        )
        response.raise_for_status()
        return response.json()

    async def get_tools(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get tools for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/tools"
        )
        response.raise_for_status()
        return response.json()

    async def get_graph(self, repo_id: str) -> Dict[str, Any]:
        """Get knowledge graph for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/graph"
        )
        response.raise_for_status()
        return response.json()
```

#### `src/prompt_templates.py` - AI Prompt Templates

```python
from typing import Dict, Any

class PromptTemplates:
    """Templates for AI prompts"""

    def build_query_prompt(
        self, query: str, context: Dict[str, Any], intent: Dict[str, Any]
    ) -> str:
        """Build prompt for general query"""
        repo = context.get("repository", {})
        sources = context.get("sources", [])

        prompt = f"""You are an AI assistant helping developers understand their codebase architecture.

Repository: {repo.get('name', 'Unknown')}
Language: {repo.get('language', 'Unknown')}

User Query: {query}

Based on the following codebase analysis, answer the user's question:

"""

        # Add code elements
        code_elements = [s for s in sources if s.get("type") == "code_element"]
        if code_elements:
            prompt += "\n## Available Functions/Code Elements:\n\n"
            for elem in code_elements:
                prompt += f"- **{elem.get('name')}**\n"
                if elem.get('signature'):
                    prompt += f"  Signature: `{elem['signature']}`\n"
                if elem.get('file_path'):
                    prompt += f"  Location: `{elem['file_path']}`\n"
                if elem.get('relationships'):
                    prompt += f"  Relationships: {len(elem['relationships'])} found\n"
                prompt += "\n"

        # Add services
        services = [s for s in sources if s.get("type") == "service"]
        if services:
            prompt += "\n## Services Used:\n\n"
            for svc in services:
                prompt += f"- **{svc.get('name')}** ({svc.get('provider')})\n"
            prompt += "\n"

        # Add dependencies
        dependencies = [s for s in sources if s.get("type") == "dependency"]
        if dependencies:
            prompt += "\n## Dependencies:\n\n"
            for dep in dependencies:
                prompt += f"- **{dep.get('name')}** (v{dep.get('version', 'unknown')})\n"
            prompt += "\n"

        prompt += """
Please provide a clear, concise answer to the user's question. Include:
- Specific function/entity names and locations
- How elements relate to each other
- Usage examples if available
- Any important warnings or considerations

Be thorough but concise."""

        return prompt

    def build_refactoring_prompt(
        self, impact: Dict[str, Any], proposed_changes: str
    ) -> str:
        """Build prompt for refactoring analysis"""
        prompt = f"""You are an AI assistant analyzing refactoring impact.

Proposed Changes: {proposed_changes}

## Impact Analysis:

**Affected Functions**: {len(impact.get('affected_functions', []))}
**Affected Services**: {len(impact.get('affected_services', []))}
**Affected Dependencies**: {len(impact.get('affected_dependencies', []))}

### Affected Functions:
"""

        for func in impact.get('affected_functions', [])[:10]:
            prompt += f"- {func.get('name')} ({func.get('file_path')})\n"
            prompt += f"  Relationship: {func.get('relationship')}\n"
            prompt += f"  Risk: {func.get('risk')}\n\n"

        prompt += f"""
### Call Chains:
"""
        for chain in impact.get('call_chains', [])[:5]:
            prompt += f"- {' → '.join(chain.get('path', []))}\n"

        prompt += """
Based on this impact analysis, provide:
1. Risk assessment (low/medium/high) with reasoning
2. Step-by-step refactoring recommendations
3. Testing considerations
4. Potential breaking changes
5. Migration strategy

Be specific about what could break and why."""

        return prompt
```

#### `src/refactoring_analyzer.py` - Refactoring Impact Analysis

```python
from typing import Dict, Any, List
from src.clients import ArchitectureDecoderClient

class RefactoringAnalyzer:
    """Analyze refactoring impact"""

    def __init__(self, client: ArchitectureDecoderClient):
        self.client = client

    async def analyze(
        self,
        repository_id: str,
        target_elements: List[str],
        proposed_changes: str
    ) -> Dict[str, Any]:
        """Analyze refactoring impact"""
        impact = {
            "affected_functions": [],
            "affected_services": [],
            "affected_dependencies": [],
            "call_chains": [],
            "risk_level": "low",
            "recommendations": []
        }

        # For each target element, find what depends on it
        for element_id in target_elements:
            # Get element details
            relationships = await self.client.get_code_relationships(
                repository_id, code_element_id=element_id
            )

            # Find callers (functions that call this)
            callers = await self._find_callers(repository_id, element_id)

            # Find callees (functions this calls)
            callees = await self._find_callees(repository_id, element_id)

            # Build call chains
            chains = await self._build_call_chains(
                repository_id, element_id, callers
            )

            impact["affected_functions"].extend(callers)
            impact["call_chains"].extend(chains)

        # Assess risk
        impact["risk_level"] = self._assess_risk(impact)

        # Generate recommendations
        impact["recommendations"] = self._generate_recommendations(impact)

        return impact

    async def _find_callers(
        self, repo_id: str, element_id: str
    ) -> List[Dict[str, Any]]:
        """Find functions that call this element"""
        # Get code calls
        calls = await self.client.get_code_elements(repo_id)

        # Filter calls where target is our element
        callers = []
        for call in calls:
            if call.get("target_id") == element_id:
                callers.append({
                    "id": call.get("source_id"),
                    "name": call.get("source_name"),
                    "file_path": call.get("source_file_path"),
                    "relationship": "calls",
                    "risk": "high"  # Direct call = high risk
                })

        return callers

    async def _find_callees(
        self, repo_id: str, element_id: str
    ) -> List[Dict[str, Any]]:
        """Find functions called by this element"""
        # Get relationships where this element is the source
        relationships = await self.client.get_code_relationships(
            repo_id, code_element_id=element_id
        )

        callees = []
        for rel in relationships:
            if rel.get("relationship_type") == "calls":
                callees.append({
                    "id": rel.get("target_id"),
                    "name": rel.get("target_name"),
                    "relationship": "called_by"
                })

        return callees

    async def _build_call_chains(
        self, repo_id: str, element_id: str, callers: List[Dict[str, Any]]
    ) -> List[Dict[str, Any]]:
        """Build call chains starting from callers"""
        chains = []

        for caller in callers[:5]:  # Limit to avoid explosion
            chain = await self._traverse_up(
                repo_id, caller["id"], [element_id]
            )
            if chain:
                chains.append({
                    "path": chain,
                    "depth": len(chain)
                })

        return chains

    async def _traverse_up(
        self, repo_id: str, element_id: str, path: List[str]
    ) -> List[str]:
        """Traverse call graph upward"""
        if len(path) > 5:  # Limit depth
            return path

        # Get callers of this element
        callers = await self._find_callers(repo_id, element_id)

        if not callers:
            return path

        # Take first caller and continue
        next_caller = callers[0]
        return await self._traverse_up(
            repo_id, next_caller["id"], path + [next_caller["name"]]
        )

    def _assess_risk(self, impact: Dict[str, Any]) -> str:
        """Assess overall risk level"""
        affected_count = len(impact.get("affected_functions", []))
        chain_depth = max(
            [c.get("depth", 0) for c in impact.get("call_chains", [])],
            default=0
        )

        if affected_count > 10 or chain_depth > 4:
            return "high"
        elif affected_count > 5 or chain_depth > 2:
            return "medium"
        else:
            return "low"

    def _generate_recommendations(
        self, impact: Dict[str, Any]
    ) -> List[str]:
        """Generate refactoring recommendations"""
        recommendations = []

        affected_count = len(impact.get("affected_functions", []))

        if affected_count > 0:
            recommendations.append(
                f"Update {affected_count} affected function(s) before refactoring"
            )

        if impact.get("risk_level") == "high":
            recommendations.append(
                "Consider creating a wrapper function first"
            )
            recommendations.append(
                "Add deprecation warnings before removal"
            )

        recommendations.append("Run full test suite after changes")
        recommendations.append("Update documentation")

        return recommendations
```

### Usage Example

```bash
# Start Architecture Decoder
cd wavelength-arch-decoder
cargo run --release

# Start AI Assistant (in another terminal)
cd ai-assistant
pip install -r requirements.txt
uvicorn src.main:app --port 8000

# Query example
curl -X POST http://localhost:8000/api/v1/ai/query \
  -H "Content-Type: application/json" \
  -d '{
    "repository_id": "repo-123",
    "query": "What functions use Firebase?",
    "max_results": 10
  }'

# Refactoring analysis example
curl -X POST http://localhost:8000/api/v1/ai/refactor-analysis \
  -H "Content-Type: application/json" \
  -d '{
    "repository_id": "repo-123",
    "target_elements": ["func-123"],
    "proposed_changes": "Rename getAdminStorage to getFirebaseStorage"
  }'
```

This provides a complete, working example of how to integrate AI assistance with your Architecture Decoder!

