# Repository Detail Analyze Test

This Playwright test navigates to a repository detail page, clicks the analyze button, and tracks the progress of the analysis in real-time.

## Usage

### Basic Usage (with default repository ID)
```bash
node scripts/test-repository-detail-analyze.js
```

### With a specific repository ID
```bash
node scripts/test-repository-detail-analyze.js --repo-id=5845dcce-dbf9-4adc-a349-ce4245ff2e48
```

### Run in visible mode (not headless)
```bash
node scripts/test-repository-detail-analyze.js --headless=false
```

### Using npm script
```bash
cd scripts
npm run test:repository-detail
npm run test:repository-detail:visible  # Visible browser
```

## What the Test Does

1. **Navigates** to the repository detail page: `/#repository-detail?repo=<id>&tab=overview`
2. **Clicks** the analyze button (`#btn-analyze-detail`)
3. **Tracks progress** by polling the progress status every 2 seconds
4. **Captures screenshots** at key moments and every 10 seconds during progress
5. **Logs progress updates** showing:
   - Step number (e.g., "Step 8/11")
   - Progress percentage
   - Status messages (e.g., "Storing code elements: 1000/2045 (48%)...")
6. **Verifies completion** when progress reaches 100% or analysis completes

## Output

The test generates:
- **Console logs** showing real-time progress updates
- **Screenshots** saved to `scripts/screenshots/repository-detail/`
- **Progress history** showing all captured progress updates
- **Summary** with test results and statistics

## Configuration

- **Base URL**: Set via `BASE_URL` environment variable (default: `http://localhost:8080`)
- **Default Repository ID**: `5845dcce-dbf9-4adc-a349-ce4245ff2e48`
- **Max Wait Time**: 10 minutes (600 seconds)
- **Polling Interval**: 2 seconds
- **Screenshot Interval**: Every 10 seconds during progress

## Example Output

```
🚀 Starting Repository Detail Analyze Test
============================================================
📍 Base URL: http://localhost:8080
🆔 Repository ID: 5845dcce-dbf9-4adc-a349-ce4245ff2e48
👁️  Headless: true
📸 Screenshots: scripts/screenshots/repository-detail
============================================================

🌐 Navigating to repository detail page...
   URL: http://localhost:8080/#repository-detail?repo=5845dcce-dbf9-4adc-a349-ce4245ff2e48&tab=overview
✅ Repository detail page loaded
📸 Screenshot: 01-page-loaded -> scripts/screenshots/repository-detail/...

🔘 Clicking analyze button...
   Initial button state: "Analyze" (disabled: false)
   After click: "Analyzing..." (disabled: true)
✅ Analyze button clicked and state updated

📊 Tracking analysis progress...
   [1] 2025-11-17T... (2s elapsed)
       Step: Step 1/11 - Fetching repository information
       Progress: 9%
       Status: Loading repository details...

   [2] 2025-11-17T... (4s elapsed)
       Step: Step 2/11 - Initializing crawler
       Progress: 18%
       Status: Setting up repository crawler...

   ...

✅ Analysis completed!

============================================================
📊 TEST SUMMARY
============================================================
✅ Repository ID: 5845dcce-dbf9-4adc-a349-ce4245ff2e48
✅ Progress updates captured: 45
⏱️  Duration: 245s
📸 Screenshots saved: 12
✅ Status: Analysis completed successfully
```

## Troubleshooting

- **Progress not appearing**: Make sure the server is running and the repository exists
- **Timeout**: Large repositories may take longer than 10 minutes - increase `maxWaitTime` in the script
- **Screenshots not saving**: Check that the `scripts/screenshots/repository-detail/` directory is writable

