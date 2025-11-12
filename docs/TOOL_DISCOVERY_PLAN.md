# Tool Discovery Plan

## Overview

Extend the Wavelength Architecture Decoder to discover and catalog developer tools, scripts, and development environments within repositories. This complements the existing service detection system by identifying the tools developers use to build, test, and maintain their codebase.

## Goals

1. **Discover Developer Tools**: Identify CLI tools, build tools, testing frameworks, linters, formatters, etc.
2. **Catalog Scripts**: Extract and document npm scripts, shell scripts, makefiles, and other automation
3. **Map Tool Relationships**: Understand how tools relate to dependencies, services, and code
4. **Document Tool Usage**: Provide insights into development workflows and practices

## Tool Categories

### 1. Build Tools
- **Examples**: webpack, vite, rollup, esbuild, tsc, cargo, go build, maven, gradle
- **Detection**: package.json scripts, config files, CLI invocations

### 2. Testing Frameworks
- **Examples**: jest, mocha, pytest, unittest, cargo test, go test
- **Detection**: test scripts, test file patterns, config files

### 3. Linters & Formatters
- **Examples**: eslint, prettier, pylint, black, rustfmt, gofmt
- **Detection**: config files (.eslintrc, .prettierrc, pyproject.toml), scripts

### 4. Development Servers
- **Examples**: webpack-dev-server, vite dev, nodemon, hot-reload tools
- **Detection**: dev scripts, config files

### 5. Code Generators
- **Examples**: create-react-app, yeoman, cookiecutter, cargo-generate
- **Detection**: CLI invocations, generated code patterns

### 6. Debugging Tools
- **Examples**: debugger, pdb, gdb, lldb, chrome devtools
- **Detection**: debug scripts, config files

### 7. Shell Scripts
- **Examples**: .sh, .bash, .zsh files, Makefile, Justfile
- **Detection**: File patterns, shebang detection

### 8. Task Runners
- **Examples**: npm scripts, make, just, task, gulp, grunt
- **Detection**: package.json scripts, task runner configs

### 9. Development Environments
- **Examples**: .venv, virtualenv, conda, docker-compose.dev.yml, devcontainer
- **Detection**: Directory patterns, config files

### 10. SDKs & Dev Kits
- **Examples**: AWS CDK, Terraform, Serverless Framework, Firebase CLI
- **Detection**: Package dependencies, config files, CLI usage

## Detection Methods

### Method 1: Package Scripts Analysis
**Source**: `package.json`, `Cargo.toml`, `pyproject.toml`, etc.

```json
{
  "scripts": {
    "build": "webpack --mode production",
    "test": "jest",
    "lint": "eslint .",
    "format": "prettier --write ."
  }
}
```

**Detection**:
- Parse scripts section
- Extract tool names from commands
- Identify tool categories from command patterns

### Method 2: Config File Detection
**Patterns**:
- `.eslintrc.*`, `.prettierrc.*`, `.editorconfig`
- `jest.config.*`, `pytest.ini`, `Cargo.toml`
- `webpack.config.*`, `vite.config.*`
- `Makefile`, `justfile`, `Taskfile.yml`

**Detection**:
- File name patterns
- Config file content analysis
- Tool-specific config formats

### Method 3: Executable File Detection
**Patterns**:
- Shell scripts: `*.sh`, `*.bash`, `*.zsh`
- Python scripts: `*.py` with shebang
- Binary executables: `bin/`, `scripts/`, `tools/`

**Detection**:
- File extensions
- Shebang analysis (`#!/usr/bin/env node`, `#!/bin/bash`)
- Executable permissions
- Directory patterns

### Method 4: Dependency Analysis
**Source**: Existing dependency extraction

**Detection**:
- Dev dependencies that are tools
- Tool-specific package patterns (`*-cli`, `*-tool`, `*-dev`)
- Known tool packages (eslint, prettier, jest, etc.)

### Method 5: Code Pattern Analysis
**Source**: Code structure analysis

**Detection**:
- CLI invocations in code
- Tool-specific imports/requires
- Configuration object patterns

### Method 6: Environment Detection
**Patterns**:
- `.venv/`, `venv/`, `env/` directories
- `Dockerfile.dev`, `docker-compose.dev.yml`
- `.devcontainer/`, `devcontainer.json`
- `.nvmrc`, `.node-version`, `.python-version`

**Detection**:
- Directory patterns
- Config file analysis
- Version file patterns

## Data Model

### Tool Entity

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTool {
    pub id: String,
    pub name: String,
    pub tool_type: ToolType,
    pub category: ToolCategory,
    pub version: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub detection_method: DetectionMethod,
    pub configuration: HashMap<String, String>,
    pub scripts: Vec<ToolScript>,
    pub relationships: Vec<ToolRelationship>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolType {
    BuildTool,
    TestFramework,
    Linter,
    Formatter,
    DevServer,
    CodeGenerator,
    Debugger,
    TaskRunner,
    ShellScript,
    DevEnvironment,
    Sdk,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    // Build Tools
    Webpack,
    Vite,
    Rollup,
    Esbuild,
    Tsc,
    Cargo,
    GoBuild,
    Maven,
    Gradle,
    
    // Testing
    Jest,
    Mocha,
    Pytest,
    Unittest,
    CargoTest,
    GoTest,
    
    // Linting
    Eslint,
    Pylint,
    Rustfmt,
    Gofmt,
    
    // Formatting
    Prettier,
    Black,
    
    // Dev Servers
    WebpackDevServer,
    ViteDev,
    Nodemon,
    
    // Task Runners
    NpmScripts,
    Make,
    Just,
    Task,
    
    // Shell Scripts
    Bash,
    Zsh,
    Fish,
    
    // Dev Environments
    Venv,
    Conda,
    DockerDev,
    DevContainer,
    
    // SDKs
    AwsCdk,
    Terraform,
    Serverless,
    FirebaseCli,
    
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolScript {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRelationship {
    pub target_type: RelationshipTargetType,
    pub target_id: String,
    pub relationship_type: String, // "uses", "depends_on", "generates", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipTargetType {
    Dependency,
    Service,
    CodeElement,
    OtherTool,
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1)

1. **Create Tool Detection Module**
   - `src/analysis/tool_detector.rs`
   - Basic tool detection structure
   - Integration with existing analysis pipeline

2. **Database Schema**
   - `tools` table
   - `tool_scripts` table
   - `tool_relationships` table
   - Migration scripts

3. **Configurable Patterns**
   - Extend pattern config system
   - `config/tool_patterns.json`
   - Tool detection patterns

### Phase 2: Core Detection (Week 2)

1. **Package Scripts Analysis**
   - Parse npm scripts
   - Extract tool names from commands
   - Categorize tools

2. **Config File Detection**
   - Common config file patterns
   - Tool-specific config parsing
   - Config-to-tool mapping

3. **Dependency-Based Detection**
   - Analyze dev dependencies
   - Tool package patterns
   - Known tool registry

### Phase 3: Advanced Detection (Week 3)

1. **Script File Analysis**
   - Shell script detection
   - Shebang analysis
   - Executable detection

2. **Code Pattern Analysis**
   - CLI invocation patterns
   - Tool imports/requires
   - Configuration objects

3. **Environment Detection**
   - Virtual environment detection
   - Dev container detection
   - Version file detection

### Phase 4: Relationships & Integration (Week 4)

1. **Tool Relationships**
   - Link tools to dependencies
   - Link tools to services
   - Link tools to code elements
   - Tool-to-tool relationships

2. **Knowledge Graph Integration**
   - Add tool nodes
   - Create tool edges
   - Visualize tool relationships

3. **API & UI**
   - Tool API endpoints
   - Tool UI components
   - Tool detail views

## Configuration System

### Tool Patterns Config

```json
{
  "version": "1.0",
  "tool_patterns": {
    "package_scripts": [
      {
        "pattern": "webpack",
        "category": "Webpack",
        "tool_type": "BuildTool",
        "confidence": 0.9
      },
      {
        "pattern": "jest",
        "category": "Jest",
        "tool_type": "TestFramework",
        "confidence": 0.9
      }
    ],
    "config_files": [
      {
        "pattern": ".eslintrc",
        "category": "Eslint",
        "tool_type": "Linter",
        "confidence": 0.95
      },
      {
        "pattern": "webpack.config",
        "category": "Webpack",
        "tool_type": "BuildTool",
        "confidence": 0.9
      }
    ],
    "dev_dependencies": [
      {
        "pattern": "eslint",
        "category": "Eslint",
        "tool_type": "Linter",
        "confidence": 0.9
      },
      {
        "pattern": "prettier",
        "category": "Prettier",
        "tool_type": "Formatter",
        "confidence": 0.9
      }
    ],
    "executable_patterns": [
      {
        "pattern": "*.sh",
        "category": "Bash",
        "tool_type": "ShellScript",
        "confidence": 0.8
      },
      {
        "pattern": "Makefile",
        "category": "Make",
        "tool_type": "TaskRunner",
        "confidence": 0.9
      }
    ],
    "environment_patterns": [
      {
        "pattern": ".venv",
        "category": "Venv",
        "tool_type": "DevEnvironment",
        "confidence": 0.9
      },
      {
        "pattern": "devcontainer.json",
        "category": "DevContainer",
        "tool_type": "DevEnvironment",
        "confidence": 0.95
      }
    ]
  }
}
```

## Integration Points

### 1. With Dependency Analysis
- Tools often appear as dev dependencies
- Link tools to their package dependencies
- Show tool usage in dependency graph

### 2. With Service Detection
- Some tools interact with services (e.g., AWS CLI, Terraform)
- Link tools to services they configure/manage
- Show tool-service relationships

### 3. With Code Structure
- Tools are invoked from code
- Link tool invocations to code elements
- Show tool usage patterns

### 4. With Knowledge Graph
- Add tool nodes
- Create relationships:
  - Tool → Dependency
  - Tool → Service
  - Tool → Code Element
  - Tool → Tool (toolchains)

## Example Use Cases

### Use Case 1: Discover Development Workflow
**Input**: Repository path
**Output**: List of tools used, scripts available, development environment

**Example**:
```
Tools Detected:
- Build: webpack, tsc
- Test: jest
- Lint: eslint
- Format: prettier
- Dev Server: webpack-dev-server

Scripts Available:
- npm run build → webpack --mode production
- npm test → jest
- npm run lint → eslint .
- npm run format → prettier --write .

Dev Environment: Node.js 18.x (from .nvmrc)
```

### Use Case 2: Tool Relationship Mapping
**Input**: Tool name
**Output**: What dependencies it uses, what services it interacts with, what code uses it

**Example**:
```
Webpack Tool:
- Uses: webpack, webpack-cli (dependencies)
- Generates: dist/ (output)
- Used by: build script, deploy script
- Configures: AWS S3 (deployment target)
```

### Use Case 3: Missing Tool Detection
**Input**: Repository analysis
**Output**: Suggestions for missing tools based on project type

**Example**:
```
Project Type: JavaScript/TypeScript
Detected: webpack, jest
Missing (suggested):
- Linter: eslint (common for JS/TS projects)
- Formatter: prettier (common for JS/TS projects)
```

## Success Metrics

1. **Coverage**: Detect 80%+ of common tools in repositories
2. **Accuracy**: 90%+ confidence in tool identification
3. **Relationships**: Map 70%+ of tool relationships
4. **Performance**: Analyze repository in <30 seconds

## Future Enhancements

1. **Tool Version Detection**: Extract exact tool versions
2. **Tool Usage Analytics**: How often tools are used
3. **Tool Recommendations**: Suggest better tools
4. **Tool Compatibility**: Check tool compatibility
5. **Tool Documentation**: Link to tool docs
6. **Tool Migration**: Suggest tool migrations (e.g., webpack → vite)

## Technical Considerations

1. **Performance**: Tool detection should be fast
2. **Extensibility**: Use configurable patterns (like service detection)
3. **Accuracy**: Balance false positives vs false negatives
4. **Maintainability**: Keep tool patterns updated
5. **Integration**: Seamless integration with existing systems

## Dependencies

- Existing pattern config system
- Dependency extraction system
- Code structure analysis
- Knowledge graph builder
- Database storage layer

## Timeline

- **Week 1**: Foundation & database schema
- **Week 2**: Core detection (scripts, configs, dependencies)
- **Week 3**: Advanced detection (scripts, code patterns, environments)
- **Week 4**: Relationships, graph integration, API/UI

**Total**: ~4 weeks for full implementation

