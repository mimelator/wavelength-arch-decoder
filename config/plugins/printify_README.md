# Printify Plugin Example

This plugin demonstrates how to add detection for **Printify**, a print-on-demand service that integrates with e-commerce platforms.

## What This Plugin Detects

### 1. Environment Variables
- `PRINTIFY_API_KEY` - API authentication key
- `PRINTIFY_SHOP_ID` - Shop identifier
- `PRINTIFY_WEBHOOK` - Webhook configuration
- Any env var containing `PRINTIFY`

### 2. SDK Patterns
- `@printify/` - Official Printify npm packages
- `printify-api` - Printify API client libraries
- `printify` - Generic Printify packages

### 3. API Endpoints
- `api.printify.com` - Main API endpoint
- `printify.com/api` - Alternative API endpoint

## How It Works

When you place this file in `config/plugins/printify.json`, the system will:

1. **Load the plugin** automatically when `ServiceDetector` is initialized with plugins
2. **Merge patterns** with the base configuration
3. **Detect Printify** in repositories through:
   - Environment variable files (`.env`, `.env.local`, etc.)
   - Package files (`package.json`, `requirements.txt`, etc.)
   - Code files containing API calls or SDK imports

## Example Detection Scenarios

### Scenario 1: Environment Variables
```bash
# .env file
PRINTIFY_API_KEY=sk_live_abc123
PRINTIFY_SHOP_ID=12345
```
**Result**: Detects "Printify API" service with high confidence (0.9)

### Scenario 2: SDK Usage
```javascript
// package.json
{
  "dependencies": {
    "@printify/printify-api": "^1.0.0"
  }
}
```
**Result**: Detects "Printify SDK" service from package.json

### Scenario 3: API Calls
```javascript
// api.js
fetch('https://api.printify.com/v1/products')
```
**Result**: Detects "Printify API" service from API endpoint

## Testing the Plugin

1. **Place the plugin file**:
   ```bash
   cp config/plugins/printify.json config/plugins/
   ```

2. **Initialize detector with plugins**:
   ```rust
   let plugin_dir = Path::new("config/plugins");
   let detector = ServiceDetector::with_plugins(Some(plugin_dir))?;
   ```

3. **Analyze a repository** that uses Printify:
   ```rust
   let services = detector.detect_services(repo_path)?;
   // Look for services with name containing "Printify"
   ```

## Customizing for Your Service

To create your own plugin:

1. **Copy this file** and rename it (e.g., `my_service.json`)
2. **Update the patterns**:
   - Change `PRINTIFY` to your service name
   - Update API endpoints
   - Adjust confidence scores based on specificity
3. **Add provider enum** (if not using `Unknown`):
   - Add to `ServiceProvider` enum in `src/security/service_detector.rs`
   - Add to `parse_provider()` method
4. **Test** with a repository that uses your service

## Notes

- **Provider**: Currently set to `Unknown` since Printify isn't in the base enum. To add it:
  1. Add `Printify` to `ServiceProvider` enum
  2. Add mapping in `parse_provider()` method
  3. Update plugin to use `"provider": "Printify"`
  
- **Confidence Scores**:
  - `0.9`: Very specific patterns (exact API keys, official SDKs)
  - `0.85`: Specific but could have false positives
  - `0.7-0.8`: Generic patterns (service name in env var)

- **Service Type**: Set to `Api` since Printify is primarily an API service. Other options:
  - `CloudProvider` - For infrastructure services
  - `SaaS` - For software-as-a-service platforms
  - `Payment` - For payment processors
  - `Other` - For services that don't fit categories

