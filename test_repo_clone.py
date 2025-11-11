#!/usr/bin/env python3
"""
Test script to verify GitHub token works and repository cloning
"""
import os
import sys
import requests
import json
from pathlib import Path

# Load environment variables from .env.local
env_file = Path('.env.local')
if env_file.exists():
    with open(env_file) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith('#') and '=' in line:
                key, value = line.split('=', 1)
                os.environ[key.strip()] = value.strip()

BASE_URL = os.getenv('BASE_URL', 'http://localhost:8080')
GITHUB_TOKEN = os.getenv('GITHUB_TOKEN')

def test_github_token():
    """Test if GitHub token is valid"""
    print("\n=== Testing GitHub Token ===")
    
    if not GITHUB_TOKEN:
        print("❌ GITHUB_TOKEN not found in environment")
        print("   Make sure .env.local exists with GITHUB_TOKEN=...")
        return False
    
    print(f"✓ Found GITHUB_TOKEN: {GITHUB_TOKEN[:10]}...")
    
    # Test GitHub API access
    headers = {
        'Authorization': f'token {GITHUB_TOKEN}',
        'Accept': 'application/vnd.github.v3+json'
    }
    
    try:
        response = requests.get('https://api.github.com/user', headers=headers, timeout=5)
        if response.status_code == 200:
            user_data = response.json()
            print(f"✓ GitHub token is valid!")
            print(f"  Authenticated as: {user_data.get('login', 'unknown')}")
            return True
        else:
            print(f"❌ GitHub token validation failed: {response.status_code}")
            print(f"  Response: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Error testing GitHub token: {e}")
        return False

def test_repository_clone():
    """Test cloning a repository via the API"""
    print("\n=== Testing Repository Clone ===")
    
    # Use a public test repository
    test_repo = {
        'name': 'test-repo-clone',
        'url': 'https://github.com/octocat/Hello-World.git',
        'branch': 'master'  # This repo uses 'master' not 'main'
    }
    
    print(f"Creating repository: {test_repo['name']}")
    print(f"URL: {test_repo['url']}")
    print(f"Branch: {test_repo['branch']}")
    
    try:
        # Create repository
        response = requests.post(
            f'{BASE_URL}/api/v1/repositories',
            json=test_repo,
            timeout=30
        )
        
        if response.status_code not in [200, 201]:
            print(f"❌ Failed to create repository: {response.status_code}")
            print(f"  Response: {response.text}")
            return False
        
        repo_data = response.json()
        repo_id = repo_data.get('id')
        print(f"✓ Repository created: {repo_id}")
        
        # Analyze repository (this will clone it)
        print("\nAnalyzing repository (this will clone it)...")
        analyze_response = requests.post(
            f'{BASE_URL}/api/v1/repositories/{repo_id}/analyze',
            json={'repository_id': repo_id},
            timeout=120  # Cloning can take time
        )
        
        if analyze_response.status_code == 200:
            print("✓ Repository analysis started successfully!")
            result = analyze_response.json()
            print(f"  Dependencies found: {result.get('dependencies_found', 0)}")
            print(f"  Services found: {result.get('services_found', 0)}")
            return True
        else:
            print(f"❌ Failed to analyze repository: {analyze_response.status_code}")
            print(f"  Response: {analyze_response.text}")
            return False
            
    except Exception as e:
        print(f"❌ Error during repository clone test: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    print("=" * 60)
    print("Repository Clone Test")
    print("=" * 60)
    
    # Test GitHub token
    if not test_github_token():
        print("\n⚠️  GitHub token test failed, but continuing with clone test...")
    
    # Test repository cloning
    success = test_repository_clone()
    
    print("\n" + "=" * 60)
    if success:
        print("✓ All tests passed!")
        sys.exit(0)
    else:
        print("❌ Tests failed")
        sys.exit(1)

if __name__ == '__main__':
    main()

