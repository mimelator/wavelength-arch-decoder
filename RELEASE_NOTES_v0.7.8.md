# 🎉 Release v0.7.8 - Plugin System & Enhanced Knowledge Graph

**Major release** introducing the powerful Plugin System and significant UI/UX improvements!

## 🌟 What's New

This release introduces a **comprehensive plugin architecture** that enables domain-specific asset detection without modifying the core decoder. Alongside this, we've made significant improvements to the Knowledge Graph visualization and fixed several UI issues.

## ✨ Key Features

### 🔌 Plugin System

**Zero Coupling Architecture**: Plugins are separate repositories that adapt their output to the decoder's generic data model. The decoder has **zero knowledge** of plugin internals.

**Key Capabilities**:
- **Generic Plugin Discovery**: Automatically finds plugins in standard locations
- **Language Flexibility**: Plugins can be written in any language (Python, Node.js, Rust, etc.)
- **JSON Communication**: Language-agnostic communication via JSON
- **Automatic Integration**: Plugins are automatically called during repository analysis
- **Plugin Listing**: New `/api/v1/plugins` endpoint to see loaded plugins
- **Version Integration**: Plugin information included in version endpoint

**Plugin Types**:
1. **Service Pattern Plugins** (JSON config files) - Add service detection patterns
2. **Asset Detection Plugins** (Executable scripts) - Detect domain-specific assets and relationships

**Benefits**:
- ✅ **Separation of Concerns**: Generic decoder vs. domain-specific plugins
- ✅ **Maintainability**: Core decoder stays clean and generic
- ✅ **Extensibility**: Add new domains without modifying decoder
- ✅ **Independent Development**: Plugins can be separate repos/projects
- ✅ **Zero Coupling**: Decoder has no knowledge of plugin internals

**Available Plugins**:
- **[webMethods Asset Detection Plugin](https://github.com/mimelator/wavelength-arch-decoder-webm-asset-plugin)** - Detects IS packages, MWS assets, CAF configurations, and relationships

### 🕸️ Knowledge Graph Enhancements

**Expandable Parent Nodes**:
- Code elements are now grouped by type (Functions, Classes, Modules, Methods)
- Parent nodes show counts and can be expanded/collapsed
- Cleaner initial view with code elements collapsed by default
- Visual indicators (▶/▼) for expansion state

**Node Details Popup**:
- Click any node to see detailed information
- Includes entity properties, file paths, and relationships
- Direct navigation links to repository detail tabs
- Improved property formatting and display

**Enhanced Filtering**:
- Better node type toggles
- Improved edge filtering for collapsed/expanded nodes
- More intuitive graph controls

### 🐛 Bug Fixes

- **Tools Tab**: Fixed loading issue that prevented tools from displaying
- **Dependencies Tab**: Fixed rendering issues with dependency lists
- **Knowledge Graph**: Fixed node click functionality and details display
- **Code Elements**: Fixed missing Module-type elements in graph visualization

### 🔧 Improvements

- **Generic Dependency Extraction**: Plugin-based dependency extraction for domain-specific package managers
- **Code Relationships**: Enhanced relationship detection for plugin-provided assets
- **UI/UX**: Improved repository overview tab layout and error handling
- **Error Messages**: Better user feedback and diagnostic information

## 📚 Documentation

- **[Plugin System Guide](https://github.com/mimelator/wavelength-arch-decoder/blob/main/README.md#plugin-system)** - Comprehensive plugin architecture documentation
- **[Plugin Developer Guide](https://github.com/mimelator/wavelength-arch-decoder-webm-asset-plugin/blob/main/PLUGIN_DEVELOPER_GUIDE.md)** - Create your own plugins
- **[API Reference](https://github.com/mimelator/wavelength-arch-decoder/blob/main/README.md#api-reference)** - Complete API documentation

## 🔗 Links

- **Repository**: https://github.com/mimelator/wavelength-arch-decoder
- **Issues**: https://github.com/mimelator/wavelength-arch-decoder/issues
- **Releases**: https://github.com/mimelator/wavelength-arch-decoder/releases

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/mimelator/wavelength-arch-decoder.git
cd wavelength-arch-decoder

# Build and run
cargo build --release
cargo run --release

# Access the UI
open http://localhost:8080
```

## 🎯 What This Enables

✅ **Domain-Specific Detection**: Add plugins for webMethods, Salesforce, SAP, or any custom platform  
✅ **Clean Architecture**: Core decoder remains generic and maintainable  
✅ **Extensibility**: Extend functionality without modifying core code  
✅ **Independent Development**: Plugins can be developed and released separately  
✅ **Better Visualization**: Enhanced Knowledge Graph with expandable nodes and better navigation

## 🛠️ Technical Details

- **Rust Version**: 1.70+
- **License**: MIT
- **Database**: SQLite (embedded)
- **API**: REST & GraphQL

## 🙏 Acknowledgments

This release represents a significant architectural milestone, demonstrating how a generic tool can be extended with domain-specific capabilities through a clean plugin interface.

---

**Ready to extend the decoder?** [Create a Plugin →](https://github.com/mimelator/wavelength-arch-decoder-webm-asset-plugin/blob/main/PLUGIN_DEVELOPER_GUIDE.md)

