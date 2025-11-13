# Validation Scripts

Scripts to validate that all repository detail tabs work correctly.

## Node.js Version (Puppeteer)

### Setup

```bash
cd scripts
npm install
```

### Usage

```bash
# Use default repository ID
npm run validate-tabs

# Or specify a repository ID
node validate-tabs.js f424b3dc-f3c0-440f-89f7-bf1219d693ec
```

## Python Version (Selenium)

### Setup

```bash
pip install selenium webdriver-manager
```

### Usage

```bash
# Use default repository ID
python3 scripts/validate-tabs.py

# Or specify a repository ID
python3 scripts/validate-tabs.py f424b3dc-f3c0-440f-89f7-bf1219d693ec
```

## What It Tests

The script validates:

1. **Tab Navigation**: Each tab button can be clicked
2. **Content Loading**: Each tab loads its content container
3. **Content Display**: Content is displayed (or shows appropriate "no items" message)
4. **Filter Functionality**: Search and group-by filters work
5. **Error Handling**: No errors are displayed

## Tabs Validated

- Dependencies
- Services
- Code Structure
- Security
- Tools
- Tests
- Documentation

## Environment Variables

- `BASE_URL`: Override the base URL (default: `http://localhost:8080`)

Example:
```bash
BASE_URL=http://localhost:3000 node validate-tabs.js
```

