pub mod service_detector;
pub mod analyzer;
pub mod api_key_detector;
pub mod templates;
pub mod types;
pub mod helpers;
pub mod vulnerabilities;
pub mod terraform;
pub mod cloudformation;
pub mod serverless;
pub mod firebase;
pub mod env_config;
pub mod security_config;
pub mod pattern_config;
pub mod generic_provider;

pub use service_detector::{ServiceDetector, DetectedService, ServiceProvider, ServiceType};
pub use analyzer::SecurityAnalyzer;
pub use types::SecurityAnalysis;
pub use types::{SecurityEntity, SecurityEntityType, SecurityRelationship, SecurityVulnerability, VulnerabilitySeverity};
pub use api_key_detector::ApiKeyDetector;

