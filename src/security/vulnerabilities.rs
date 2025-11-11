use std::path::Path;
use crate::security::types::{SecurityVulnerability, VulnerabilitySeverity};
use crate::security::templates;

/// Check IAM role for vulnerabilities
pub fn check_iam_role_vulnerabilities(
    config: &str,
    entity_id: &str,
    _path: &Path,
    normalized_path: &str,
    line_num: usize,
) -> Option<SecurityVulnerability> {
    // Check for overly permissive assume role policies
    if config.contains("*") && config.contains("Effect") && config.contains("Allow") {
        return Some(SecurityVulnerability {
            id: format!("{}:vuln:1", entity_id),
            entity_id: entity_id.to_string(),
            vulnerability_type: "OverlyPermissiveAssumeRolePolicy".to_string(),
            severity: VulnerabilitySeverity::High,
            description: templates::DESC_ASSUME_ROLE_POLICY.to_string(),
            recommendation: templates::REC_ASSUME_ROLE_POLICY.to_string(),
            file_path: normalized_path.to_string(),
            line_number: Some(line_num + 1),
        });
    }
    None
}

/// Check IAM policy for vulnerabilities
pub fn check_iam_policy_vulnerabilities(
    config: &str,
    entity_id: &str,
    _path: &Path,
    normalized_path: &str,
    line_num: usize,
) -> Option<SecurityVulnerability> {
    // Check for wildcard actions
    if config.contains(r#""Action": "*""#) || config.contains("Action: *") {
        return Some(SecurityVulnerability {
            id: format!("{}:vuln:1", entity_id),
            entity_id: entity_id.to_string(),
            vulnerability_type: "WildcardAction".to_string(),
            severity: VulnerabilitySeverity::Critical,
            description: templates::DESC_WILDCARD_ACTION.to_string(),
            recommendation: templates::REC_WILDCARD_ACTION.to_string(),
            file_path: normalized_path.to_string(),
            line_number: Some(line_num + 1),
        });
    }

    // Check for wildcard resources
    if config.contains(r#""Resource": "*""#) || config.contains("Resource: *") {
        return Some(SecurityVulnerability {
            id: format!("{}:vuln:2", entity_id),
            entity_id: entity_id.to_string(),
            vulnerability_type: "WildcardResource".to_string(),
            severity: VulnerabilitySeverity::Critical,
            description: templates::DESC_WILDCARD_RESOURCE.to_string(),
            recommendation: templates::REC_WILDCARD_RESOURCE.to_string(),
            file_path: normalized_path.to_string(),
            line_number: Some(line_num + 1),
        });
    }
    None
}

/// Check S3 bucket for vulnerabilities
pub fn check_s3_bucket_vulnerabilities(
    config: &str,
    entity_id: &str,
    _path: &Path,
    normalized_path: &str,
    line_num: usize,
) -> Option<SecurityVulnerability> {
    // Check for public access
    if config.contains("public_access_block") && 
       (config.contains(templates::BLOCK_PUBLIC_ACLS) || 
        config.contains(templates::BLOCK_PUBLIC_POLICY) ||
        config.contains(templates::IGNORE_PUBLIC_ACLS) ||
        config.contains(templates::RESTRICT_PUBLIC_BUCKETS)) {
        return Some(SecurityVulnerability {
            id: format!("{}:vuln:1", entity_id),
            entity_id: entity_id.to_string(),
            vulnerability_type: "PublicS3Bucket".to_string(),
            severity: VulnerabilitySeverity::Critical,
            description: templates::DESC_S3_PUBLIC_ACCESS.to_string(),
            recommendation: templates::REC_S3_PUBLIC_ACCESS.to_string(),
            file_path: normalized_path.to_string(),
            line_number: Some(line_num + 1),
        });
    }

    // Check for missing encryption
    if !config.contains("server_side_encryption") && !config.contains("encryption") {
        return Some(SecurityVulnerability {
            id: format!("{}:vuln:2", entity_id),
            entity_id: entity_id.to_string(),
            vulnerability_type: "UnencryptedS3Bucket".to_string(),
            severity: VulnerabilitySeverity::Medium,
            description: templates::DESC_S3_UNENCRYPTED.to_string(),
            recommendation: templates::REC_S3_UNENCRYPTED.to_string(),
            file_path: normalized_path.to_string(),
            line_number: Some(line_num + 1),
        });
    }

    None
}

/// Check security group for vulnerabilities
pub fn check_security_group_vulnerabilities(
    config: &str,
    entity_id: &str,
    _path: &Path,
    normalized_path: &str,
    line_num: usize,
) -> Option<SecurityVulnerability> {
    // Check for open ingress rules (0.0.0.0/0)
    if config.contains("0.0.0.0/0") || config.contains("::/0") {
        return Some(SecurityVulnerability {
            id: format!("{}:vuln:1", entity_id),
            entity_id: entity_id.to_string(),
            vulnerability_type: "OpenSecurityGroup".to_string(),
            severity: VulnerabilitySeverity::High,
            description: templates::DESC_SECURITY_GROUP_OPEN.to_string(),
            recommendation: templates::REC_SECURITY_GROUP_OPEN.to_string(),
            file_path: normalized_path.to_string(),
            line_number: Some(line_num + 1),
        });
    }

    None
}

