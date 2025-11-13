#!/usr/bin/env python3
"""
Validation script for AI Assistant entity queries.
Tests "what xxx exists for this repo" queries across all entity types.
"""

import sys
import json
import httpx
import os
from typing import Dict, Any, List, Tuple
from dotenv import load_dotenv

load_dotenv()

# Configuration
AI_ASSISTANT_URL = os.getenv("AI_ASSISTANT_URL", "http://localhost:8000")
ARCHITECTURE_DECODER_URL = os.getenv("ARCHITECTURE_DECODER_URL", "http://localhost:8080")

# Entity types and their corresponding query patterns
ENTITY_TESTS = [
    {
        "entity_type": "functions",
        "query": "what functions exist for this repo?",
        "expected_intent": "find_functions",
        "expected_source_type": "code_element"
    },
    {
        "entity_type": "services",
        "query": "what services exist for this repo?",
        "expected_intent": "list_services",
        "expected_source_type": "service"
    },
    {
        "entity_type": "dependencies",
        "query": "what dependencies exist for this repo?",
        "expected_intent": "find_dependencies",
        "expected_source_type": "dependency"
    },
    {
        "entity_type": "tools",
        "query": "what tools exist for this repo?",
        "expected_intent": "tool_discovery",
        "expected_source_type": "tool"
    },
    {
        "entity_type": "tests",
        "query": "what tests exist for this repo?",
        "expected_intent": "find_tests",
        "expected_source_type": "test"
    },
    {
        "entity_type": "documentation",
        "query": "what docs exist for this repo?",
        "expected_intent": "find_documentation",
        "expected_source_type": "documentation"
    },
]

class Colors:
    """ANSI color codes for terminal output"""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def print_success(message: str):
    print(f"{Colors.GREEN}✓{Colors.RESET} {message}")

def print_error(message: str):
    print(f"{Colors.RED}✗{Colors.RESET} {message}")

def print_warning(message: str):
    print(f"{Colors.YELLOW}⚠{Colors.RESET} {message}")

def print_info(message: str):
    print(f"{Colors.BLUE}ℹ{Colors.RESET} {message}")

def print_header(message: str):
    print(f"\n{Colors.BOLD}{Colors.BLUE}{'='*60}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.BLUE}{message}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.BLUE}{'='*60}{Colors.RESET}\n")

def get_repositories() -> List[Dict[str, Any]]:
    """Get list of repositories from Architecture Decoder"""
    try:
        response = httpx.get(f"{ARCHITECTURE_DECODER_URL}/api/v1/repositories", timeout=10.0)
        response.raise_for_status()
        repos = response.json()
        if not isinstance(repos, list):
            return []
        return repos
    except Exception as e:
        print_error(f"Failed to fetch repositories: {e}")
        return []

def test_ai_assistant_health() -> bool:
    """Check if AI Assistant is available"""
    try:
        response = httpx.get(f"{AI_ASSISTANT_URL}/health", timeout=5.0)
        response.raise_for_status()
        health = response.json()
        
        if not health.get("decoder_available"):
            print_error(f"Architecture Decoder is not available: {health.get('decoder_error', 'Unknown error')}")
            return False
        
        print_success(f"AI Assistant is available at {AI_ASSISTANT_URL}")
        print_info(f"Architecture Decoder: {health.get('decoder_url')}")
        print_info(f"OpenAI configured: {health.get('openai_configured', False)}")
        return True
    except Exception as e:
        print_error(f"AI Assistant health check failed: {e}")
        return False

def validate_query_response(
    entity_type: str,
    query: str,
    repo_id: str,
    repo_name: str,
    expected_intent: str,
    expected_source_type: str
) -> Tuple[bool, Dict[str, Any]]:
    """Test a single entity query and validate the response"""
    print(f"\nTesting: {Colors.BOLD}{entity_type}{Colors.RESET}")
    print(f"  Query: '{query}'")
    print(f"  Repository: {repo_name} ({repo_id[:8]}...)")
    
    try:
        response = httpx.post(
            f"{AI_ASSISTANT_URL}/api/v1/ai/query",
            json={
                "repository_id": repo_id,
                "query": query,
                "max_results": 10,
                "include_graph": False
            },
            timeout=30.0
        )
        response.raise_for_status()
        data = response.json()
        
        # Validate response structure
        if "answer" not in data:
            print_error("Response missing 'answer' field")
            return False, data
        
        if "intent" not in data:
            print_error("Response missing 'intent' field")
            return False, data
        
        if "sources" not in data:
            print_error("Response missing 'sources' field")
            return False, data
        
        # Validate intent
        actual_intent = data.get("intent", "")
        if actual_intent != expected_intent:
            print_warning(f"Intent mismatch: expected '{expected_intent}', got '{actual_intent}'")
            # Not a failure, but worth noting
        
        # Validate answer is not empty
        answer = data.get("answer", "").strip()
        if not answer:
            print_error("Answer is empty")
            return False, data
        
        # Validate answer doesn't contain disclaimers
        disclaimers = [
            "as an ai",
            "i don't have direct access",
            "based on the information provided, it's not clear",
            "however",
            "remember to"
        ]
        answer_lower = answer.lower()
        found_disclaimers = [d for d in disclaimers if d in answer_lower]
        if found_disclaimers:
            print_warning(f"Answer contains disclaimers: {', '.join(found_disclaimers)}")
        
        # Validate sources
        sources = data.get("sources", [])
        if not sources:
            print_warning(f"No sources returned (this may be valid if no {entity_type} exist)")
        else:
            # Check if sources match expected type
            matching_sources = [s for s in sources if s.get("type") == expected_source_type]
            if matching_sources:
                print_success(f"Found {len(matching_sources)} {expected_source_type} sources")
            else:
                # Check what types we actually got
                source_types = {}
                for s in sources:
                    src_type = s.get("type", "unknown")
                    source_types[src_type] = source_types.get(src_type, 0) + 1
                print_warning(f"No {expected_source_type} sources found. Got: {dict(source_types)}")
        
        # Validate answer summarizes findings
        if sources and len(answer) < 50:
            print_warning("Answer seems too short given that sources were found")
        
        # Print summary
        print(f"  Answer length: {len(answer)} characters")
        print(f"  Sources: {len(sources)} items")
        print(f"  Intent: {actual_intent}")
        
        # Show first part of answer
        answer_preview = answer[:200] + "..." if len(answer) > 200 else answer
        print(f"  Answer preview: {answer_preview[:100]}...")
        
        return True, data
        
    except httpx.HTTPStatusError as e:
        print_error(f"HTTP error {e.response.status_code}: {e.response.text[:200]}")
        return False, {}
    except Exception as e:
        print_error(f"Query failed: {e}")
        return False, {}

def main():
    """Main validation function"""
    print_header("AI Assistant Entity Query Validation")
    
    # Check AI Assistant health
    if not test_ai_assistant_health():
        print_error("AI Assistant is not available. Please start it first.")
        sys.exit(1)
    
    # Get repositories
    print_info("Fetching repositories...")
    repos = get_repositories()
    if not repos:
        print_error("No repositories found. Please analyze at least one repository first.")
        sys.exit(1)
    
    print_success(f"Found {len(repos)} repository(ies)")
    
    # Use the first repository for testing
    test_repo = repos[0]
    repo_id = test_repo.get("id")
    repo_name = test_repo.get("name", "Unknown")
    
    print_info(f"Using repository: {repo_name} ({repo_id})")
    
    # Run tests for each entity type
    results = []
    for test_config in ENTITY_TESTS:
        success, response_data = validate_query_response(
            entity_type=test_config["entity_type"],
            query=test_config["query"],
            repo_id=repo_id,
            repo_name=repo_name,
            expected_intent=test_config["expected_intent"],
            expected_source_type=test_config["expected_source_type"]
        )
        results.append({
            "entity_type": test_config["entity_type"],
            "success": success,
            "response": response_data
        })
    
    # Print summary
    print_header("Validation Summary")
    
    passed = sum(1 for r in results if r["success"])
    total = len(results)
    
    for result in results:
        status = "✓" if result["success"] else "✗"
        color = Colors.GREEN if result["success"] else Colors.RED
        print(f"{color}{status}{Colors.RESET} {result['entity_type']}")
    
    print(f"\n{Colors.BOLD}Results: {passed}/{total} tests passed{Colors.RESET}")
    
    if passed == total:
        print_success("All entity query tests passed!")
        sys.exit(0)
    else:
        print_error(f"{total - passed} test(s) failed")
        sys.exit(1)

if __name__ == "__main__":
    main()

