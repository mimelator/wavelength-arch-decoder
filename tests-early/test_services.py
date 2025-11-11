#!/usr/bin/env python3
"""
Test script for Service Detection API endpoints
"""
import requests
import json
import sys
import time

BASE_URL = "http://localhost:8080"

def get_api_key():
    """Get API key by logging in"""
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

def test_service_detection(api_key):
    """Test service detection with a repository that has services"""
    print("=" * 60)
    print("Testing Service Detection")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    
    # Create a repository (we'll use express which has AWS SDK)
    print("\n1. Creating repository...")
    repo_data = {
        "name": "express-services",
        "url": "https://github.com/expressjs/express.git",
        "branch": "master"
    }
    
    response = requests.post(
        f"{BASE_URL}/api/v1/repositories",
        headers=headers,
        json=repo_data,
        timeout=10
    )
    
    if response.status_code != 201:
        print(f"✗ Failed to create repository: {response.status_code}")
        return None
    
    repo = response.json()
    repo_id = repo["id"]
    print(f"✓ Repository created: {repo_id}")
    
    # Analyze repository
    print("\n2. Analyzing repository (this will detect services)...")
    analyze_data = {"repository_id": repo_id}
    
    response = requests.post(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/analyze",
        headers=headers,
        json=analyze_data,
        timeout=120
    )
    
    if response.status_code != 200:
        print(f"✗ Failed to analyze repository: {response.status_code}")
        print(response.text)
        return None
    
    result = response.json()
    print(f"✓ Analysis complete:")
    print(f"  - Manifests found: {result.get('manifests_found', 0)}")
    print(f"  - Dependencies: {result.get('total_dependencies', 0)}")
    print(f"  - Services found: {result.get('services_found', 0)}")
    
    # Get services
    print("\n3. Getting detected services...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/services",
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 200:
        services = response.json()
        print(f"✓ Found {len(services)} services")
        
        if services:
            print("\nDetected services:")
            for service in services[:10]:  # Show first 10
                print(f"  - {service['name']} ({service['provider']}) - {service['service_type']}")
                print(f"    File: {service['file_path']}")
                if service.get('line_number'):
                    print(f"    Line: {service['line_number']}")
                print(f"    Confidence: {service['confidence']:.2f}")
                print()
        else:
            print("  (No services detected - repository may not have service configurations)")
    else:
        print(f"✗ Failed to get services: {response.status_code}")
    
    return repo_id

def test_service_search(api_key):
    """Test searching services by provider"""
    print("\n" + "=" * 60)
    print("Testing Service Search")
    print("=" * 60)
    
    headers = {"Authorization": f"Bearer {api_key}"}
    
    # Search by provider
    print("\n1. Searching for AWS services...")
    response = requests.get(
        f"{BASE_URL}/api/v1/services/search",
        headers=headers,
        params={"provider": "aws"},
        timeout=10
    )
    
    if response.status_code == 200:
        services = response.json()
        print(f"✓ Found {len(services)} AWS services")
        for service in services[:5]:
            print(f"  - {service['name']} in repository {service['repository_id']}")
    else:
        print(f"✗ Search failed: {response.status_code}")
    
    # Search by type
    print("\n2. Searching for auth services...")
    response = requests.get(
        f"{BASE_URL}/api/v1/services/search",
        headers=headers,
        params={"type": "auth"},
        timeout=10
    )
    
    if response.status_code == 200:
        services = response.json()
        print(f"✓ Found {len(services)} auth services")
        for service in services[:5]:
            print(f"  - {service['name']} ({service['provider']})")
    else:
        print(f"✗ Search failed: {response.status_code}")

def main():
    print("=" * 60)
    print("Service Detection API Tests")
    print("=" * 60)
    
    # Get API key
    api_key = get_api_key()
    if not api_key:
        print("✗ Failed to get API key")
        sys.exit(1)
    
    # Test service detection
    repo_id = test_service_detection(api_key)
    
    if repo_id:
        # Test service search
        test_service_search(api_key)
    
    print("\n" + "=" * 60)
    print("Test Complete")
    print("=" * 60)

if __name__ == "__main__":
    main()

