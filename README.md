# Wavelength Architecture Decoder

A self-contained hosted server that enables Software Engineers to understand the complete architecture of repositories through an intelligent knowledge graph system.

## Project Overview

The Wavelength Architecture Decoder analyzes code repositories to build a comprehensive knowledge graph that maps:
- **External Service Dependencies**: Clerk, Vercel, AWS services, databases, APIs, etc.
- **Package Dependencies**: npm, pip, cargo, maven, etc. with version tracking
- **Security Relationships**: IAM roles, Lambda functions, S3 buckets, security groups, network policies
- **Code Structure**: Modules, functions, classes, and their relationships
- **Deployment Entities**: Infrastructure as Code (Terraform, CloudFormation), CI/CD pipelines, containers

### Core Purposes

1. **Human Exploration**: Interactive visualization and exploration of architectural relationships
2. **AI Assistant Integration**: Structured knowledge graph for AI tools to reason about dependencies and find solutions
3. **Multi-Source Updates**: Updatable by humans, AI assistance, or automated crawlers

## Architecture Principles

### Self-Contained Design
- **No External Dependencies**: All services run locally or in a single deployment
- **Embedded Database**: SQLite or embedded graph database (Neo4j embedded, Dgraph, or custom)
- **Built-in Crawler**: Repository ingestion without external services
- **Local Processing**: All analysis runs on the server without cloud dependencies

### Security & API Keys

**Best Practices for API Key Management:**

1. **Environment Variables**: All API keys stored in environment variables, never in code
2. **Encrypted Storage**: API keys encrypted at rest in the database
3. **Key Rotation**: Support for key rotation without service interruption
4. **Scoped Access**: API keys have scoped permissions (read-only, write, admin)
5. **Audit Logging**: All API key usage logged for security auditing
6. **Key Validation**: Server validates API keys before processing requests
7. **Rate Limiting**: Per-key rate limiting to prevent abuse
8. **Documentation**: Clear documentation on how to obtain and use API keys

**API Key Storage Format:**
```
API_KEY_<SERVICE>_<ENVIRONMENT>=encrypted_value
```

## System Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway Layer                         │
│  (Authentication, Rate Limiting, Request Routing)            │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│  Ingestion     │  │  Analysis      │  │  Query         │
│  Service       │  │  Engine        │  │  Service       │
└───────┬────────┘  └───────┬────────┘  └───────┬────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
┌───────────────────────────▼───────────────────────────────┐
│              Knowledge Graph Database                      │
│  (Graph Structure: Nodes, Edges, Properties, Metadata)    │
└────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐  ┌───────▼────────┐  ┌───────▼────────┐
│  Repository    │  │  File System   │  │  Cache Layer   │
│  Crawler       │  │  Index         │  │  (Optional)    │
└────────────────┘  └────────────────┘  └────────────────┘
```

### Core Services

#### 1. Ingestion Service
- **Repository Crawler**: Clones/accesses repositories (Git, SVN, etc.)
- **File Parser**: Parses various file types (code, config, IaC, CI/CD)
- **Dependency Extractor**: Identifies package dependencies from lock files
- **Service Detector**: Finds references to external services in code/config
- **Security Scanner**: Extracts IAM roles, policies, security configurations

#### 2. Analysis Engine
- **Relationship Mapper**: Builds connections between entities
- **Dependency Resolver**: Resolves transitive dependencies
- **Security Analyzer**: Identifies security relationships and vulnerabilities
- **Graph Builder**: Constructs and updates the knowledge graph

#### 3. Query Service
- **GraphQL API**: Flexible querying of the knowledge graph
- **REST API**: Standard REST endpoints for common queries
- **Visualization API**: Endpoints for graph visualization tools
- **Search Engine**: Full-text search across all entities

#### 4. Knowledge Graph Database
- **Node Types**: Repository, File, Function, Service, Dependency, SecurityEntity
- **Edge Types**: DEPENDS_ON, CALLS, USES, CONFIGURES, SECURES, DEPLOYS_TO
- **Properties**: Metadata, versions, locations, configurations

## Data Model

### Node Types

```
Repository
  - id: string
  - name: string
  - url: string
  - branch: string
  - last_analyzed: timestamp

File
  - id: string
  - path: string
  - type: enum (code, config, iac, cicd, readme)
  - language: string
  - content_hash: string

Function/Class/Module
  - id: string
  - name: string
  - type: enum (function, class, module, component)
  - language: string
  - file_id: reference
  - line_start: int
  - line_end: int

ExternalService
  - id: string
  - name: string (e.g., "AWS S3", "Clerk", "Vercel")
  - type: enum (saas, cloud, database, api)
  - configuration: json

PackageDependency
  - id: string
  - name: string
  - version: string
  - package_manager: enum (npm, pip, cargo, maven, etc.)
  - repository_id: reference

SecurityEntity
  - id: string
  - type: enum (iam_role, lambda, s3_bucket, security_group, policy)
  - name: string
  - configuration: json
  - permissions: json
```

### Edge Types

```
DEPENDS_ON: PackageDependency -> PackageDependency
IMPORTS: File -> File
CALLS: Function -> Function
USES_SERVICE: Code -> ExternalService
CONFIGURES: File -> SecurityEntity
SECURES: SecurityEntity -> SecurityEntity
DEPLOYS_TO: Repository -> ExternalService
REFERENCES: Any -> Any (generic relationship)
```

## Technology Stack

### Backend
- **Language**: Rust (performance, memory safety) or Go (simplicity, concurrency)
- **Web Framework**: Actix-web (Rust) or Gin/Echo (Go)
- **Graph Database**: 
  - Option A: Neo4j Embedded (Java-based, mature)
  - Option B: Dgraph (Go-based, distributed)
  - Option C: Custom graph layer on SQLite (lightweight, self-contained)
- **Parser Libraries**: Tree-sitter (multi-language parsing)

### Frontend (Optional)
- **Visualization**: D3.js, Cytoscape.js, or vis.js for graph visualization
- **Framework**: React or Vue.js for interactive UI
- **API Client**: GraphQL client (Apollo, Relay)

### Storage
- **Primary**: Embedded graph database
- **File Cache**: Local filesystem or embedded object store
- **Metadata**: SQLite for indexing and search

## API Design

### Authentication
```
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/refresh

Headers:
  Authorization: Bearer <api_key>
```

### Repository Management
```
POST   /api/v1/repositories          # Add repository for analysis
GET    /api/v1/repositories          # List repositories
GET    /api/v1/repositories/{id}     # Get repository details
POST   /api/v1/repositories/{id}/analyze  # Trigger analysis
DELETE /api/v1/repositories/{id}     # Remove repository
```

### Query API (GraphQL)
```graphql
query {
  repository(id: "repo-123") {
    name
    files {
      path
      functions {
        name
        calls {
          target {
            name
          }
        }
      }
    }
    dependencies {
      name
      version
      dependsOn {
        name
      }
    }
    externalServices {
      name
      type
    }
    securityEntities {
      type
      name
      permissions
    }
  }
}

query {
  findDependencies(service: "AWS S3") {
    repositories {
      name
    }
    functions {
      name
      file {
        path
      }
    }
  }
}
```

### REST Query Endpoints
```
GET /api/v1/repositories/{id}/dependencies
GET /api/v1/repositories/{id}/services
GET /api/v1/repositories/{id}/security
GET /api/v1/search?q={query}
GET /api/v1/graph/visualize?repo_id={id}&depth={n}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-4)
- [ ] Set up project structure and build system
- [ ] Implement embedded database layer
- [ ] Create basic API server with authentication
- [ ] Implement API key management system
- [ ] Set up repository cloning/access mechanism
- [ ] Basic file parsing (common file types)

### Phase 2: Dependency Analysis (Weeks 5-8)
- [ ] Package dependency extractors (npm, pip, cargo, etc.)
- [ ] Dependency graph construction
- [ ] Transitive dependency resolution
- [ ] Version conflict detection
- [ ] Dependency API endpoints

### Phase 3: Service Detection (Weeks 9-12)
- [ ] External service detection patterns
- [ ] Configuration file parsing (AWS, Vercel, etc.)
- [ ] Service relationship mapping
- [ ] Service usage tracking
- [ ] Service API endpoints

### Phase 4: Security Analysis (Weeks 13-16)
- [ ] IAM role and policy extraction
- [ ] Lambda function analysis
- [ ] S3 bucket and security group detection
- [ ] Security relationship mapping
- [ ] Security vulnerability detection
- [ ] Security API endpoints

### Phase 5: Code Structure Analysis (Weeks 17-20)
- [ ] Multi-language code parsing (Tree-sitter integration)
- [ ] Function/class/module extraction
- [ ] Call graph construction
- [ ] Import/export relationship mapping
- [ ] Code structure API endpoints

### Phase 6: Knowledge Graph & Query (Weeks 21-24)
- [ ] Unified knowledge graph construction
- [ ] GraphQL API implementation
- [ ] Graph traversal algorithms
- [ ] Relationship inference
- [ ] Graph visualization endpoints

### Phase 7: Crawler & Automation (Weeks 25-28)
- [ ] Automated repository crawler
- [ ] Scheduled analysis jobs
- [ ] Webhook support for repository updates
- [ ] Batch processing capabilities
- [ ] Progress tracking and notifications

### Phase 8: UI & Documentation (Weeks 29-32)
- [ ] Graph visualization UI
- [ ] Repository browser
- [ ] Search interface
- [ ] API documentation
- [ ] User guides and tutorials

## Security Considerations

### API Key Security
- Keys stored encrypted using AES-256
- Keys never logged or exposed in error messages
- Key rotation mechanism
- Per-key rate limiting
- Key expiration support

### Repository Access
- Support for private repositories via SSH keys or tokens
- Secure credential storage
- Access control per repository
- Audit logging for all access

### Data Privacy
- Option to analyze repositories without storing full content
- Content hashing instead of storing raw files
- Configurable data retention policies
- GDPR compliance considerations

## Configuration

### Environment Variables
```bash
# Server Configuration
PORT=8080
HOST=0.0.0.0
ENVIRONMENT=production

# Database
DATABASE_PATH=./data/wavelength.db
GRAPH_DB_PATH=./data/graph.db

# Security
API_KEY_ENCRYPTION_KEY=<32-byte-key>
JWT_SECRET=<secret>
RATE_LIMIT_PER_KEY=1000  # requests per hour

# Repository Access (if needed)
GITHUB_TOKEN=<optional>
GITLAB_TOKEN=<optional>
SSH_KEY_PATH=<optional>

# Storage
REPOSITORY_CACHE_PATH=./cache/repos
MAX_CACHE_SIZE=10GB
```

## Development Setup

### Prerequisites
- Rust 1.70+ (or Go 1.21+)
- Git
- SQLite3
- 4GB+ RAM recommended

### Local Development
```bash
# Clone repository
git clone <repo-url>
cd wavelength-arch-decoder

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Build
cargo build --release  # or: go build

# Run
cargo run  # or: ./wavelength-arch-decoder

# Run tests
cargo test  # or: go test ./...
```

## API Key Best Practices Documentation

See [API_KEYS.md](./docs/API_KEYS.md) for detailed documentation on:
- How to obtain API keys
- How to securely store API keys
- Key rotation procedures
- Scoped permissions
- Rate limiting
- Security best practices

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Contributing

This is a public project. Contributions welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Roadmap

- [ ] Multi-repository analysis
- [ ] Real-time updates via webhooks
- [ ] AI-powered relationship inference
- [ ] Export/import knowledge graphs
- [ ] Plugin system for custom analyzers
- [ ] Integration with popular IDEs
- [ ] CLI tool for local analysis
