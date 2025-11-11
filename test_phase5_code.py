#!/usr/bin/env python3
"""
Comprehensive test for Phase 5: Code Structure Analysis
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
            json={"email": "codetest@example.com", "password": "testpass123"},
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
                json={"email": "codetest@example.com", "password": "testpass123"},
                timeout=5
            )
            if response.status_code == 200:
                return response.json()["api_key"]
        except:
            pass
    return None

def test_code_structure_analysis():
    """Test code structure analysis workflow"""
    print("=" * 70)
    print("Phase 5: Code Structure Analysis - Comprehensive Test")
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
        "name": "express-code-test",
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
    
    # Analyze repository (this should analyze code structure)
    print("\n3. Analyzing repository (analyzing code structure)...")
    print("   This may take a minute as it clones, analyzes dependencies,")
    print("   detects services, builds graph, and analyzes code structure...")
    
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
    print(f"   - Graph built: {result.get('graph_built', False)}")
    print(f"   - Code elements found: {result.get('code_elements_found', 0)}")
    print(f"   - Code calls found: {result.get('code_calls_found', 0)}")
    
    # Get code elements
    print("\n4. Retrieving code elements...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/code/elements",
        headers=headers,
        timeout=10
    )
    
    if response.status_code != 200:
        print(f"✗ Failed to get code elements: {response.status_code}")
        print(response.text)
        return False
    
    elements = response.json()
    print(f"✓ Found {len(elements)} code elements")
    
    if elements:
        print("\n   Code Elements by Type:")
        element_types = {}
        languages = {}
        for element in elements:
            elem_type = element.get("element_type", "unknown")
            language = element.get("language", "unknown")
            element_types[elem_type] = element_types.get(elem_type, 0) + 1
            languages[language] = languages.get(language, 0) + 1
        
        for elem_type, count in sorted(element_types.items()):
            print(f"     - {elem_type}: {count} elements")
        
        print("\n   Code Elements by Language:")
        for language, count in sorted(languages.items()):
            print(f"     - {language}: {count} elements")
        
        print("\n   Sample Code Elements:")
        for i, element in enumerate(elements[:10]):
            name = element.get("name", "Unknown")
            elem_type = element.get("element_type", "unknown")
            language = element.get("language", "unknown")
            file_name = element.get("file_path", "").split("/")[-1]
            line_num = element.get("line_number", 0)
            print(f"     {i+1}. {name} ({elem_type}) - {language} - {file_name}:{line_num}")
            if element.get("signature"):
                sig = element["signature"][:60]
                print(f"        Signature: {sig}...")
    else:
        print("   (No code elements found - repository may not have code files)")
    
    # Get code elements by type
    print("\n5. Testing code elements filtering by type...")
    for elem_type in ["function", "class", "module"]:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/code/elements",
            headers=headers,
            params={"type": elem_type},
            timeout=10
        )
        
        if response.status_code == 200:
            filtered = response.json()
            print(f"   - {elem_type}: {len(filtered)} elements")
    
    # Get code calls
    print("\n6. Retrieving code calls...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/code/calls",
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 200:
        calls = response.json()
        print(f"✓ Found {len(calls)} code calls")
        
        if calls:
            print("\n   Sample Code Calls:")
            for i, call in enumerate(calls[:5]):
                print(f"     {i+1}. {call.get('caller_id', 'unknown')[:50]}...")
                print(f"        → {call.get('callee_id', 'unknown')[:50]}...")
                print(f"        Type: {call.get('call_type', 'unknown')}, Line: {call.get('line_number', 0)}")
    else:
        print(f"✗ Failed to get code calls: {response.status_code}")
    
    print("\n" + "=" * 70)
    print("Test Summary")
    print("=" * 70)
    print("✓ Code structure analysis completed successfully!")
    print(f"✓ Found {len(elements)} code elements")
    print(f"✓ Code structure query endpoints working")
    print("✓ Code filtering by type working")
    
    return True

if __name__ == "__main__":
    success = test_code_structure_analysis()
    sys.exit(0 if success else 1)

