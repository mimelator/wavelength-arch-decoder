#!/bin/bash

# Wrapper script that auto-detects Node.js or Python and runs the appropriate validation script

REPO_ID="${1:-f424b3dc-f3c0-440f-89f7-bf1219d693ec}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔍 Repository Tab Validation Script"
echo "Repository ID: $REPO_ID"
echo ""

# Check for Node.js and Puppeteer
if command -v node &> /dev/null; then
    if [ -f "$SCRIPT_DIR/node_modules/puppeteer/package.json" ] || [ -f "$SCRIPT_DIR/../node_modules/puppeteer/package.json" ]; then
        echo "✓ Using Node.js (Puppeteer)"
        cd "$SCRIPT_DIR"
        if [ ! -d "node_modules" ]; then
            echo "Installing dependencies..."
            npm install
        fi
        node validate-tabs.js "$REPO_ID"
        exit $?
    elif [ -f "$SCRIPT_DIR/package.json" ]; then
        echo "✓ Using Node.js (Puppeteer) - installing dependencies..."
        cd "$SCRIPT_DIR"
        npm install
        node validate-tabs.js "$REPO_ID"
        exit $?
    fi
fi

# Check for Python and Selenium
if command -v python3 &> /dev/null; then
    if python3 -c "import selenium" 2>/dev/null; then
        echo "✓ Using Python3 (Selenium)"
        python3 "$SCRIPT_DIR/validate-tabs.py" "$REPO_ID"
        exit $?
    else
        echo "⚠ Python3 found but selenium not installed"
        echo "Install with: pip install selenium webdriver-manager"
    fi
fi

# If we get here, neither is available
echo "❌ Error: Neither Node.js (with Puppeteer) nor Python3 (with Selenium) is available"
echo ""
echo "Please install one of the following:"
echo ""
echo "Option 1 - Node.js:"
echo "  cd scripts && npm install"
echo ""
echo "Option 2 - Python3:"
echo "  pip install selenium webdriver-manager"
echo ""
exit 1

