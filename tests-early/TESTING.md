# Test instructions and examples

## Quick Start

1. **Start the server:**
   ```bash
   cargo run
   ```

2. **In another terminal, run the test script:**
   ```bash
   python3 test_api.py
   ```

## Manual Testing with curl

### 1. Health Check
```bash
curl http://localhost:8080/health
```

### 2. Register a User
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpassword123"
  }'
```

### 3. Login
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpassword123"
  }'
```

### 4. Create API Key (requires admin scope)
```bash
curl -X POST http://localhost:8080/api/v1/auth/keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY_HERE" \
  -d '{
    "name": "my-key",
    "scopes": ["read", "write", "admin"],
    "expires_in_days": 30
  }'
```

## Expected Responses

### Health Check
```json
{
  "status": "ok",
  "service": "wavelength-arch-decoder"
}
```

### Registration
```json
{
  "api_key": "wl_live_...",
  "message": "User registered successfully"
}
```

### Login
```json
{
  "api_key": "wl_live_...",
  "refresh_token": "...",
  "expires_at": "2025-..."
}
```

