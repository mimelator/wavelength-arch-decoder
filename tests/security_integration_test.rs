use wavelength_arch_decoder::security::{SecurityAnalyzer, ServiceDetector, ApiKeyDetector};
use tempfile::TempDir;
use std::fs;
use std::path::Path;

/// Test helper to create a realistic Python project with venv
fn create_python_project_with_venv() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // Create source directory
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    
    // Create real application code
    let app_file = src_dir.join("app.py");
    fs::write(&app_file, r#"
import cohere
import openai

# Real API keys (for testing purposes only)
cohere_client = cohere.Client(api_key="sk_test_cohere_12345678901234567890")
openai_client = openai.OpenAI(api_key="sk-test-openai-12345678901234567890")
    "#).unwrap();
    
    // Create venv directory structure
    let venv_dir = temp_dir.path().join("venv");
    fs::create_dir_all(&venv_dir).unwrap();
    let site_packages = venv_dir.join("lib").join("python3.13").join("site-packages");
    fs::create_dir_all(&site_packages).unwrap();
    
    // Create charset_normalizer (the problematic dependency)
    let charset_dir = site_packages.join("charset_normalizer");
    fs::create_dir_all(&charset_dir).unwrap();
    let charset_file = charset_dir.join("models.py");
    fs::write(&charset_file, r#"
from typing import List, Tuple

CoherenceMatch = Tuple[str, float]
CoherenceMatches = List[CoherenceMatch]

class CharsetMatch:
    def __init__(
        self,
        payload: bytes,
        guessed_encoding: str,
        mean_mess_ratio: float,
        has_sig_or_bom: bool,
        languages: CoherenceMatches,
        decoded_payload: str | None = None,
        preemptive_declaration: str | None = None,
    ):
        self._payload: bytes = payload
        self._encoding: str = guessed_encoding
        self._mean_mess_ratio: float = mean_mess_ratio
        self._languages: CoherenceMatches = languages
        self._has_sig_or_bom: bool = has_sig_or_bom
        self._mean_coherence_ratio: float = 0.0
    "#).unwrap();
    
    temp_dir
}

#[test]
fn test_security_analyzer_skips_venv() {
    let temp_dir = create_python_project_with_venv();
    
    let analyzer = SecurityAnalyzer::new();
    let result = analyzer.analyze_repository(
        temp_dir.path(),
        None,
        None,
    ).unwrap();
    
    // Should not detect services from venv files
    let venv_detections: Vec<_> = result.entities.iter()
        .filter(|e| e.file_path.contains("venv") || 
               e.file_path.contains("site-packages") ||
               e.file_path.contains("charset_normalizer"))
        .collect();
    
    assert_eq!(venv_detections.len(), 0, 
        "Security analyzer should not detect entities from venv directories");
}

#[test]
fn test_service_detector_skips_venv_integration() {
    let temp_dir = create_python_project_with_venv();
    
    let detector = ServiceDetector::new();
    let services = detector.detect_services(temp_dir.path()).unwrap();
    
    // Should detect Cohere and OpenAI from real code
    let cohere_services: Vec<_> = services.iter()
        .filter(|s| s.provider == wavelength_arch_decoder::security::ServiceProvider::Cohere)
        .collect();
    
    let openai_services: Vec<_> = services.iter()
        .filter(|s| s.provider == wavelength_arch_decoder::security::ServiceProvider::OpenAI)
        .collect();
    
    // Should have detections from real code
    assert!(cohere_services.len() > 0, "Should detect Cohere from real code");
    assert!(openai_services.len() > 0, "Should detect OpenAI from real code");
    
    // Should NOT have detections from venv
    let venv_cohere = cohere_services.iter()
        .any(|s| s.file_path.contains("venv") || s.file_path.contains("site-packages"));
    let venv_openai = openai_services.iter()
        .any(|s| s.file_path.contains("venv") || s.file_path.contains("site-packages"));
    
    assert!(!venv_cohere, "Should not detect Cohere from venv");
    assert!(!venv_openai, "Should not detect OpenAI from venv");
}

#[test]
fn test_api_key_detector_false_positive_reduction() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a file with false positive patterns
    let config_file = temp_dir.path().join("config.js");
    fs::write(&config_file, r#"
// Firebase configuration - should NOT trigger false positives
const firebaseConfig = {
    apiKey: '${firebaseConfig.apiKey}',  // Template literal - NOT a hardcoded key
    authDomain: 'example.firebaseapp.com',
    projectId: 'my-project',
};

// Type definition - should NOT trigger
interface Config {
    apiKey: string;
    secret: string;
}

// Real hardcoded key - SHOULD trigger (test pattern - not real key)
// Using example prefix to avoid GitHub secret scanning false positives
const realConfig = {
    apiKey: 'example_live_key_1234567890123456789012345678901234567890',
};
    "#).unwrap();
    
    let detector = ApiKeyDetector::new();
    let result = detector.detect_api_keys(
        temp_dir.path(),
        None,
        None,
    ).unwrap();
    
    // Should not detect false positives
    let false_positives: Vec<_> = result.0.iter()
        .filter(|e| e.context.contains("firebaseConfig.apiKey") ||
               e.context.contains("interface Config") ||
               e.context.contains("${"))
        .collect();
    
    assert_eq!(false_positives.len(), 0, 
        "Should not detect false positives from template literals or type definitions");
    
    // Should detect real API key
    let real_keys: Vec<_> = result.0.iter()
        .filter(|e| e.context.contains("sk_live_"))
        .collect();
    
    assert!(real_keys.len() > 0, "Should detect real hardcoded API keys");
}

#[test]
fn test_comprehensive_venv_filtering() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple venv structures
    let venv_variants = vec![
        ("venv", "venv"),
        (".venv", ".venv"),
        ("env", "env"),
    ];
    
    for (venv_name, _) in venv_variants {
        let venv_dir = temp_dir.path().join(venv_name);
        fs::create_dir_all(&venv_dir).unwrap();
        
        let site_packages = venv_dir.join("lib").join("python3.13").join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();
        
        // Create a file with "cohere" in it
        let test_file = site_packages.join("test_module.py");
        fs::write(&test_file, r#"
# This module has coherence calculations
def calculate_coherence(text):
    return 0.5
        "#).unwrap();
    }
    
    // Create real code
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let app_file = src_dir.join("app.py");
    fs::write(&app_file, r#"
import cohere
client = cohere.Client()
    "#).unwrap();
    
    let detector = ServiceDetector::new();
    let services = detector.detect_services(temp_dir.path()).unwrap();
    
    // Should not detect from any venv variant
    let venv_detections: Vec<_> = services.iter()
        .filter(|s| s.provider == wavelength_arch_decoder::security::ServiceProvider::Cohere &&
               (s.file_path.contains("venv") || 
                s.file_path.contains(".venv") ||
                s.file_path.contains("/env/") ||
                s.file_path.contains("site-packages")))
        .collect();
    
    assert_eq!(venv_detections.len(), 0, 
        "Should not detect Cohere from any venv variant");
    
    // Should detect from real code
    let real_detections: Vec<_> = services.iter()
        .filter(|s| s.provider == wavelength_arch_decoder::security::ServiceProvider::Cohere &&
               s.file_path.contains("app.py"))
        .collect();
    
    assert!(real_detections.len() > 0, "Should detect Cohere from real code");
}

