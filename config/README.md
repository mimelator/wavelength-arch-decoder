# Service Detection Configuration

The Wavelength Architecture Decoder uses a configurable pattern system for service detection. This allows you to:

1. **Customize detection patterns** without modifying code
2. **Add new service providers** via configuration files
3. **Use plugins** to extend detection capabilities

## Configuration File

The main configuration file is located at `config/service_patterns.json`. This file contains all detection patterns organized by type:

- `environment_variables`: Patterns for detecting services from environment variable names
- `sdk_patterns`: Patterns for detecting service SDKs in code
- `api_endpoints`: Patterns for detecting API endpoint URLs
- `database_patterns`: Patterns for detecting database connection strings
- `aws_infrastructure`: Patterns for detecting AWS services in Terraform/CloudFormation
- `aws_sdk_v2_services`: Patterns for detecting specific AWS services in SDK v2 code
- `aws_sdk_v3_service_map`: Mapping of AWS SDK v3 client names to display names

## Pattern Format

Each pattern rule has the following structure:

```json
{
  "pattern": "SERVICE_NAME",
  "provider": "ProviderName",
  "service_type": "ServiceType",
  "confidence": 0.7,
  "service_name": "Optional Display Name"
}
```

### Fields

- `pattern`: The string pattern to search for (case-insensitive for most patterns)
- `provider`: The provider name (must match a `ServiceProvider` enum value)
- `service_type`: The service type (must match a `ServiceType` enum value)
- `confidence`: Confidence score (0.0 to 1.0)
- `service_name`: Optional custom display name (defaults to pattern-based name)

### Provider Values

Valid provider values include:
- Cloud Providers: `Aws`, `Azure`, `Gcp`, `Vercel`, `Netlify`, `Heroku`, `DigitalOcean`
- SaaS Services: `Clerk`, `Auth0`, `Stripe`, `Twilio`, `SendGrid`, `Mailgun`, `Slack`, `Discord`
- Databases: `Postgres`, `MySQL`, `MongoDB`, `Redis`, `DynamoDB`, `RDS`
- APIs: `GitHub`, `GitLab`, `Jira`, `Linear`
- CDN: `Cloudflare`, `CloudFront`
- Monitoring: `Datadog`, `NewRelic`, `Sentry`, `LogRocket`
- AI Services: `OpenAI`, `Anthropic`, `GitHubCopilot`, `GoogleAI`, `Cohere`, `HuggingFace`, `Replicate`, `TogetherAI`, `MistralAI`, `Perplexity`
- Other: `Unknown`

### Service Type Values

Valid service type values:
- `CloudProvider`
- `SaaS`
- `Database`
- `Api`
- `Cdn`
- `Monitoring`
- `Auth`
- `Payment`
- `AI`
- `Other`

## Plugin System

You can extend the detection system by adding custom pattern files to the `config/plugins/` directory. Any JSON file in this directory will be automatically loaded and merged with the base configuration.

### Creating a Plugin

1. Create a JSON file in `config/plugins/` (e.g., `my_custom_services.json`)
2. Use the same structure as the main config file
3. Add your custom patterns
4. The plugin patterns will be merged with the base patterns

Example plugin file (`config/plugins/example_plugin.json`):

```json
{
  "version": "1.0",
  "patterns": {
    "environment_variables": [
      {
        "pattern": "MY_CUSTOM_SERVICE",
        "provider": "Unknown",
        "service_type": "Other",
        "confidence": 0.6
      }
    ],
    "sdk_patterns": [
      {
        "pattern": "@my-custom-service/",
        "provider": "Unknown",
        "service_type": "Other",
        "confidence": 0.7
      }
    ]
  }
}
```

## Generic Provider Detection

In addition to pattern-based detection, the system also includes generic provider detection that analyzes package files (`package.json`, `requirements.txt`, `Cargo.toml`, `go.mod`) to identify service SDKs. This can catch services that aren't explicitly configured in patterns.

## Usage

The `ServiceDetector` automatically loads patterns from the config file:

```rust
let detector = ServiceDetector::new();
let services = detector.detect_services(repo_path)?;
```

To use plugins:

```rust
let plugin_dir = Path::new("config/plugins");
let detector = ServiceDetector::with_plugins(Some(plugin_dir))?;
let services = detector.detect_services(repo_path)?;
```

## Adding New Providers

To add a new provider that isn't in the enum:

1. Add the provider to the `ServiceProvider` enum in `src/security/service_detector.rs`
2. Add it to the `parse_provider` method
3. Add detection patterns to `config/service_patterns.json` or a plugin file

## Best Practices

1. **Use specific patterns**: More specific patterns reduce false positives
2. **Set appropriate confidence**: Lower confidence for generic patterns, higher for specific ones
3. **Test patterns**: Test new patterns with real repositories before committing
4. **Document custom plugins**: Add comments or documentation for custom plugins
5. **Keep patterns updated**: Update patterns as services evolve

