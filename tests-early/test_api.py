#!/usr/bin/env python3
"""
Test script for Wavelength Architecture Decoder API
"""
import requests
import json
import sys
import time

BASE_URL = "http://localhost:8080"

def test_health():
    """Test health check endpoint"""
    print("Testing health check...")
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        print(f"  Status: {response.status_code}")
        print(f"  Response: {json.dumps(response.json(), indent=2)}")
        return response.status_code == 200
    except requests.exceptions.ConnectionError:
        print("  ✗ Server is not running. Please start the server first.")
        return False
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return False

def test_register():
    """Test user registration"""
    print("\nTesting user registration...")
    data = {
        "email": "test@example.com",
        "password": "testpassword123"
    }
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/auth/register",
            json=data,
            timeout=5
        )
        print(f"  Status: {response.status_code}")
        result = response.json()
        print(f"  Response: {json.dumps(result, indent=2)}")
        if response.status_code == 201:
            api_key = result.get("api_key")
            print(f"\n  ✓ Registration successful!")
            print(f"  API Key: {api_key}")
            return api_key
        else:
            print(f"  ✗ Registration failed")
            return None
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return None

def test_login():
    """Test user login"""
    print("\nTesting user login...")
    data = {
        "email": "test@example.com",
        "password": "testpassword123"
    }
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/auth/login",
            json=data,
            timeout=5
        )
        print(f"  Status: {response.status_code}")
        result = response.json()
        print(f"  Response: {json.dumps(result, indent=2)}")
        if response.status_code == 200:
            api_key = result.get("api_key")
            print(f"\n  ✓ Login successful!")
            print(f"  API Key: {api_key}")
            return api_key
        else:
            print(f"  ✗ Login failed")
            return None
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return None

def test_create_api_key(api_key):
    """Test creating a new API key"""
    print("\nTesting API key creation...")
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }
    data = {
        "name": "test-key",
        "scopes": ["read", "write", "admin"],
        "expires_in_days": 30
    }
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/auth/keys",
            headers=headers,
            json=data,
            timeout=5
        )
        print(f"  Status: {response.status_code}")
        result = response.json()
        print(f"  Response: {json.dumps(result, indent=2)}")
        if response.status_code == 201:
            new_key = result.get("api_key")
            print(f"\n  ✓ API key creation successful!")
            print(f"  New API Key: {new_key}")
            return new_key
        else:
            print(f"  ✗ API key creation failed")
            return None
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return None

def main():
    print("=" * 60)
    print("Wavelength Architecture Decoder API Test")
    print("=" * 60)
    
    # Test health check
    if not test_health():
        print("\n✗ Health check failed. Make sure the server is running.")
        print("  Start the server with: cargo run")
        sys.exit(1)
    
    # Test registration
    api_key = test_register()
    
    # Test login
    login_key = test_login()
    
    # Test creating API key (requires admin scope)
    if login_key:
        # First, we need to update the default key to have admin scope
        # For now, let's just test with the login key
        print("\nNote: API key creation requires admin scope.")
        print("      The default key from registration/login has read/write scope.")
    
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    print("✓ Health check: PASSED")
    if api_key:
        print("✓ Registration: PASSED")
    if login_key:
        print("✓ Login: PASSED")
    print("\nAll basic tests completed!")

if __name__ == "__main__":
    main()

