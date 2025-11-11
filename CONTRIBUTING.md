# Contributing to Wavelength Architecture Decoder

Thank you for your interest in contributing to Wavelength Architecture Decoder! This document provides guidelines for contributing to the project.

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Respect different viewpoints and experiences

## Getting Started

### Prerequisites

- Rust 1.70+ (or Go 1.21+)
- Git
- SQLite3
- Basic understanding of graph databases

### Development Setup

1. **Fork the repository**
   ```bash
   git clone https://github.com/your-username/wavelength-arch-decoder.git
   cd wavelength-arch-decoder
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   # Edit .env with your local configuration
   ```

3. **Build the project**
   ```bash
   cargo build  # or: go build
   ```

4. **Run tests**
   ```bash
   cargo test  # or: go test ./...
   ```

## Development Workflow

### Branching Strategy

- `main`: Stable, production-ready code
- `develop`: Integration branch for features
- `feature/*`: New features
- `fix/*`: Bug fixes
- `docs/*`: Documentation updates

### Making Changes

1. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write clear, readable code
   - Add tests for new functionality
   - Update documentation as needed

3. **Commit your changes**
   ```bash
   git commit -m "feat: add support for Python package dependencies"
   ```

4. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

Examples:
```
feat: add support for npm package.json parsing
fix: resolve memory leak in graph traversal
docs: update API key best practices
refactor: simplify dependency resolution logic
```

## Code Standards

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Write documentation comments for public APIs

### Go Guidelines

- Follow [Effective Go](https://go.dev/doc/effective_go)
- Use `gofmt` for formatting
- Use `golint` and `go vet` for linting
- Write godoc comments for exported functions

### General Guidelines

- **Self-contained**: No external service dependencies
- **Performance**: Optimize for large repositories
- **Security**: Never log or expose API keys
- **Testing**: Aim for >80% test coverage
- **Documentation**: Document public APIs and complex logic

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_package_parser

# With output
cargo test -- --nocapture
```

### Writing Tests

- Unit tests for individual functions
- Integration tests for API endpoints
- Test fixtures for sample repositories
- Mock external dependencies

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_json() {
        let content = r#"{"dependencies": {"express": "^4.18.0"}}"#;
        let deps = parse_package_json(content).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "express");
    }
}
```

## Project Structure

```
wavelength-arch-decoder/
├── src/
│   ├── api/          # API handlers
│   ├── ingestion/    # Repository crawler
│   ├── analysis/     # Analysis engine
│   ├── graph/        # Knowledge graph
│   ├── parsers/      # File parsers
│   └── security/     # Security analysis
├── tests/
│   ├── unit/         # Unit tests
│   ├── integration/  # Integration tests
│   └── fixtures/     # Test data
├── docs/             # Documentation
└── examples/         # Example usage
```

## Areas for Contribution

### High Priority

- [ ] Package dependency parsers (npm, pip, cargo, maven, etc.)
- [ ] External service detection patterns
- [ ] Security relationship analysis
- [ ] Graph visualization improvements
- [ ] Performance optimizations

### Medium Priority

- [ ] Additional language support
- [ ] CI/CD pipeline analysis
- [ ] Infrastructure as Code parsing
- [ ] Documentation improvements
- [ ] Test coverage improvements

### Nice to Have

- [ ] CLI tool
- [ ] IDE plugins
- [ ] Webhook support
- [ ] Batch processing
- [ ] Export/import functionality

## Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new functionality
3. **Ensure all tests pass**
4. **Update CHANGELOG.md** with your changes
5. **Request review** from maintainers
6. **Address feedback** promptly
7. **Squash commits** before merging (if requested)

### PR Checklist

- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No breaking changes (or documented)
- [ ] Self-contained (no external dependencies)

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

