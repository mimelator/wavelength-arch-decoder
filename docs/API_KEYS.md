# API Key Management Best Practices

## Overview

The Wavelength Architecture Decoder uses API keys for authentication and access control. This document outlines best practices for managing API keys securely.

## Obtaining API Keys

### Initial Registration

1. **Register an Account**
   ```bash
   POST /api/v1/auth/register
   {
     "email": "user@example.com",
     "password": "secure-password"
   }
   ```

2. **Login to Get API Key**
   ```bash
   POST /api/v1/auth/login
   {
     "email": "user@example.com",
     "password": "secure-password"
   }
   
   Response:
   {
     "api_key": "wl_live_abc123...",
     "refresh_token": "...",
     "expires_at": "2025-12-31T23:59:59Z"
   }
   ```

3. **Generate Additional Keys** (for different environments)
   ```bash
   POST /api/v1/auth/keys
   Authorization: Bearer <your-api-key>
   {
     "name": "production-key",
     "scopes": ["read", "write"],
     "expires_in_days": 90
   }
   ```

## API Key Scopes

API keys support scoped permissions:

- **`read`**: Query repositories and knowledge graphs (read-only)
- **`write`**: Add/update repositories and trigger analysis
- **`admin`**: Full access including key management and deletion

Example scoped key:
```json
{
  "scopes": ["read", "write"],
  "rate_limit": 1000,
  "allowed_ips": ["192.168.1.0/24"]
}
```

## Secure Storage

### ✅ DO

1. **Store in Environment Variables**
   ```bash
   export WAVELENGTH_API_KEY="wl_live_abc123..."
   ```

2. **Use Secret Management Tools**
   - AWS Secrets Manager
   - HashiCorp Vault
   - Kubernetes Secrets
   - CI/CD secret variables (GitHub Secrets, GitLab CI Variables)

3. **Use Configuration Files with Restricted Permissions**
   ```bash
   # .env file (never commit to git)
   WAVELENGTH_API_KEY=wl_live_abc123...
   chmod 600 .env
   ```

4. **Rotate Keys Regularly**
   - Production keys: Every 90 days
   - Development keys: Every 180 days
   - Compromised keys: Immediately

### ❌ DON'T

1. **Never Commit Keys to Version Control**
   ```bash
   # Add to .gitignore
   .env
   *.key
   config/secrets.json
   ```

2. **Never Hardcode in Source Code**
   ```rust
   // BAD
   let api_key = "wl_live_abc123...";
   
   // GOOD
   let api_key = env::var("WAVELENGTH_API_KEY")
       .expect("WAVELENGTH_API_KEY must be set");
   ```

3. **Never Share Keys in Chat/Email**
   - Use secure channels for key sharing
   - Consider using key sharing mechanisms with expiration

4. **Never Log Keys**
   ```rust
   // BAD
   println!("API Key: {}", api_key);
   
   // GOOD
   log::debug!("API Key: {}...{}", &api_key[..4], &api_key[api_key.len()-4..]);
   ```

## Using API Keys

### HTTP Headers

```bash
curl -H "Authorization: Bearer wl_live_abc123..." \
     https://api.wavelength.dev/api/v1/repositories
```

### Environment Variable

```bash
export WAVELENGTH_API_KEY="wl_live_abc123..."
curl -H "Authorization: Bearer $WAVELENGTH_API_KEY" \
     https://api.wavelength.dev/api/v1/repositories
```

### Configuration File

```toml
# config.toml
[api]
key = "${WAVELENGTH_API_KEY}"  # Reference env var
endpoint = "https://api.wavelength.dev"
```

## Key Rotation

### Rotating Keys Without Downtime

1. **Generate New Key**
   ```bash
   POST /api/v1/auth/keys
   {
     "name": "new-production-key",
     "scopes": ["read", "write"]
   }
   ```

2. **Update Applications Gradually**
   - Deploy new key to staging
   - Deploy new key to production (both keys work during transition)
   - Remove old key after all services updated

3. **Revoke Old Key**
   ```bash
   DELETE /api/v1/auth/keys/{old-key-id}
   ```

### Emergency Rotation

If a key is compromised:

1. **Immediately Revoke**
   ```bash
   DELETE /api/v1/auth/keys/{compromised-key-id}
   ```

2. **Generate Replacement**
   ```bash
   POST /api/v1/auth/keys
   {
     "name": "emergency-replacement",
     "scopes": ["read", "write", "admin"]
   }
   ```

3. **Update All Systems**
   - Update environment variables
   - Restart services
   - Verify functionality

## Rate Limiting

Each API key has rate limits:

- **Default**: 1,000 requests per hour
- **Read-only keys**: 5,000 requests per hour
- **Admin keys**: 10,000 requests per hour

### Checking Rate Limits

```bash
GET /api/v1/auth/keys/current
Authorization: Bearer <your-api-key>

Response:
{
  "key_id": "key_123",
  "rate_limit": 1000,
  "requests_remaining": 850,
  "reset_at": "2025-01-15T14:00:00Z"
}
```

### Handling Rate Limits

```rust
// Example: Exponential backoff
if response.status() == 429 {
    let retry_after = response.headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    
    thread::sleep(Duration::from_secs(retry_after));
    // Retry request
}
```

## Key Expiration

API keys can have expiration dates:

```bash
POST /api/v1/auth/keys
{
  "name": "temporary-key",
  "expires_in_days": 30,
  "scopes": ["read"]
}
```

### Monitoring Expiration

```bash
GET /api/v1/auth/keys

Response:
{
  "keys": [
    {
      "id": "key_123",
      "name": "production-key",
      "expires_at": "2025-03-15T00:00:00Z",
      "days_until_expiry": 45
    }
  ]
}
```

## IP Whitelisting

For enhanced security, restrict keys to specific IP ranges:

```bash
POST /api/v1/auth/keys
{
  "name": "restricted-key",
  "allowed_ips": [
    "192.168.1.0/24",
    "10.0.0.0/8"
  ],
  "scopes": ["read", "write"]
}
```

## Audit Logging

All API key usage is logged:

- Timestamp
- Key ID (not full key)
- Endpoint accessed
- IP address
- Request method
- Response status

Access logs:
```bash
GET /api/v1/auth/keys/{key-id}/logs
Authorization: Bearer <admin-key>

Response:
{
  "logs": [
    {
      "timestamp": "2025-01-15T10:30:00Z",
      "endpoint": "/api/v1/repositories",
      "method": "GET",
      "ip": "192.168.1.100",
      "status": 200
    }
  ]
}
```

## Best Practices Summary

1. ✅ Use environment variables or secret managers
2. ✅ Rotate keys regularly (90 days for production)
3. ✅ Use scoped permissions (principle of least privilege)
4. ✅ Monitor key usage and expiration
5. ✅ Implement rate limiting in your applications
6. ✅ Use IP whitelisting for production keys
7. ✅ Never commit keys to version control
8. ✅ Have an emergency rotation plan
9. ✅ Use different keys for different environments
10. ✅ Audit key usage regularly

## Security Incident Response

If you suspect a key is compromised:

1. **Immediately revoke the key** via API or admin panel
2. **Review audit logs** for unauthorized access
3. **Generate new key** with same or reduced scopes
4. **Update all systems** using the compromised key
5. **Monitor for suspicious activity** on new key
6. **Document the incident** for future reference

## Support

For API key issues or security concerns:
- Email: security@wavelength.dev
- Documentation: https://docs.wavelength.dev/api-keys
- Status Page: https://status.wavelength.dev

