# Release Checklist for v0.7.8

## ✅ Pre-Release Tasks Completed

- [x] Version badge updated in README (0.7.7 → 0.7.8)
- [x] Repository URL fixed in Cargo.toml (your-org → mimelator)
- [x] CHANGELOG.md updated with v0.7.8 features
- [x] Release notes created (RELEASE_NOTES_v0.7.8.md)
- [x] Short release notes created (RELEASE_NOTES_SHORT.md)

## 📋 Git Commands to Run

```bash
cd /Volumes/5bits/current/wavelength-dev/arch/wavelength-arch-decoder

# Stage all changes
git add -A

# Review changes
git status

# Commit
git commit -m "Prepare for v0.7.8 release

- Update version badge to 0.7.8
- Fix repository URL in Cargo.toml
- Update CHANGELOG with plugin system and enhancements
- Add release notes templates"

# Create tag
git tag -a v0.7.8 -m "Release v0.7.8 - Plugin System & Enhanced Knowledge Graph

Major release introducing:
- Comprehensive plugin architecture for domain-specific asset detection
- Enhanced Knowledge Graph with expandable parent nodes
- UI improvements and bug fixes
- Generic dependency extraction"

# Push
git push origin main
git push origin v0.7.8
```

## 🚀 GitHub Release Steps

1. **Navigate to**: https://github.com/mimelator/wavelength-arch-decoder/releases/new

2. **Fill in**:
   - **Tag**: Select `v0.7.8` from dropdown
   - **Release title**: `Release v0.7.8 - Plugin System & Enhanced Knowledge Graph`
   - **Description**: Copy contents from `RELEASE_NOTES_v0.7.8.md`
   - **Check**: "Set as the latest release"
   - **Click**: "Publish release"

## 📝 Release Notes Location

- **Full notes**: `RELEASE_NOTES_v0.7.8.md`
- **Short notes**: `RELEASE_NOTES_SHORT.md`

## 🔗 Related Release

**Plugin Release**: v1.0.0 of `wavelength-arch-decoder-webm-asset-plugin` should be released simultaneously to demonstrate the plugin system.

## ✨ Key Features in This Release

- **Plugin System**: Zero-coupling architecture for domain-specific plugins
- **Knowledge Graph**: Expandable parent nodes, node details popup
- **Bug Fixes**: Tools tab, dependencies tab, graph node clicks
- **Improvements**: Generic dependency extraction, enhanced relationships

