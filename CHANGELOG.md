# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.8] - 2025-01-XX

### Added
- **Plugin System**: Comprehensive plugin architecture for domain-specific asset detection
  - Generic plugin discovery and execution framework
  - Support for executable plugins (Python, Node.js, etc.)
  - Plugin output in decoder-compatible `CodeElement` and `CodeRelationship` format
  - Zero coupling - plugins adapt to decoder's data model
  - Plugin listing API endpoint (`/api/v1/plugins`)
  - Plugin information in version endpoint
- **Knowledge Graph Enhancements**:
  - Expandable parent nodes for code element types (Functions, Classes, Modules)
  - Node details popup with entity information and navigation links
  - Improved graph filtering and node type toggles
- **UI Improvements**:
  - Fixed tools tab loading issue
  - Enhanced dependencies tab rendering
  - Improved repository overview tab layout
  - Better error handling and user feedback
- **Generic Dependency Extraction**:
  - Plugin-based dependency extraction for domain-specific package managers
  - Support for webMethods IS package dependencies
- **Code Relationships**:
  - Enhanced relationship detection for plugin-provided assets
  - Support for code-to-code relationships

### Changed
- Refactored plugin integration to be fully generic (no domain-specific code)
- Improved error messages and logging
- Enhanced graph builder to include all Module-type code elements

### Fixed
- Tools tab not loading correctly
- Dependencies tab rendering issues
- Knowledge graph node click functionality
- Missing code element types in graph visualization

## [Unreleased]

### Added
- Initial project plan and architecture documentation
- API key management best practices documentation
- Contributing guidelines

[0.7.8]: https://github.com/mimelator/wavelength-arch-decoder/compare/v0.7.7...v0.7.8
[Unreleased]: https://github.com/mimelator/wavelength-arch-decoder/compare/v0.7.8...HEAD

