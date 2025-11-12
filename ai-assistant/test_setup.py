#!/usr/bin/env python3
"""Quick test to verify AI Assistant setup"""

import sys
import os

def test_imports():
    """Test that all modules can be imported"""
    print("Testing imports...")
    try:
        from src.query_parser import QueryParser
        from src.context_builder import ContextBuilder
        from src.refactoring_analyzer import RefactoringAnalyzer
        from src.prompt_templates import PromptTemplates
        from src.clients import ArchitectureDecoderClient
        print("✅ All imports successful")
        return True
    except ImportError as e:
        print(f"❌ Import error: {e}")
        return False

def test_query_parser():
    """Test query parser"""
    print("\nTesting query parser...")
    try:
        from src.query_parser import QueryParser
        parser = QueryParser()

        test_queries = [
            "What functions use Firebase?",
            "Which services are used?",
            "What would break if I rename getAdminStorage?",
            "What build tools are configured?"
        ]

        for query in test_queries:
            result = parser.parse(query)
            intent = result.get("intent")
            print(f"  Query: '{query}'")
            print(f"    Intent: {intent}")
            print(f"    Topics: {result.get('topics', [])}")
            print(f"    Entities: {result.get('entities', [])[:3]}")

        print("✅ Query parser working")
        return True
    except Exception as e:
        print(f"❌ Query parser error: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_prompt_templates():
    """Test prompt templates"""
    print("\nTesting prompt templates...")
    try:
        from src.prompt_templates import PromptTemplates
        templates = PromptTemplates()

        # Test query prompt
        context = {
            "repository": {"name": "test-repo", "language": "TypeScript"},
            "sources": [
                {
                    "type": "code_element",
                    "name": "testFunction",
                    "file_path": "src/test.ts",
                    "signature": "function testFunction(): void"
                }
            ],
            "related": {}
        }

        intent = {"intent": "find_functions", "topics": [], "entities": []}
        prompt = templates.build_query_prompt("test query", context, intent)

        if len(prompt) > 100:
            print("✅ Prompt templates working")
            print(f"   Generated prompt length: {len(prompt)} characters")
            return True
        else:
            print("❌ Prompt too short")
            return False
    except Exception as e:
        print(f"❌ Prompt templates error: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_client():
    """Test Architecture Decoder client (without making actual requests)"""
    print("\nTesting Architecture Decoder client...")
    try:
        from src.clients import ArchitectureDecoderClient
        client = ArchitectureDecoderClient(base_url="http://localhost:8080")
        print(f"✅ Client initialized with URL: {client.base_url}")
        return True
    except Exception as e:
        print(f"❌ Client error: {e}")
        return False

def main():
    """Run all tests"""
    print("🧪 Testing AI Assistant Setup\n")
    print("=" * 50)

    tests = [
        test_imports,
        test_query_parser,
        test_prompt_templates,
        test_client
    ]

    results = []
    for test in tests:
        try:
            result = test()
            results.append(result)
        except Exception as e:
            print(f"❌ Test failed with exception: {e}")
            results.append(False)

    print("\n" + "=" * 50)
    print(f"\n📊 Results: {sum(results)}/{len(results)} tests passed")

    if all(results):
        print("✅ All tests passed! Setup looks good.")
        print("\nNext steps:")
        print("1. Copy .env.example to .env")
        print("2. Add your OPENAI_API_KEY to .env")
        print("3. Make sure Architecture Decoder is running")
        print("4. Run: ./start.sh")
        return 0
    else:
        print("❌ Some tests failed. Please check the errors above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())

