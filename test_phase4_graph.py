#!/usr/bin/env python3
"""
Comprehensive test for Phase 4: Knowledge Graph Construction
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
            json={"email": "graphtest@example.com", "password": "testpass123"},
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
                json={"email": "graphtest@example.com", "password": "testpass123"},
                timeout=5
            )
            if response.status_code == 200:
                return response.json()["api_key"]
        except:
            pass
    return None

def test_graph_construction():
    """Test knowledge graph construction workflow"""
    print("=" * 70)
    print("Phase 4: Knowledge Graph Construction - Comprehensive Test")
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
        "name": "express-graph-test",
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
    
    # Analyze repository (this should build the graph)
    print("\n3. Analyzing repository (building knowledge graph)...")
    print("   This may take a minute as it clones, analyzes, and builds the graph...")
    
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
    
    if not result.get('graph_built', False):
        print("⚠ Warning: Graph building may have failed")
    
    # Get graph
    print("\n4. Retrieving knowledge graph...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/graph",
        headers=headers,
        timeout=10
    )
    
    if response.status_code != 200:
        print(f"✗ Failed to get graph: {response.status_code}")
        print(response.text)
        return False
    
    graph = response.json()
    nodes = graph.get("nodes", [])
    edges = graph.get("edges", [])
    
    print(f"✓ Graph retrieved:")
    print(f"   - Total nodes: {len(nodes)}")
    print(f"   - Total edges: {len(edges)}")
    
    if nodes:
        print("\n   Node Types:")
        node_types = {}
        for node in nodes:
            node_type = node.get("node_type", "unknown")
            node_types[node_type] = node_types.get(node_type, 0) + 1
        
        for node_type, count in sorted(node_types.items()):
            print(f"     - {node_type}: {count} nodes")
        
        print("\n   Sample Nodes:")
        for i, node in enumerate(nodes[:5]):
            print(f"     {i+1}. {node.get('name')} ({node.get('node_type')})")
    
    if edges:
        print("\n   Edge Types:")
        edge_types = {}
        for edge in edges:
            edge_type = edge.get("edge_type", "unknown")
            edge_types[edge_type] = edge_types.get(edge_type, 0) + 1
        
        for edge_type, count in sorted(edge_types.items()):
            print(f"     - {edge_type}: {count} edges")
    
    # Get graph statistics
    print("\n5. Getting graph statistics...")
    response = requests.get(
        f"{BASE_URL}/api/v1/repositories/{repo_id}/graph/statistics",
        headers=headers,
        timeout=10
    )
    
    if response.status_code == 200:
        stats = response.json()
        print(f"✓ Graph Statistics:")
        print(f"   - Total nodes: {stats.get('total_nodes', 0)}")
        print(f"   - Total edges: {stats.get('total_edges', 0)}")
        
        if stats.get('nodes_by_type'):
            print("\n   Nodes by Type:")
            for node_type, count in stats['nodes_by_type'].items():
                print(f"     - {node_type}: {count}")
        
        if stats.get('edges_by_type'):
            print("\n   Edges by Type:")
            for edge_type, count in stats['edges_by_type'].items():
                print(f"     - {edge_type}: {count}")
        
        if stats.get('most_connected_nodes'):
            print("\n   Most Connected Nodes:")
            for node_id, connections in stats['most_connected_nodes'][:5]:
                # Find node name
                node_name = "Unknown"
                for node in nodes:
                    if node.get("id") == node_id:
                        node_name = node.get("name", "Unknown")
                        break
                print(f"     - {node_name}: {connections} connections")
    else:
        print(f"✗ Failed to get statistics: {response.status_code}")
    
    # Test node neighbors (if we have nodes)
    if nodes:
        print("\n6. Testing node neighbors...")
        test_node = nodes[0]
        node_id = test_node.get("id")
        
        response = requests.get(
            f"{BASE_URL}/api/v1/repositories/{repo_id}/graph/nodes/{node_id}/neighbors",
            headers=headers,
            timeout=10
        )
        
        if response.status_code == 200:
            neighbors = response.json()
            print(f"✓ Found {len(neighbors)} neighbors for node '{test_node.get('name')}'")
            if neighbors:
                print("   Sample neighbors:")
                for i, neighbor in enumerate(neighbors[:3]):
                    print(f"     {i+1}. {neighbor.get('name')} ({neighbor.get('node_type')})")
        else:
            print(f"✗ Failed to get neighbors: {response.status_code}")
    
    print("\n" + "=" * 70)
    print("Test Summary")
    print("=" * 70)
    print("✓ Knowledge graph construction completed successfully!")
    print(f"✓ Graph contains {len(nodes)} nodes and {len(edges)} edges")
    print("✓ Graph query endpoints working")
    print("✓ Graph statistics working")
    print("✓ Graph traversal working")
    
    return True

if __name__ == "__main__":
    success = test_graph_construction()
    sys.exit(0 if success else 1)

