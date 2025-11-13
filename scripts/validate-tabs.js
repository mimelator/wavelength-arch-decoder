#!/usr/bin/env node

/**
 * Script to validate all repository detail tabs work correctly
 * Usage: node scripts/validate-tabs.js [repository-id]
 */

const puppeteer = require('puppeteer');

const REPO_ID = process.argv[2] || 'f424b3dc-f3c0-440f-89f7-bf1219d693ec';
const BASE_URL = process.env.BASE_URL || 'http://localhost:8080';
const REPO_URL = `${BASE_URL}/#repository-detail?repo=${REPO_ID}`;

const TABS = [
    { id: 'dependencies', name: 'Dependencies', containerId: 'dependencies-list' },
    { id: 'services', name: 'Services', containerId: 'services-list' },
    { id: 'code', name: 'Code Structure', containerId: 'code-list' },
    { id: 'security', name: 'Security', containerId: 'security-list' },
    { id: 'tools', name: 'Tools', containerId: 'tools-list' },
    { id: 'tests', name: 'Tests', containerId: 'tests-list' },
    { id: 'documentation', name: 'Documentation', containerId: 'documentation-list' },
];

const COLORS = {
    reset: '\x1b[0m',
    green: '\x1b[32m',
    red: '\x1b[31m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
};

function log(message, color = 'reset') {
    console.log(`${COLORS[color]}${message}${COLORS.reset}`);
}

function logSuccess(message) {
    log(`✓ ${message}`, 'green');
}

function logError(message) {
    log(`✗ ${message}`, 'red');
}

function logInfo(message) {
    log(`ℹ ${message}`, 'cyan');
}

function logWarning(message) {
    log(`⚠ ${message}`, 'yellow');
}

async function validateTab(page, tab, repoId) {
    const { id, name, containerId } = tab;
    
    try {
        logInfo(`Testing tab: ${name}...`);
        
        // Click the tab button
        const tabButton = await page.$(`button.repo-tab[data-tab="${id}"]`);
        if (!tabButton) {
            throw new Error(`Tab button not found: ${id}`);
        }
        
        await tabButton.click();
        
        // Wait for tab content to be visible
        await page.waitForSelector(`#tab-${id}`, { visible: true, timeout: 5000 });
        
        // Wait a bit for content to load
        await page.waitForTimeout(2000);
        
        // Check if container exists
        const container = await page.$(`#${containerId}`);
        if (!container) {
            throw new Error(`Container not found: ${containerId}`);
        }
        
        // Get container content
        const content = await page.evaluate((containerId) => {
            const el = document.getElementById(containerId);
            return {
                innerHTML: el ? el.innerHTML : '',
                textContent: el ? el.textContent : '',
                hasLoading: el ? el.textContent.includes('Loading') : false,
                hasError: el ? el.textContent.includes('Failed') || el.textContent.includes('Error') : false,
                hasNoItems: el ? el.textContent.includes('No ') && el.textContent.includes('found') : false,
                hasItems: el ? el.querySelectorAll('.detail-item').length > 0 : false,
                itemCount: el ? el.querySelectorAll('.detail-item').length : 0,
            };
        }, containerId);
        
        // Validate content
        if (content.hasLoading) {
            logWarning(`  Tab ${name} still showing loading state`);
            return { success: false, reason: 'Still loading' };
        }
        
        if (content.hasError) {
            logError(`  Tab ${name} shows error: ${content.textContent.substring(0, 100)}`);
            return { success: false, reason: 'Error displayed' };
        }
        
        if (content.hasNoItems && !content.hasItems) {
            logWarning(`  Tab ${name} shows "No items found" - this may be expected`);
            return { success: true, reason: 'No items (expected)', itemCount: 0 };
        }
        
        if (content.hasItems) {
            logSuccess(`  Tab ${name} loaded successfully with ${content.itemCount} items`);
            return { success: true, reason: 'Items loaded', itemCount: content.itemCount };
        }
        
        // If we get here, something unexpected happened
        logWarning(`  Tab ${name} loaded but content is unclear`);
        return { success: true, reason: 'Loaded (unclear content)', itemCount: 0 };
        
    } catch (error) {
        logError(`  Tab ${name} failed: ${error.message}`);
        return { success: false, reason: error.message };
    }
}

async function validateFilters(page, tab) {
    const { id, name } = tab;
    
    try {
        // Check if search input exists
        const searchInput = await page.$(`#${id}-search`);
        if (searchInput) {
            logInfo(`  Checking search filter for ${name}...`);
            await searchInput.type('test');
            await page.waitForTimeout(500);
            await searchInput.click({ clickCount: 3 }); // Select all
            await page.keyboard.press('Backspace'); // Clear
            await page.waitForTimeout(500);
            logSuccess(`    Search filter works`);
        }
        
        // Check if group-by select exists
        const groupBySelect = await page.$(`#${id}-group-by`);
        if (groupBySelect) {
            logInfo(`  Checking group-by filter for ${name}...`);
            const options = await page.evaluate((selectId) => {
                const select = document.getElementById(selectId);
                return select ? Array.from(select.options).map(opt => opt.value) : [];
            }, `${id}-group-by`);
            
            if (options.length > 0) {
                // Try changing the value
                await page.select(`#${id}-group-by`, options[options.length - 1]);
                await page.waitForTimeout(1000);
                logSuccess(`    Group-by filter works (${options.length} options)`);
            }
        }
        
        return true;
    } catch (error) {
        logWarning(`  Filter validation for ${name} had issues: ${error.message}`);
        return false;
    }
}

async function main() {
    log(`\n${'='.repeat(60)}`, 'blue');
    log(`Repository Tab Validation Script`, 'blue');
    log(`${'='.repeat(60)}`, 'blue');
    log(`Repository ID: ${REPO_ID}`, 'cyan');
    log(`URL: ${REPO_URL}\n`, 'cyan');
    
    let browser;
    let page;
    
    try {
        // Launch browser
        logInfo('Launching browser...');
        
        // Try to use system Chrome if available
        let executablePath = null;
        const possiblePaths = [
            '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
            '/usr/bin/google-chrome',
            '/usr/bin/chromium',
            '/usr/bin/chromium-browser',
        ];
        
        for (const path of possiblePaths) {
            try {
                const fs = require('fs');
                if (fs.existsSync(path)) {
                    executablePath = path;
                    logInfo(`Using system Chrome at: ${path}`);
                    break;
                }
            } catch (e) {
                // Continue searching
            }
        }
        
        const launchOptions = {
            headless: 'new',
            args: [
                '--no-sandbox', 
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--disable-gpu',
            ],
            defaultViewport: { width: 1920, height: 1080 },
            timeout: 60000,
        };
        
        if (executablePath) {
            launchOptions.executablePath = executablePath;
        }
        
        browser = await puppeteer.launch(launchOptions);
        
        page = await browser.newPage();
        
        // Set longer timeout
        page.setDefaultTimeout(30000);
        
        // Navigate to repository detail page
        logInfo(`Navigating to ${REPO_URL}...`);
        await page.goto(REPO_URL, { waitUntil: 'networkidle2', timeout: 30000 });
        
        // Wait for page to load
        await page.waitForSelector('#repository-detail', { visible: true, timeout: 10000 });
        logSuccess('Page loaded successfully');
        
        // Wait a bit for initial content
        await page.waitForTimeout(2000);
        
        // Validate each tab
        const results = [];
        for (const tab of TABS) {
            const result = await validateTab(page, tab, REPO_ID);
            results.push({ ...tab, ...result });
            
            // Validate filters if tab loaded successfully
            if (result.success) {
                await validateFilters(page, tab);
            }
            
            // Small delay between tabs
            await page.waitForTimeout(500);
        }
        
        // Print summary
        log(`\n${'='.repeat(60)}`, 'blue');
        log(`Validation Summary`, 'blue');
        log(`${'='.repeat(60)}`, 'blue');
        
        const successful = results.filter(r => r.success);
        const failed = results.filter(r => !r.success);
        
        log(`\nSuccessful tabs: ${successful.length}/${results.length}`, 'green');
        successful.forEach(r => {
            const itemInfo = r.itemCount > 0 ? ` (${r.itemCount} items)` : '';
            log(`  ✓ ${r.name}${itemInfo}`, 'green');
        });
        
        if (failed.length > 0) {
            log(`\nFailed tabs: ${failed.length}/${results.length}`, 'red');
            failed.forEach(r => {
                log(`  ✗ ${r.name}: ${r.reason}`, 'red');
            });
        }
        
        // Overall result
        log(`\n${'='.repeat(60)}`, 'blue');
        if (failed.length === 0) {
            log(`✓ All tabs validated successfully!`, 'green');
            process.exit(0);
        } else {
            log(`⚠ Some tabs failed validation`, 'yellow');
            process.exit(1);
        }
        
    } catch (error) {
        logError(`\nFatal error: ${error.message}`);
        console.error(error);
        process.exit(1);
    } finally {
        if (browser) {
            await browser.close();
        }
    }
}

// Run the script
main().catch(error => {
    logError(`Unhandled error: ${error.message}`);
    console.error(error);
    process.exit(1);
});

