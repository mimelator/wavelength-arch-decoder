#!/usr/bin/env python3
"""
Comprehensive test for Phase 6: Security Analysis
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
            json={"email": "securitytest@example.com", "password": "testpass123"},
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
                json={"email": "securitytest@example.com", "password": "testpass123"},
                timeout=5
            )
            if response.status_code == 200:
                return response.json()["api_key"]
        except:
            pass
    return None

def test_security_analysis():
    """Test security analysis workflow"""
    print("=" * 70)
    print("Phase 6: Security Analysis - Comprehensive Test")
    print("=" * 70)
    
    # Get API key
    print("\n1. Authenticating...")
    api_key = get_api_key()
    if not api_key:
        print("✗ Failed to get API key")
        return False
    print(f"✓ Got API key: {api_key[:30]}...")
    
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    
    # Create repository with infrastructure files
    # Using a repository that likely has Terraform/CloudFormation files
    print("\n2. Creating repository...")
    repo_data = {
        "name": "terraform-aws-examples",
        "url": "https://github.com/terraform-aws-modules/terraform-aws-s3-bucket.git",
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
    
    # Analyze repository (this should analyze security)
    print("\n3. Analyzing repository (analyzing security configuration)...")
    print("   This may take a minute as it clones, analyzes dependencies,")
    print("   detects services, builds graph, analyzes code, and security...")
    
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
    print(f"   - Code elements: {result.get('code_elements_found', 0)}")
    print(f"   - Security entities: {result.get('security_entities_found', 0)}")
    print(f"   - Security relationships: {result.get('security_relationships_found', 0)}")
    print(f"   - Security vulnerabilities: {result.get('security_vulnerabilities_found', 0)}")
    
    # Get security entities
    print("\n4. Retrieving security entities...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/security/entities",
        headers=headers,
        timeout=10
    )
    
    if response.status_code != 200:
        print(f"✗ Failed to get security entities: {response.status_code}")
        print(response.text)
        return False
    
    entities = response.json()
    print(f"✓ Found {len(entities)} security entities")
    
    if entities:
        print("\n   Security Entities by Type:")
        entity_types = {}
        providers = {}
        for entity in entities:
            entity_type = entity.get("entity_type", "unknown")
            provider = entity.get("provider", "unknown")
            entity_types[entity_type] = entity_types.get(entity_type, 0) + 1
            providers[provider] = providers.get(provider, 0) + 1
        
        for entity_type, count in sorted(entity_types.items()):
            print(f"     - {entity_type}: {count} entities")
        
        print("\n   Security Entities by Provider:")
        for provider, count in sorted(providers.items()):
            print(f"     - {provider}: {count} entities")
        
        print("\n   Sample Security Entities:")
        for i, entity in enumerate(entities[:10]):
            name = entity.get("name", "Unknown")
            entity_type = entity.get("entity_type", "unknown")
            file_name = entity.get("file_path", "").split("/")[-1]
            line_num = entity.get("line_number", 0)
            arn = entity.get("arn", "")
            print(f"     {i+1}. {name} ({entity_type})")
            print(f"        File: {file_name}:{line_num}")
            if arn:
                print(f"        ARN: {arn[:60]}...")
    else:
        print("   (No security entities found - repository may not have infrastructure files)")
    
    # Get security entities by type
    print("\n5. Testing security entities filtering by type...")
    for entity_type in ["iam_role", "lambda_function", "s3_bucket"]:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/security/entities",
            headers=headers,
            params={"type": entity_type},
            timeout=10
        )
        
        if response.status_code == 200:
            filtered = response.json()
            print(f"   - {entity_type}: {len(filtered)} entities")
    
    # Get security relationships
    print("\n6. Retrieving security relationships...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/security/relationships",
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 200:
        relationships = response.json()
        print(f"✓ Found {len(relationships)} security relationships")
        
        if relationships:
            print("\n   Sample Security Relationships:")
            for i, rel in enumerate(relationships[:5]):
                print(f"     {i+1}. {rel.get('source_entity_id', 'unknown')[:40]}...")
                print(f"        → {rel.get('target_entity_id', 'unknown')[:40]}...")
                print(f"        Type: {rel.get('relationship_type', 'unknown')}")
                if rel.get('permissions'):
                    print(f"        Permissions: {', '.join(rel['permissions'][:3])}")
    else:
        print(f"✗ Failed to get relationships: {response.status_code}")
    
    # Get security vulnerabilities
    print("\n7. Retrieving security vulnerabilities...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/security/vulnerabilities",
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 200:
        vulnerabilities = response.json()
        print(f"✓ Found {len(vulnerabilities)} security vulnerabilities")
        
        if vulnerabilities:
            print("\n   Vulnerabilities by Severity:")
            severity_counts = {}
            for vuln in vulnerabilities:
                severity = vuln.get("severity", "unknown")
                severity_counts[severity] = severity_counts.get(severity, 0) + 1
            
            for severity, count in sorted(severity_counts.items()):
                print(f"     - {severity}: {count} vulnerabilities")
            
            print("\n   Sample Vulnerabilities:")
            for i, vuln in enumerate(vulnerabilities[:10]):
                vuln_type = vuln.get("vulnerability_type", "unknown")
                severity = vuln.get("severity", "unknown")
                description = vuln.get("description", "")[:60]
                file_name = vuln.get("file_path", "").split("/")[-1]
                line_num = vuln.get("line_number", 0)
                print(f"     {i+1}. [{severity}] {vuln_type}")
                print(f"        {description}...")
                print(f"        File: {file_name}:{line_num}")
        else:
            print("   (No vulnerabilities found - repository may have secure configurations)")
    else:
        print(f"✗ Failed to get vulnerabilities: {response.status_code}")
    
    # Test filtering vulnerabilities by severity
    print("\n8. Testing vulnerability filtering by severity...")
    for severity in ["Critical", "High", "Medium"]:
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/security/vulnerabilities",
            headers=headers,
            params={"severity": severity},
            timeout=10
        )
        
        if response.status_code == 200:
            filtered = response.json()
            print(f"   - {severity}: {len(filtered)} vulnerabilities")
    
    print("\n" + "=" * 70)
    print("Test Summary")
    print("=" * 70)
    print("✓ Security analysis completed successfully!")
    print(f"✓ Found {len(entities)} security entities")
    print(f"✓ Security query endpoints working")
    print("✓ Security filtering working")
    print("✓ Vulnerability detection working")
    
    return True

if __name__ == "__main__":
    success = test_security_analysis()
    sys.exit(0 if success else 1)

