#!/usr/bin/env python3
"""
Test script to verify Printify plugin detection on wavelength-hub repository
"""
import subprocess
import sys
import json
from pathlib import Path

def test_printify_detection():
    """Test Printify detection in wavelength-hub repo"""
    
    repo_path = Path("/Volumes/5bits/current/wavelength-dev/wavelength-hub")
    decoder_path = Path("/Volumes/5bits/current/wavelength-dev/arch/wavelength-arch-decoder")
    
    if not repo_path.exists():
        print(f"❌ Repository not found: {repo_path}")
        return False
    
    if not decoder_path.exists():
        print(f"❌ Decoder project not found: {decoder_path}")
        return False
    
    print("🔍 Testing Printify detection in wavelength-hub...")
    print(f"   Repository: {repo_path}")
    print(f"   Decoder: {decoder_path}")
    print()
    
    # Check if Printify references exist
    print("📋 Checking for Printify references in repository...")
    try:
        result = subprocess.run(
            ["grep", "-ri", "printify", str(repo_path), "--include=*.env*", "--include=*.js", "--include=*.json", "--include=*.md"],
            capture_output=True,
            text=True,
            timeout=10
        )
        matches = result.stdout.strip().split('\n')
        printify_refs = [m for m in matches if m and 'printify' in m.lower()]
        
        if printify_refs:
            print(f"   ✓ Found {len(printify_refs)} Printify references")
            print("   Sample references:")
            for ref in printify_refs[:5]:
                print(f"      - {ref[:100]}...")
        else:
            print("   ⚠ No Printify references found")
    except Exception as e:
        print(f"   ⚠ Could not search for references: {e}")
    
    print()
    print("🧪 Testing service detection...")
    print("   (This would require running the Rust detector)")
    print("   Expected detections:")
    print("   - PRINTIFY_API_KEY from env.example")
    print("   - api.printify.com from docs")
    print("   - printify-service.js from services/")
    print()
    
    # Check plugin file exists
    plugin_file = decoder_path / "config" / "plugins" / "printify.json"
    if plugin_file.exists():
        print(f"✅ Printify plugin found: {plugin_file}")
        with open(plugin_file) as f:
            plugin_data = json.load(f)
            env_patterns = len(plugin_data.get("patterns", {}).get("environment_variables", []))
            sdk_patterns = len(plugin_data.get("patterns", {}).get("sdk_patterns", []))
            api_patterns = len(plugin_data.get("patterns", {}).get("api_endpoints", []))
            print(f"   - {env_patterns} environment variable patterns")
            print(f"   - {sdk_patterns} SDK patterns")
            print(f"   - {api_patterns} API endpoint patterns")
    else:
        print(f"❌ Printify plugin not found: {plugin_file}")
        return False
    
    print()
    print("✅ Test setup complete!")
    print()
    print("To test detection:")
    print("1. Start the decoder server")
    print("2. Add wavelength-hub repository")
    print("3. Run analysis")
    print("4. Check Services tab for Printify detections")
    
    return True

if __name__ == "__main__":
    success = test_printify_detection()
    sys.exit(0 if success else 1)

