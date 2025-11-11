#!/usr/bin/env python3
"""
Test script for Repository and Dependency API endpoints
"""
import requests
import json
import sys
import time
import os

BASE_URL = "http://localhost:8080"

def get_api_key():
    """Get API key by registering or logging in"""
    # Try to register first
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/auth/register",
            json={"email": "test@example.com", "password": "testpassword123"},
            timeout=5
        )
        if response.status_code == 201:
            return response.json()["api_key"]
    except:
        pass
    
    # If registration fails, try login
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/auth/login",
            json={"email": "test@example.com", "password": "testpassword123"},
            timeout=5
        )
        if response.status_code == 200:
            return response.json()["api_key"]
    except:
        pass
    
    return None

def test_health():
    """Test health check"""
    print("=" * 60)
    print("Testing Health Check")
    print("=" * 60)
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        print(f"Status: {response.status_code}")
        print(f"Response: {json.dumps(response.json(), indent=2)}")
        return response.status_code == 200
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def test_create_repository(api_key):
    """Test creating a repository"""
    print("\n" + "=" * 60)
    print("Testing Create Repository")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    
    # Use a small public repository for testing
    data = {
        "name": "test-repo",
        "url": "https://github.com/octocat/Hello-World.git",
        "branch": "master"
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/repositories",
            headers=headers,
            json=data,
            timeout=10
        )
        print(f"Status: {response.status_code}")
        result = response.json()
        print(f"Response: {json.dumps(result, indent=2)}")
        
        if response.status_code == 201:
            print("✓ Repository created successfully")
            return result.get("id")
        else:
            print("✗ Failed to create repository")
            return None
    except Exception as e:
        print(f"✗ Error: {e}")
        return None

def test_list_repositories(api_key):
    """Test listing repositories"""
    print("\n" + "=" * 60)
    print("Testing List Repositories")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}"}
    
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories",
            headers=headers,
            timeout=5
        )
        print(f"Status: {response.status_code}")
        repos = response.json()
        print(f"Found {len(repos)} repositories")
        if repos:
            print(f"First repository: {json.dumps(repos[0], indent=2)}")
        return response.status_code == 200
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def test_get_repository(api_key, repo_id):
    """Test getting a repository"""
    print("\n" + "=" * 60)
    print("Testing Get Repository")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}"}
    
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}",
            headers=headers,
            timeout=5
        )
        print(f"Status: {response.status_code}")
        result = response.json()
        print(f"Response: {json.dumps(result, indent=2)}")
        return response.status_code == 200
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def test_analyze_repository(api_key, repo_id):
    """Test analyzing a repository"""
    print("\n" + "=" * 60)
    print("Testing Analyze Repository")
    print("=" * 60)
    print("This may take a while as it clones and analyzes the repository...")
    
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    
    data = {"repository_id": repo_id}
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/analyze",
            headers=headers,
            json=data,
            timeout=120  # Longer timeout for analysis
        )
        print(f"Status: {response.status_code}")
        result = response.json()
        print(f"Response: {json.dumps(result, indent=2)}")
        
        if response.status_code == 200:
            print("✓ Repository analyzed successfully")
            return True
        else:
            print("✗ Failed to analyze repository")
            return False
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def test_get_dependencies(api_key, repo_id):
    """Test getting dependencies"""
    print("\n" + "=" * 60)
    print("Testing Get Dependencies")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}"}
    
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/dependencies",
            headers=headers,
            timeout=5
        )
        print(f"Status: {response.status_code}")
        deps = response.json()
        print(f"Found {len(deps)} dependencies")
        if deps:
            print(f"\nFirst 5 dependencies:")
            for dep in deps[:5]:
                print(f"  - {dep['name']} {dep['version']} ({dep['package_manager']})")
        return response.status_code == 200
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def test_search_dependencies(api_key):
    """Test searching dependencies"""
    print("\n" + "=" * 60)
    print("Testing Search Dependencies")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}"}
    
    # Search for a common package
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/dependencies/search",
            headers=headers,
            params={"name": "express"},
            timeout=5
        )
        print(f"Status: {response.status_code}")
        deps = response.json()
        print(f"Found {len(deps)} repositories using 'express'")
        if deps:
            print(f"\nFirst result:")
            print(json.dumps(deps[0], indent=2))
        return response.status_code == 200
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

def main():
    print("=" * 60)
    print("Wavelength Architecture Decoder - Repository & Dependency Tests")
    print("=" * 60)
    
    # Check if server is running
    if not test_health():
        print("\n✗ Server is not running. Please start the server first:")
        print("  cargo run")
        sys.exit(1)
    
    # Get API key
    print("\n" + "=" * 60)
    print("Getting API Key")
    print("=" * 60)
    api_key = get_api_key()
    if not api_key:
        print("✗ Failed to get API key")
        sys.exit(1)
    print(f"✓ Got API key: {api_key[:20]}...")
    
    # Test repository creation
    repo_id = test_create_repository(api_key)
    if not repo_id:
        print("\n✗ Cannot continue without repository ID")
        sys.exit(1)
    
    # Test listing repositories
    test_list_repositories(api_key)
    
    # Test getting repository
    test_get_repository(api_key, repo_id)
    
    # Test analyzing repository (this clones and extracts dependencies)
    if test_analyze_repository(api_key, repo_id):
        # Test getting dependencies
        test_get_dependencies(api_key, repo_id)
        
        # Test searching dependencies
        test_search_dependencies(api_key)
    
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    print("✓ All tests completed!")
    print("\nNote: Some tests may have failed if the test repository")
    print("      doesn't have package files (package.json, etc.)")

if __name__ == "__main__":
    main()

