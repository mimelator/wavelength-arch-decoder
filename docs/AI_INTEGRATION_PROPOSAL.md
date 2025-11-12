# AI Integration Proposal for Wavelength Architecture Decoder

## Overview

This document proposes an AI assistant integration that helps developers understand available tools/functions and assess refactoring risks by leveraging the knowledge graph data from the Architecture Decoder.

## Goals

1. **Natural Language Query Interface**: Ask questions about the codebase in plain English
2. **Tool & Function Discovery**: Understand what functions, tools, and services are available
3. **Refactoring Impact Analysis**: Identify risks and dependencies before making changes
4. **Intelligent Recommendations**: Get AI-powered suggestions based on code relationships

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────┐
│              AI Assistant Service                        │
│  (Python/Node.js service with OpenAI/Anthropic API)     │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
┌───────▼──────┐ ┌─────▼──────┐ ┌─────▼──────┐
│  Query       │ │  Context   │ │  Analysis  │
│  Parser      │ │  Builder    │ │  Engine    │
└───────┬──────┘ └─────┬───────┘ └─────┬──────┘
        │               │               │
        └───────────────┼───────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│         Architecture Decoder API                        │
│    (REST & GraphQL endpoints)                          │
│                                                         │
│  • /api/v1/repositories/{id}/code/elements            │
│  • /api/v1/repositories/{id}/code/relationships       │
│  • /api/v1/repositories/{id}/graph                    │
│  • /api/v1/repositories/{id}/dependencies              │
│  • /api/v1/repositories/{id}/services                  │
│  • /api/v1/repositories/{id}/tools                    │
│  • GraphQL queries                                     │
└─────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. AI Assistant Service (`ai-assistant/`)

A separate service (Python or Node.js) that:
- Accepts natural language queries
- Queries the Architecture Decoder API for context
- Constructs prompts with relevant code relationships
- Calls AI API (OpenAI/Anthropic) with structured context
- Returns formatted responses

**Key Files:**
- `ai-assistant/src/query_parser.py` - Parse natural language queries
- `ai-assistant/src/context_builder.py` - Build context from knowledge graph
- `ai-assistant/src/refactoring_analyzer.py` - Analyze refactoring impact
- `ai-assistant/src/prompt_templates.py` - AI prompt templates
- `ai-assistant/src/api.py` - REST API endpoints

#### 2. New Architecture Decoder Endpoints

Add to existing Rust service:

**`/api/v1/ai/query`** - Natural language query endpoint
```rust
POST /api/v1/ai/query
{
  "repository_id": "repo-123",
  "query": "What functions use Firebase?",
  "context_types": ["code", "services", "dependencies"] // optional
}

Response:
{
  "answer": "The following functions use Firebase...",
  "sources": [
    {
      "type": "code_element",
      "id": "func-123",
      "name": "getAdminStorage",
      "confidence": 0.85
    }
  ],
  "graph_context": { ... }
}
```

**`/api/v1/ai/refactor-analysis`** - Refactoring impact analysis
```rust
POST /api/v1/ai/refactor-analysis
{
  "repository_id": "repo-123",
  "target_elements": ["func-123", "class-456"],
  "proposed_changes": "Rename function and update dependencies"
}

Response:
{
  "impact_analysis": {
    "affected_functions": [...],
    "affected_services": [...],
    "affected_dependencies": [...],
    "risk_level": "medium",
    "recommendations": [...]
  }
}
```

#### 3. GraphQL Enhancements

Add new queries to existing GraphQL schema:

```graphql
type Query {
  # ... existing queries ...

  # AI-focused queries
  codeElementSearch(
    repositoryId: String!
    query: String!
    elementType: String
  ): [CodeElementType!]!

  refactoringImpact(
    repositoryId: String!
    targetElementIds: [String!]!
  ): RefactoringImpact!

  functionDependencies(
    repositoryId: String!
    functionId: String!
  ): FunctionDependencies!
}

type RefactoringImpact {
  affectedElements: [CodeElementType!]!
  affectedServices: [ServiceType!]!
  affectedDependencies: [DependencyType!]!
  callChain: [CodeCallType!]!
  riskLevel: RiskLevel!
  recommendations: [String!]!
}

type FunctionDependencies {
  function: CodeElementType!
  calls: [CodeElementType!]!
  calledBy: [CodeElementType!]!
  usesServices: [ServiceType!]!
  usesDependencies: [DependencyType!]!
  securityEntities: [SecurityEntityType!]!
}
```

## Implementation Plan

### Phase 1: Basic Query Interface

**Goal**: Enable natural language queries about codebase

**Steps**:
1. Create `ai-assistant/` directory structure
2. Implement query parser (identify intent: "find functions", "list services", etc.)
3. Build context builder that queries Architecture Decoder API
4. Create prompt templates for common queries
5. Add `/api/v1/ai/query` endpoint to Architecture Decoder
6. Test with sample queries

**Example Queries**:
- "What functions are available in this repository?"
- "Which services does this codebase use?"
- "Show me all functions that use Firebase"
- "What dependencies are used by the authentication module?"

### Phase 2: Function & Tool Discovery

**Goal**: Help developers discover available functions and tools

**Steps**:
1. Enhance context builder to include:
   - Function signatures with parameters
   - Tool configurations
   - Usage examples from code
2. Create specialized prompts for function discovery
3. Add function search endpoint with semantic matching
4. Generate "function catalog" responses

**Example Queries**:
- "What authentication functions are available?"
- "How do I use the Firebase storage functions?"
- "What build tools are configured?"
- "Show me all API endpoints in this codebase"

### Phase 3: Refactoring Impact Analysis

**Goal**: Assess risks before refactoring

**Steps**:
1. Implement graph traversal to find affected elements
2. Build dependency chain analyzer
3. Create risk assessment algorithm
4. Generate refactoring recommendations
5. Add `/api/v1/ai/refactor-analysis` endpoint

**Example Queries**:
- "What would break if I rename `getAdminStorage`?"
- "What depends on the Firebase service?"
- "Can I safely remove this function?"
- "What's the impact of updating React to version 18?"

### Phase 4: Advanced Analysis

**Goal**: Proactive insights and recommendations

**Steps**:
1. Add code smell detection
2. Implement security risk analysis
3. Create migration path suggestions
4. Add "what-if" scenario analysis

**Example Queries**:
- "Are there any security risks in this refactoring?"
- "What's the best way to migrate from Firebase to AWS?"
- "Which functions have high coupling?"
- "What code should I review before this change?"

## API Design

### REST Endpoints

#### Query Endpoint
```http
POST /api/v1/ai/query
Content-Type: application/json

{
  "repository_id": "repo-123",
  "query": "What functions use Firebase?",
  "max_results": 10,
  "include_graph": true
}

Response: 200 OK
{
  "answer": "Based on the codebase analysis, the following functions use Firebase...",
  "sources": [
    {
      "type": "code_element",
      "id": "func-123",
      "name": "getAdminStorage",
      "file_path": "src/utils/storage.js",
      "line": 45,
      "confidence": 0.85,
      "evidence": "imports 'firebase/app', uses 'firebase.storage()'"
    }
  ],
  "graph_context": {
    "nodes": [...],
    "edges": [...]
  },
  "related_entities": {
    "services": ["Firebase"],
    "dependencies": ["firebase@9.0.0"]
  }
}
```

#### Refactoring Analysis Endpoint
```http
POST /api/v1/ai/refactor-analysis
Content-Type: application/json

{
  "repository_id": "repo-123",
  "target_elements": ["func-123", "class-456"],
  "proposed_changes": {
    "action": "rename",
    "old_name": "getAdminStorage",
    "new_name": "getFirebaseStorage"
  },
  "include_call_chains": true
}

Response: 200 OK
{
  "impact_analysis": {
    "affected_functions": [
      {
        "id": "func-789",
        "name": "uploadFile",
        "relationship": "calls",
        "risk": "high"
      }
    ],
    "affected_services": [],
    "affected_dependencies": [],
    "call_chains": [
      {
        "path": ["func-123", "func-789", "func-101"],
        "depth": 2
      }
    ],
    "risk_level": "medium",
    "risk_factors": [
      "3 functions directly call this function",
      "Used in production API endpoint"
    ],
    "recommendations": [
      "Update all call sites before renaming",
      "Add deprecation warning first",
      "Consider creating wrapper function"
    ],
    "safe_refactor_steps": [
      "1. Create new function with new name",
      "2. Update call sites one by one",
      "3. Remove old function after migration"
    ]
  }
}
```

### GraphQL Queries

```graphql
# Function discovery query
query FindFunctions($repoId: String!, $query: String!) {
  codeElementSearch(
    repositoryId: $repoId
    query: $query
    elementType: "function"
  ) {
    id
    name
    signature
    filePath
    language
    relationships {
      targetType
      targetName
      relationshipType
    }
  }
}

# Refactoring impact query
query RefactoringImpact($repoId: String!, $targetIds: [String!]!) {
  refactoringImpact(
    repositoryId: $repoId
    targetElementIds: $targetIds
  ) {
    affectedElements {
      id
      name
      elementType
      filePath
    }
    affectedServices {
      name
      provider
    }
    riskLevel
    recommendations
  }

  functionDependencies(
    repositoryId: $repoId
    functionId: $targetIds[0]
  ) {
    function {
      name
      signature
    }
    calls {
      name
      filePath
    }
    calledBy {
      name
      filePath
    }
    usesServices {
      name
      provider
    }
  }
}
```

## Prompt Engineering

### Context Building Strategy

The AI assistant will build rich context by:

1. **Querying Relevant Entities**
   - Code elements matching the query
   - Related services and dependencies
   - Call chains and relationships
   - Security entities

2. **Structuring Context**
   ```python
   context = {
     "repository": {
       "name": "wavelength-hub",
       "language": "TypeScript"
     },
     "code_elements": [
       {
         "name": "getAdminStorage",
         "type": "function",
         "signature": "async function getAdminStorage(): Promise<Storage>",
         "file_path": "src/utils/storage.ts",
         "relationships": [
           {
             "type": "uses_service",
             "target": "Firebase Storage",
             "confidence": 0.85
           }
         ]
       }
     ],
     "graph_context": {
       "nodes": [...],
       "edges": [...]
     }
   }
   ```

3. **Prompt Templates**

**Function Discovery Prompt**:
```
You are an AI assistant helping developers understand their codebase.

Repository: {repository_name}
Language: {language}

The following functions are available:

{formatted_code_elements}

User Query: {user_query}

Based on the code structure and relationships shown above, answer the user's question.
Include:
- Specific function names and signatures
- File locations
- How functions relate to services/dependencies
- Usage examples if available

Be concise but thorough.
```

**Refactoring Analysis Prompt**:
```
You are an AI assistant analyzing refactoring impact.

Repository: {repository_name}
Target Elements: {target_elements}

Proposed Change: {proposed_changes}

Impact Analysis:
- Functions that call these elements: {callers}
- Functions called by these elements: {callees}
- Services used: {services}
- Dependencies: {dependencies}
- Call chains: {call_chains}

Analyze the refactoring impact and provide:
1. Risk assessment (low/medium/high)
2. List of affected code elements
3. Potential breaking changes
4. Step-by-step refactoring recommendations
5. Testing considerations

Be specific about what could break and why.
```

## Example Use Cases

### Use Case 1: Discovering Available Functions

**User Query**: "What authentication functions are available?"

**Process**:
1. Query parser identifies intent: "find functions" + topic: "authentication"
2. Context builder queries:
   - Code elements with "auth" in name/context
   - Related services (Clerk, Firebase Auth, etc.)
   - Functions that use auth services
3. AI receives structured context and generates response

**Response**:
```
Based on the codebase analysis, here are the authentication functions:

1. **authenticateUser(email, password)** - `src/auth/user.ts:45`
   - Uses Firebase Auth service
   - Returns User object
   - Called by: loginHandler, registerHandler

2. **verifyToken(token)** - `src/auth/token.ts:12`
   - Uses Clerk service
   - Returns boolean
   - Called by: middleware, API routes

3. **refreshSession(sessionId)** - `src/auth/session.ts:78`
   - Uses Firebase Auth
   - Returns Session object
   - Called by: sessionManager

Related services: Firebase Auth, Clerk
Related dependencies: firebase@9.0.0, @clerk/clerk-sdk-node
```

### Use Case 2: Refactoring Impact Analysis

**User Query**: "What would break if I rename `getAdminStorage` to `getFirebaseStorage`?"

**Process**:
1. Query parser identifies intent: "refactoring impact"
2. Context builder:
   - Finds function `getAdminStorage`
   - Traverses graph to find all callers
   - Identifies related services/dependencies
   - Builds call chain
3. AI analyzes impact and generates recommendations

**Response**:
```
⚠️ Refactoring Impact Analysis

**Risk Level**: Medium

**Affected Functions** (3 found):
1. `uploadFile` - `src/api/files.ts:123` - Directly calls getAdminStorage
2. `deleteFile` - `src/api/files.ts:156` - Directly calls getAdminStorage
3. `listFiles` - `src/utils/storage.ts:89` - Directly calls getAdminStorage

**Call Chain**:
getAdminStorage → uploadFile → handleFileUpload → API endpoint

**Recommendations**:
1. ✅ Create new function `getFirebaseStorage` first
2. ✅ Update call sites one by one (start with uploadFile)
3. ✅ Add deprecation warning to old function
4. ✅ Run tests after each change
5. ✅ Remove old function after all call sites updated

**Testing Considerations**:
- Test file upload functionality
- Test file deletion
- Test file listing
- Verify Firebase connection still works

**Safe Refactoring Steps**:
1. Create wrapper: `const getFirebaseStorage = getAdminStorage;`
2. Update imports/calls gradually
3. Remove wrapper after migration complete
```

### Use Case 3: Tool Discovery

**User Query**: "What build tools are configured?"

**Process**:
1. Query parser identifies intent: "list tools" + type: "build"
2. Context builder queries tools endpoint
3. AI formats response with usage information

**Response**:
```
Build Tools Configured:

1. **Vite** (v4.0.0)
   - Configuration: `vite.config.ts`
   - Scripts: `npm run build`, `npm run dev`
   - Used for: Frontend bundling, dev server

2. **esbuild** (v0.17.0)
   - Used by: Vite (transitive dependency)
   - Purpose: Fast JavaScript bundling

3. **TypeScript** (v5.0.0)
   - Configuration: `tsconfig.json`
   - Purpose: Type checking and compilation

Related npm scripts:
- `build`: Runs Vite build
- `dev`: Starts Vite dev server
- `type-check`: Runs TypeScript compiler
```

## Integration with Existing Architecture Decoder

### New Rust Modules

**`src/api/ai.rs`** - AI query endpoints
```rust
pub async fn ai_query(
    state: web::Data<ApiState>,
    body: web::Json<AiQueryRequest>,
) -> impl Responder {
    // Parse query intent
    // Query relevant entities
    // Build context
    // Return structured response
}

pub async fn refactor_analysis(
    state: web::Data<ApiState>,
    body: web::Json<RefactorAnalysisRequest>,
) -> impl Responder {
    // Find affected elements via graph traversal
    // Analyze call chains
    // Assess risk
    // Generate recommendations
}
```

**`src/analysis/refactoring.rs`** - Refactoring impact analysis
```rust
pub struct RefactoringAnalyzer {
    graph_builder: Arc<GraphBuilder>,
    code_repo: CodeElementRepository,
    relationship_repo: CodeRelationshipRepository,
}

impl RefactoringAnalyzer {
    pub fn analyze_impact(
        &self,
        repository_id: &str,
        target_elements: &[String],
    ) -> Result<RefactoringImpact> {
        // Traverse graph to find affected elements
        // Build call chains
        // Assess risk
    }
}
```

### External AI Assistant Service

**`ai-assistant/src/main.py`** - FastAPI service
```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import httpx
import openai

app = FastAPI()

class QueryRequest(BaseModel):
    repository_id: str
    query: str
    max_results: int = 10

@app.post("/query")
async def query_ai(request: QueryRequest):
    # Query Architecture Decoder API
    # Build context
    # Call OpenAI/Anthropic
    # Return formatted response
```

## Security Considerations

1. **API Authentication**: Add API key validation for AI endpoints
2. **Rate Limiting**: Prevent abuse of AI endpoints
3. **Context Sanitization**: Ensure no sensitive data in prompts
4. **Cost Management**: Monitor AI API usage and costs
5. **Caching**: Cache common queries to reduce API calls

## Future Enhancements

1. **IDE Integration**: VS Code extension for inline AI assistance
2. **Chat Interface**: Conversational AI that remembers context
3. **Code Generation**: Generate code based on architecture patterns
4. **Automated Refactoring**: Suggest and apply safe refactorings
5. **Learning System**: Improve recommendations based on user feedback

## Implementation Priority

1. **Phase 1** (Week 1-2): Basic query interface
2. **Phase 2** (Week 3-4): Function discovery
3. **Phase 3** (Week 5-6): Refactoring analysis
4. **Phase 4** (Week 7+): Advanced features

## Conclusion

This AI integration will transform the Architecture Decoder from a visualization tool into an intelligent development assistant, helping developers understand their codebase and make safer refactoring decisions.

