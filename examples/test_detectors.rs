// Standalone test script for port and endpoint detection
use wavelength_arch_decoder::analysis::{PortDetector, EndpointDetector, HttpMethod};
use std::path::Path;

fn main() {
    println!("🔍 Testing Port and Endpoint Detection\n");
    println!("=====================================\n");

    // Test on test_repo first, then try spring-petclinic
    let test_repo_path = if Path::new("test_repo").exists() {
        Path::new("test_repo")
    } else if Path::new("cache/repos/spring-petclinic").exists() {
        Path::new("cache/repos/spring-petclinic")
    } else {
        eprintln!("Error: No test repository found!");
        eprintln!("Please ensure test_repo/ or cache/repos/spring-petclinic/ exists.");
        std::process::exit(1);
    };
    
    if !test_repo_path.exists() {
        eprintln!("Error: test_repo directory not found!");
        eprintln!("Please ensure test_repo/ exists with sample files.");
        std::process::exit(1);
    }

    // Test Port Detection
    println!("🔌 Testing Port Detection...");
    println!("----------------------------");
    let port_detector = PortDetector::new();
    match port_detector.detect_ports(test_repo_path) {
        Ok(ports) => {
            println!("✓ Found {} port(s):\n", ports.len());
            for port in &ports {
                println!("  Port: {}", port.port);
                println!("    Type: {:?}", port.port_type);
                println!("    Framework: {:?}", port.framework);
                println!("    Environment: {:?}", port.environment);
                println!("    Is Config: {}", port.is_config);
                println!("    File: {}", port.file_path);
                println!("    Context: {}", 
                    if port.context.len() > 60 { 
                        format!("{}...", &port.context[..60]) 
                    } else { 
                        port.context.clone() 
                    });
                println!();
            }
        }
        Err(e) => {
            eprintln!("✗ Port detection failed: {}", e);
        }
    }

    println!("\n🌐 Testing Endpoint Detection...");
    println!("-------------------------------");
    let endpoint_detector = EndpointDetector::new();
    match endpoint_detector.detect_endpoints(test_repo_path) {
        Ok(endpoints) => {
            println!("✓ Found {} endpoint(s):\n", endpoints.len());
            for endpoint in &endpoints {
                println!("  {:?} {}", endpoint.method, endpoint.path);
                println!("    Handler: {:?}", endpoint.handler);
                println!("    Framework: {:?}", endpoint.framework);
                println!("    Parameters: {:?}", endpoint.parameters);
                println!("    File: {}", endpoint.file_path);
                if let Some(line) = endpoint.line_number {
                    println!("    Line: {}", line);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("✗ Endpoint detection failed: {}", e);
        }
    }

    println!("\n✅ Test complete!");
}

