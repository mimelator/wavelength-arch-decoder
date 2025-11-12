#[cfg(test)]
mod printify_plugin_test {
    use super::*;
    use crate::security::ServiceDetector;
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_printify_detection_from_env_file() {
        // Create a temporary directory with a .env file containing Printify config
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");
        
        fs::write(&env_file, r#"
# Printify Configuration
PRINTIFY_API_KEY=sk_live_test123
PRINTIFY_SHOP_ID=24952672
PRINTIFY_ENVIRONMENT=sandbox
        "#).unwrap();

        // Load detector with plugins
        let plugin_dir = Path::new("config/plugins");
        let detector = if plugin_dir.exists() {
            ServiceDetector::with_plugins(Some(plugin_dir)).unwrap_or_else(|_| ServiceDetector::new())
        } else {
            ServiceDetector::new()
        };

        // Detect services
        let services = detector.detect_services(temp_dir.path()).unwrap();
        
        // Check if Printify was detected
        let printify_services: Vec<_> = services.iter()
            .filter(|s| s.name.to_lowercase().contains("printify"))
            .collect();
        
        assert!(!printify_services.is_empty(), "Printify should be detected from .env file");
        
        // Check for specific Printify API detection
        let api_key_detection = printify_services.iter()
            .find(|s| s.name.contains("API") && s.configuration.contains_key("env_var"));
        
        assert!(api_key_detection.is_some(), "PRINTIFY_API_KEY should be detected");
        
        println!("✓ Detected {} Printify service(s):", printify_services.len());
        for service in &printify_services {
            println!("  - {} (confidence: {})", service.name, service.confidence);
        }
    }

    #[test]
    fn test_printify_detection_from_api_endpoint() {
        // Create a temporary directory with a code file containing Printify API call
        let temp_dir = TempDir::new().unwrap();
        let code_file = temp_dir.path().join("printify-service.js");
        
        fs::write(&code_file, r#"
async function fetchPrintifyProducts() {
    const response = await fetch('https://api.printify.com/v1/products');
    return response.json();
}
        "#).unwrap();

        // Load detector with plugins
        let plugin_dir = Path::new("config/plugins");
        let detector = if plugin_dir.exists() {
            ServiceDetector::with_plugins(Some(plugin_dir)).unwrap_or_else(|_| ServiceDetector::new())
        } else {
            ServiceDetector::new()
        };

        // Detect services
        let services = detector.detect_services(temp_dir.path()).unwrap();
        
        // Check if Printify API endpoint was detected
        let printify_services: Vec<_> = services.iter()
            .filter(|s| s.name.to_lowercase().contains("printify"))
            .collect();
        
        assert!(!printify_services.is_empty(), "Printify should be detected from API endpoint");
        
        println!("✓ Detected {} Printify service(s) from API endpoint:", printify_services.len());
        for service in &printify_services {
            println!("  - {} (confidence: {})", service.name, service.confidence);
        }
    }
}

