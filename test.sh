#!/bin/bash
# Test script for Wavelength Architecture Decoder

set -e

echo "=== Wavelength Architecture Decoder Test ==="
echo ""

# Check if .env exists, if not create from example
if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env
    
    # Generate secrets
    echo "Generating secrets..."
    API_KEY_ENCRYPTION_KEY=$(openssl rand -hex 32)
    JWT_SECRET=$(openssl rand -base64 32)
    
    # Update .env with generated secrets
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s|<generate-32-byte-key>|$API_KEY_ENCRYPTION_KEY|g" .env
        sed -i '' "s|<generate-secret>|$JWT_SECRET|g" .env
    else
        # Linux
        sed -i "s|<generate-32-byte-key>|$API_KEY_ENCRYPTION_KEY|g" .env
        sed -i "s|<generate-secret>|$JWT_SECRET|g" .env
    fi
    
    echo "✓ .env file created with generated secrets"
else
    echo "✓ .env file already exists"
fi

# Build the project
echo ""
echo "Building project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
else
    echo "✗ Build failed"
    exit 1
fi

echo ""
echo "=== Server is ready to test ==="
echo ""
echo "To start the server, run:"
echo "  cargo run"
echo ""
echo "Or use the release binary:"
echo "  ./target/release/wavelength-arch-decoder"
echo ""
echo "The server will start on http://localhost:8080"
echo ""
echo "Test endpoints:"
echo "  GET  http://localhost:8080/health"
echo "  POST http://localhost:8080/api/v1/auth/register"
echo "  POST http://localhost:8080/api/v1/auth/login"
echo ""

