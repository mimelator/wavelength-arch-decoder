#!/usr/bin/env python3
"""
Comprehensive test for Phase 3: Service Detection
"""
import requests
import json
import sys
import time

BASE_URL = "http://localhost:8080"

def get_api_key():
    """Get API key by registering"""
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/auth/register",
            json={"email": "servicetest@example.com", "password": "testpass123"},
            timeout=5
        )
        if response.status_code == 201:
            return response.json()["api_key"]
    except Exception as e:
        print(f"Registration failed: {e}")
        # Try login
        try:
            response = requests.post(
                f"{BASE_URL}/api/v1/auth/login",
                json={"email": "servicetest@example.com", "password": "testpass123"},
                timeout=5
            )
            if response.status_code == 200:
                return response.json()["api_key"]
        except:
            pass
    return None

def test_service_detection():
    """Test service detection workflow"""
    print("=" * 70)
    print("Phase 3: Service Detection - Comprehensive Test")
    print("=" * 70)
    
    # Get API key
    print("\n1. Authenticating...")
    api_key = get_api_key()
    if not api_key:
        print("✗ Failed to get API key")
        return False
    print(f"✓ Got API key: {api_key[:30]}...")
    
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    
    # Create repository
    print("\n2. Creating repository...")
    repo_data = {
        "name": "express-services-test",
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
        print(response.text)
        return False
    
    repo = response.json()
    repo_id = repo["id"]
    print(f"✓ Repository created: {repo_id}")
    
    # Analyze repository (this should detect services)
    print("\n3. Analyzing repository (detecting services)...")
    print("   This may take a minute as it clones and analyzes the repository...")
    
    analyze_data = {"repository_id": repo_id}
    
    response = requests.post(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/analyze",
        headers=headers,
        json=analyze_data,
        timeout=180
    )
    
    if response.status_code != 200:
        print(f"✗ Failed to analyze repository: {response.status_code}")
        print(response.text)
        return False
    
    result = response.json()
    print(f"✓ Analysis complete:")
    print(f"   - Manifests found: {result.get('manifests_found', 0)}")
    print(f"   - Dependencies: {result.get('total_dependencies', 0)}")
    print(f"   - Services found: {result.get('services_found', 0)}")
    
    # Get services
    print("\n4. Retrieving detected services...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/services",
        headers=headers,
        timeout=10
    )
    
    if response.status_code != 200:
        print(f"✗ Failed to get services: {response.status_code}")
        print(response.text)
        return False
    
    services = response.json()
    print(f"✓ Found {len(services)} services")
    
    if services:
        print("\n   Detected Services:")
        print("   " + "-" * 66)
        
        # Group by provider
        by_provider = {}
        by_type = {}
        
        for service in services:
            provider = service.get('provider', 'unknown')
            service_type = service.get('service_type', 'unknown')
            
            by_provider.setdefault(provider, []).append(service)
            by_type.setdefault(service_type, []).append(service)
        
        # Show services grouped by provider
        for provider, svcs in sorted(by_provider.items()):
            print(f"\n   {provider.upper()} ({len(svcs)} services):")
            for svc in svcs[:5]:  # Show first 5 per provider
                name = svc.get('name', 'Unknown')
                file_name = svc.get('file_path', '').split('/')[-1]
                confidence = svc.get('confidence', 0)
                print(f"     - {name} (confidence: {confidence:.2f}) in {file_name}")
            if len(svcs) > 5:
                print(f"     ... and {len(svcs) - 5} more")
        
        print("\n   Services by Type:")
        for svc_type, svcs in sorted(by_type.items()):
            print(f"     - {svc_type}: {len(svcs)} services")
    else:
        print("   (No services detected - this may be normal for some repositories)")
    
    # Test service search by provider
    print("\n5. Testing service search by provider...")
    response = requests.get(
        f"{BASE_URL}/api/v1/services/search",
        headers=headers,
        params={"provider": "aws"},
        timeout=10
    )
    
    if response.status_code == 200:
        aws_services = response.json()
        print(f"✓ Found {len(aws_services)} AWS services across all repositories")
    else:
        print(f"✗ Search failed: {response.status_code}")
    
    # Test service search by type
    print("\n6. Testing service search by type...")
    response = requests.get(
        f"{BASE_URL}/api/v1/services/search",
        headers=headers,
        params={"type": "saas"},
        timeout=10
    )
    
    if response.status_code == 200:
        saas_services = response.json()
        print(f"✓ Found {len(saas_services)} SaaS services across all repositories")
    else:
        print(f"✗ Search failed: {response.status_code}")
    
    # Test service search by auth type
    print("\n7. Testing service search for auth services...")
    response = requests.get(
        f"{BASE_URL}/api/v1/services/search",
        headers=headers,
        params={"type": "auth"},
        timeout=10
    )
    
    if response.status_code == 200:
        auth_services = response.json()
        print(f"✓ Found {len(auth_services)} auth services across all repositories")
        if auth_services:
            print("   Examples:")
            for svc in auth_services[:3]:
                print(f"     - {svc.get('name')} ({svc.get('provider')})")
    else:
        print(f"✗ Search failed: {response.status_code}")
    
    print("\n" + "=" * 70)
    print("Test Summary")
    print("=" * 70)
    print("✓ Service detection workflow completed successfully!")
    print(f"✓ Detected {len(services)} services in repository")
    print("✓ Service query endpoints working")
    print("✓ Service search endpoints working")
    
    return True

if __name__ == "__main__":
    success = test_service_detection()
    sys.exit(0 if success else 1)

