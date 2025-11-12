#!/bin/bash

# Version bumping script for Wavelength Architecture Decoder
# Usage: ./bump_version.sh [patch|minor|major]

set -e

VERSION_FILE="VERSION"
CARGO_TOML="Cargo.toml"
README_MD="README.md"
INDEX_HTML="static/index.html"
VERSION_CHECK_RS="src/api/version_check.rs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}ℹ${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to get current version
get_current_version() {
    if [ -f "$VERSION_FILE" ]; then
        cat "$VERSION_FILE" | tr -d '\n' | tr -d ' '
    else
        print_error "VERSION file not found!"
        exit 1
    fi
}

# Function to bump version
bump_version() {
    local current_version=$1
    local bump_type=$2
    
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major=${VERSION_PARTS[0]}
    local minor=${VERSION_PARTS[1]}
    local patch=${VERSION_PARTS[2]}
    
    case $bump_type in
        patch)
            patch=$((patch + 1))
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        *)
            print_error "Invalid bump type: $bump_type"
            print_info "Usage: $0 [patch|minor|major]"
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Function to update VERSION file
update_version_file() {
    local new_version=$1
    echo "$new_version" > "$VERSION_FILE"
    print_info "Updated $VERSION_FILE: $new_version"
}

# Function to update Cargo.toml
update_cargo_toml() {
    local new_version=$1
    if [ -f "$CARGO_TOML" ]; then
        # Use sed to update version line
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
        else
            # Linux
            sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
        fi
        print_info "Updated $CARGO_TOML: $new_version"
    else
        print_warn "$CARGO_TOML not found, skipping"
    fi
}

# Function to update README.md badge
update_readme() {
    local new_version=$1
    if [ -f "$README_MD" ]; then
        # Update version badge
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/version-0\.[0-9]\+\.[0-9]\+/version-$new_version/g" "$README_MD"
        else
            sed -i "s/version-0\.[0-9]\+\.[0-9]\+/version-$new_version/g" "$README_MD"
        fi
        print_info "Updated README.md badge: $new_version"
    else
        print_warn "$README_MD not found, skipping"
    fi
}

# Function to update index.html footer
update_index_html() {
    local new_version=$1
    if [ -f "$INDEX_HTML" ]; then
        # Update footer version
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/v0\.[0-9]\+\.[0-9]\+/v$new_version/g" "$INDEX_HTML"
        else
            sed -i "s/v0\.[0-9]\+\.[0-9]\+/v$new_version/g" "$INDEX_HTML"
        fi
        print_info "Updated $INDEX_HTML footer: $new_version"
    else
        print_warn "$INDEX_HTML not found, skipping"
    fi
}

# Function to update version_check.rs fallback
update_version_check_rs() {
    local new_version=$1
    if [ -f "$VERSION_CHECK_RS" ]; then
        # Update fallback version in get_current_version function
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"0\.[0-9]\+\.[0-9]\+\"/\"$new_version\"/g" "$VERSION_CHECK_RS"
        else
            sed -i "s/\"0\.[0-9]\+\.[0-9]\+\"/\"$new_version\"/g" "$VERSION_CHECK_RS"
        fi
        print_info "Updated $VERSION_CHECK_RS fallback: $new_version"
    else
        print_warn "$VERSION_CHECK_RS not found, skipping"
    fi
}

# Function to update src/api/mod.rs fallback (if exists)
update_mod_rs() {
    local new_version=$1
    local mod_rs="src/api/mod.rs"
    if [ -f "$mod_rs" ]; then
        # Check if there's a fallback version string
        if grep -q "\"0\.[0-9]\+\.[0-9]\+\"" "$mod_rs"; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/\"0\.[0-9]\+\.[0-9]\+\"/\"$new_version\"/g" "$mod_rs"
            else
                sed -i "s/\"0\.[0-9]\+\.[0-9]\+\"/\"$new_version\"/g" "$mod_rs"
            fi
            print_info "Updated $mod_rs fallback: $new_version"
        fi
    fi
}

# Main script
main() {
    if [ $# -eq 0 ]; then
        print_error "No bump type specified"
        print_info "Usage: $0 [patch|minor|major]"
        print_info ""
        print_info "Examples:"
        print_info "  $0 patch   # 0.7.3 -> 0.7.4"
        print_info "  $0 minor   # 0.7.3 -> 0.8.0"
        print_info "  $0 major   # 0.7.3 -> 1.0.0"
        exit 1
    fi
    
    local bump_type=$1
    local current_version=$(get_current_version)
    local new_version=$(bump_version "$current_version" "$bump_type")
    
    print_info "Current version: $current_version"
    print_info "Bump type: $bump_type"
    print_info "New version: $new_version"
    echo ""
    
    # Confirm before proceeding
    read -p "Continue with version bump? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warn "Version bump cancelled"
        exit 0
    fi
    
    echo ""
    print_info "Updating version files..."
    
    # Update all version references
    update_version_file "$new_version"
    update_cargo_toml "$new_version"
    update_readme "$new_version"
    update_index_html "$new_version"
    update_version_check_rs "$new_version"
    update_mod_rs "$new_version"
    
    echo ""
    print_info "✓ Version bumped successfully from $current_version to $new_version"
    echo ""
    print_info "Files updated:"
    echo "  - $VERSION_FILE"
    echo "  - $CARGO_TOML"
    echo "  - $README_MD"
    echo "  - $INDEX_HTML"
    echo "  - $VERSION_CHECK_RS"
    echo ""
    print_info "Next steps:"
    echo "  1. Review the changes: git diff"
    echo "  2. Commit: git add -A && git commit -m \"Bump version to $new_version\""
    echo "  3. Tag: git tag -a v$new_version -m \"Version $new_version\""
}

main "$@"

