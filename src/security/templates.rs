// Template constants module to avoid quote escaping issues
pub const QUOTE: char = '"';
pub const SINGLE_QUOTE: char = '\'';
pub const NEWLINE: char = '\n';
pub const SPACE: char = ' ';

// Helper function to create Terraform resource pattern using raw string literal
pub fn terraform_resource_pattern(resource_type: &str) -> String {
    format!(r#"resource "{}""#, resource_type)
}

// String constants for vulnerability descriptions to avoid quote issues
pub const DESC_ASSUME_ROLE_POLICY: &str = "IAM role has overly permissive assume role policy";
pub const REC_ASSUME_ROLE_POLICY: &str = "Restrict assume role policy to specific principals";
pub const DESC_WILDCARD_ACTION: &str = "IAM policy uses wildcard action (*)";
pub const REC_WILDCARD_ACTION: &str = "Replace wildcard actions with specific actions";
pub const DESC_WILDCARD_RESOURCE: &str = "IAM policy uses wildcard resource (*)";
pub const REC_WILDCARD_RESOURCE: &str = "Replace wildcard resources with specific ARNs";

// S3 bucket vulnerability check patterns
pub const BLOCK_PUBLIC_ACLS: &str = "block_public_acls = false";
pub const BLOCK_PUBLIC_POLICY: &str = "block_public_policy = false";
pub const IGNORE_PUBLIC_ACLS: &str = "ignore_public_acls = false";
pub const RESTRICT_PUBLIC_BUCKETS: &str = "restrict_public_buckets = false";

// Additional string constants for other vulnerability messages
pub const DESC_S3_PUBLIC_ACCESS: &str = "S3 bucket allows public access";
pub const REC_S3_PUBLIC_ACCESS: &str = "Enable public access block settings";
pub const DESC_S3_UNENCRYPTED: &str = "S3 bucket does not have encryption enabled";
pub const REC_S3_UNENCRYPTED: &str = "Enable server-side encryption for S3 bucket";
pub const DESC_SECURITY_GROUP_OPEN: &str = "Security group allows access from anywhere (0.0.0.0/0)";
pub const REC_SECURITY_GROUP_OPEN: &str = "Restrict security group rules to specific IP ranges";
pub const REC_FIREBASE_RULES: &str = "Restrict access rules to authenticated users and specific conditions";
pub const REC_FIREBASE_AUTH: &str = "Add authentication checks to access rules";
pub const REQUEST_AUTH: &str = "request.auth";
pub const DESC_FIREBASE_UNRESTRICTED: &str = "allows unrestricted read/write";
pub const DESC_FIREBASE_UNAUTH: &str = "rule may allow unauthenticated";
pub const WORD_ACCESS: &str = "access";
pub const CONFIG_TYPE_JSON: &str = "json";
pub const CONFIG_TYPE_YAML: &str = "yaml";
pub const PROVIDER_GENERIC: &str = "generic";

// CloudFormation resource types
pub const AWS_IAM_ROLE: &str = "AWS::IAM::Role";
pub const AWS_IAM_POLICY: &str = "AWS::IAM::Policy";
pub const AWS_LAMBDA_FUNCTION: &str = "AWS::Lambda::Function";
pub const AWS_S3_BUCKET: &str = "AWS::S3::Bucket";

// Firebase rules type strings
pub const FIRESTORE_RULES: &str = "Firestore Rules";
pub const STORAGE_RULES: &str = "Storage Rules";
pub const DATABASE_RULES: &str = "Database Rules";
pub const FIREBASE_RULES: &str = "Firebase Rules";

// Firebase rules patterns
pub const ALLOW_RW_TRUE: &str = "allow read, write: if true";
pub const ALLOW_RW_NULL: &str = "allow read, write: if request.auth == null";
pub const ALLOW_RW: &str = "allow read, write";
pub const IF_TRUE: &str = "if true";
pub const ALLOW_READ: &str = "allow read";
pub const ALLOW_WRITE: &str = "allow write";
pub const IF_FALSE: &str = "if false";

