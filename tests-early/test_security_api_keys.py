#!/usr/bin/env python3
"""Test security analysis including API key detection"""

import requests
import json
import sys
import time

BASE_URL = "http://localhost:8080"

def test_health():
    """Check if server is running"""
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        return response.status_code == 200
    except:
        return False

def get_repositories():
    """Get all repositories"""
    try:
        response = requests.get(f"{BASE_URL}/api/v1/repositories", timeout=10)
        if response.status_code == 200:
            return response.json()
        return []
    except Exception as e:
        print(f"Error getting repositories: {e}")
        return []

def analyze_repository(repo_id):
    """Analyze a repository"""
    print(f"\n🔍 Analyzing repository {repo_id}...")
    print("   This will check for API keys, Firebase rules, env templates, etc.")
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/analyze",
            json={"repository_id": repo_id},
            timeout=300  # 5 minutes for full analysis
        )
        
        if response.status_code == 200:
            result = response.json()
            return result
        else:
            print(f"✗ Analysis failed: {response.status_code}")
            print(response.text)
            return None
    except Exception as e:
        print(f"✗ Error during analysis: {e}")
        return None

def get_security_entities(repo_id):
    """Get security entities for a repository"""
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/security/entities",
            timeout=10
        )
        if response.status_code == 200:
            return response.json()
        return []
    except Exception as e:
        print(f"Error getting security entities: {e}")
        return []

def get_security_vulnerabilities(repo_id):
    """Get security vulnerabilities for a repository"""
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/security/vulnerabilities",
            timeout=10
        )
        if response.status_code == 200:
            return response.json()
        return []
    except Exception as e:
        print(f"Error getting vulnerabilities: {e}")
        return []

def get_security_relationships(repo_id):
    """Get security relationships for a repository"""
    try:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/security/relationships",
            timeout=10
        )
        if response.status_code == 200:
            return response.json()
        return []
    except Exception as e:
        print(f"Error getting relationships: {e}")
        return []

def main():
    print("=" * 70)
    print("Security Analysis Test - API Key Detection & More")
    print("=" * 70)
    
    # Check server
    if not test_health():
        print("\n✗ Server is not running. Please start it with: cargo run")
        sys.exit(1)
    print("✓ Server is running")
    
    # Get repositories
    print("\n📋 Fetching repositories...")
    repos = get_repositories()
    if not repos:
        print("✗ No repositories found. Please add a repository first.")
        sys.exit(1)
    
    print(f"✓ Found {len(repos)} repository(ies)")
    for repo in repos:
        print(f"   - {repo.get('name', 'Unknown')} ({repo.get('id', 'no-id')})")
    
    # Use the first repository (likely wavelength-hub)
    repo = repos[0]
    repo_id = repo['id']
    repo_name = repo.get('name', 'Unknown')
    
    print(f"\n🎯 Testing with repository: {repo_name}")
    
    # Analyze repository
    result = analyze_repository(repo_id)
    if not result:
        print("\n✗ Analysis failed")
        sys.exit(1)
    
    print("\n✓ Analysis complete!")
    print(f"   Results: {json.dumps(result.get('results', {}), indent=2)}")
    
    # Wait a moment for data to be stored
    print("\n⏳ Waiting for data to be stored...")
    time.sleep(2)
    
    # Get security entities
    print("\n🔒 Fetching security entities...")
    entities = get_security_entities(repo_id)
    print(f"✓ Found {len(entities)} security entities")
    
    if entities:
        print("\n📊 Security Entities Breakdown:")
        entity_types = {}
        for entity in entities:
            entity_type = entity.get('entity_type', 'unknown')
            entity_types[entity_type] = entity_types.get(entity_type, 0) + 1
        
        for etype, count in sorted(entity_types.items()):
            print(f"   - {etype}: {count}")
        
        # Show some examples
        print("\n📝 Sample Entities:")
        for entity in entities[:5]:
            print(f"   - {entity.get('name', 'Unknown')} ({entity.get('entity_type', 'unknown')})")
            print(f"     File: {entity.get('file_path', 'unknown')}")
            if entity.get('line_number'):
                print(f"     Line: {entity.get('line_number')}")
            print()
    else:
        print("   ⚠️  No security entities found")
    
    # Get vulnerabilities
    print("\n⚠️  Fetching security vulnerabilities...")
    vulnerabilities = get_security_vulnerabilities(repo_id)
    print(f"✓ Found {len(vulnerabilities)} vulnerabilities")
    
    if vulnerabilities:
        print("\n🚨 Vulnerabilities:")
        for vuln in vulnerabilities:
            print(f"   - [{vuln.get('severity', 'unknown')}] {vuln.get('vulnerability_type', 'unknown')}")
            print(f"     {vuln.get('description', 'No description')}")
            print(f"     File: {vuln.get('file_path', 'unknown')}")
            if vuln.get('line_number'):
                print(f"     Line: {vuln.get('line_number')}")
            print()
    else:
        print("   ✓ No vulnerabilities found")
    
    # Get relationships
    print("\n🔗 Fetching security relationships...")
    relationships = get_security_relationships(repo_id)
    print(f"✓ Found {len(relationships)} relationships")
    
    if relationships:
        print("\n📊 Relationship Types:")
        rel_types = {}
        for rel in relationships:
            rel_type = rel.get('relationship_type', 'unknown')
            rel_types[rel_type] = rel_types.get(rel_type, 0) + 1
        
        for rtype, count in sorted(rel_types.items()):
            print(f"   - {rtype}: {count}")
    
    print("\n" + "=" * 70)
    print("Test Complete!")
    print("=" * 70)
    print(f"\nSummary:")
    print(f"  - Security Entities: {len(entities)}")
    print(f"  - Vulnerabilities: {len(vulnerabilities)}")
    print(f"  - Relationships: {len(relationships)}")

if __name__ == "__main__":
    main()

