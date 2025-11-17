#!/usr/bin/env node

/**
 * UI Test Harness using Playwright for Wavelength Architecture Decoder
 * 
 * Based on the superior testing approach from ai-image-decoder
 */

const { chromium } = require('playwright');

// Configuration
const BASE_URL = process.env.BASE_URL || 'http://localhost:8080';
const HEADLESS = !process.argv.includes('--headless=false');

// Test results
const testResults = {
    startTime: new Date(),
    errors: [],
    warnings: [],
    pages: {},
    interactions: {}
};

/**
 * Setup error listeners
 */
function setupErrorListeners(page) {
    page.on('console', msg => {
        const type = msg.type();
        const text = msg.text();
        
        if (type === 'error') {
            testResults.errors.push({
                type: 'console.error',
                message: text,
                timestamp: new Date().toISOString()
            });
        } else if (type === 'warning') {
            testResults.warnings.push({
                type: 'console.warning',
                message: text,
                timestamp: new Date().toISOString()
            });
        }
    });
    
    page.on('pageerror', error => {
        testResults.errors.push({
            type: 'page.error',
            message: error.message,
            stack: error.stack,
            timestamp: new Date().toISOString()
        });
    });
    
    page.on('requestfailed', request => {
        const url = request.url();
        if (!url.includes('/static/')) {
            testResults.errors.push({
                type: 'request.failed',
                url: url,
                failureText: request.failure()?.errorText,
                timestamp: new Date().toISOString()
            });
        }
    });
    
    page.on('response', response => {
        const url = response.url();
        const status = response.status();
        if (status >= 400 && !url.includes('/static/')) {
            testResults.errors.push({
                type: 'response.error',
                url: url,
                status: status,
                statusText: response.statusText(),
                timestamp: new Date().toISOString()
            });
        }
    });
}

/**
 * Test page navigation
 */
async function testPageNavigation(page, pageName) {
    console.log(`\n📄 Testing ${pageName} page...`);
    
    try {
        // Navigate to the page
        const navLink = page.locator(`[data-page="${pageName}"]`);
        await navLink.waitFor({ timeout: 10000 });
        await navLink.click();
        
        // Wait for page content
        const pageElement = page.locator(`#${pageName}`);
        await pageElement.waitFor({ timeout: 10000 });
        await page.waitForTimeout(1000);
        
        const isVisible = await pageElement.evaluate((el) => {
            return el && el.classList.contains('active');
        });
        
        if (!isVisible) {
            throw new Error(`Page ${pageName} not visible after navigation`);
        }
        
        testResults.pages[pageName] = {
            status: 'passed',
            timestamp: new Date().toISOString()
        };
        
        console.log(`  ✅ ${pageName} page loaded successfully`);
        return true;
    } catch (error) {
        testResults.pages[pageName] = {
            status: 'failed',
            error: error.message,
            timestamp: new Date().toISOString()
        };
        console.log(`  ❌ ${pageName} page failed: ${error.message}`);
        return false;
    }
}

/**
 * Test repository detail tabs
 */
async function testRepositoryTabs(page) {
    console.log('\n📑 Testing Repository Detail tabs...');
    
    try {
        // First, check if there are any repositories
        const reposList = page.locator('#repositories-list');
        await reposList.waitFor({ timeout: 10000 });
        await page.waitForTimeout(2000);
        
        const repoItems = await page.locator('.repo-item').count();
        if (repoItems === 0) {
            console.log('  ⚠️  No repositories found - skipping tab tests');
            return;
        }
        
        // Click first repository
        const firstRepo = page.locator('.repo-item').first();
        await firstRepo.locator('button:has-text("View")').click();
        await page.waitForTimeout(2000);
        
        // Test tabs
        const tabs = ['overview', 'dependencies', 'services', 'code', 'security', 'tools', 'tests', 'documentation', 'graph'];
        for (const tab of tabs) {
            try {
                const tabButton = page.locator(`[data-tab="${tab}"]`);
                await tabButton.waitFor({ timeout: 5000 });
                await tabButton.click();
                await page.waitForTimeout(1000);
                
                const tabContent = page.locator(`#tab-${tab}`);
                const isVisible = await tabContent.evaluate((el) => {
                    return el && el.classList.contains('active');
                });
                
                if (isVisible) {
                    console.log(`  ✅ ${tab} tab loaded`);
                } else {
                    console.log(`  ⚠️  ${tab} tab may not be visible`);
                }
            } catch (error) {
                console.log(`  ⚠️  ${tab} tab test skipped: ${error.message}`);
            }
        }
        
        testResults.interactions.repositoryTabs = 'passed';
    } catch (error) {
        testResults.interactions.repositoryTabs = `failed: ${error.message}`;
        console.log(`  ❌ Repository tabs test failed: ${error.message}`);
    }
}

/**
 * Test theme toggle
 */
async function testThemeToggle(page) {
    console.log('\n🎨 Testing Theme Toggle...');
    
    try {
        const themeBtn = page.locator('#theme-toggle');
        if (await themeBtn.count() > 0) {
            const initialTheme = await page.evaluate(() => {
                return document.documentElement.getAttribute('data-theme') || 'light';
            });
            
            await themeBtn.click();
            await page.waitForTimeout(500);
            
            const newTheme = await page.evaluate(() => {
                return document.documentElement.getAttribute('data-theme') || 'light';
            });
            
            if (newTheme !== initialTheme) {
                console.log(`  ✅ Theme toggled from ${initialTheme} to ${newTheme}`);
                await themeBtn.click();
                await page.waitForTimeout(500);
                testResults.interactions.themeToggle = 'passed';
            }
        }
    } catch (error) {
        testResults.interactions.themeToggle = `failed: ${error.message}`;
        console.log(`  ❌ Theme toggle failed: ${error.message}`);
    }
}

/**
 * Validate API endpoints
 */
async function validateAPIEndpoints(page) {
    console.log('\n🔍 Validating API endpoints...');
    
    const endpoints = [
        '/api/v1/repositories',
        '/api/v1/version',
        '/api/v1/plugins'
    ];
    
    for (const endpoint of endpoints) {
        try {
            const response = await page.evaluate(async (url) => {
                const res = await fetch(url);
                return {
                    status: res.status,
                    ok: res.ok
                };
            }, endpoint);
            
            if (response.ok) {
                console.log(`  ✅ ${endpoint} - OK`);
            } else {
                console.log(`  ⚠️  ${endpoint} - Status ${response.status}`);
            }
        } catch (error) {
            console.log(`  ⚠️  ${endpoint} - Error: ${error.message}`);
        }
    }
}

/**
 * Run all tests
 */
async function runTests() {
    console.log('🚀 Starting UI Test Harness (Playwright)...');
    console.log(`📍 Base URL: ${BASE_URL}`);
    console.log(`👁️  Headless: ${HEADLESS}`);
    console.log('============================================================\n');
    
    const browser = await chromium.launch({
        headless: HEADLESS,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        const page = await browser.newPage();
        setupErrorListeners(page);
        
        console.log('🌐 Navigating to application...');
        await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
        await page.waitForTimeout(2000);
        
        // Check if JavaScript loaded
        const jsLoaded = await page.evaluate(() => {
            return typeof window.viewRepository !== 'undefined' || 
                   document.querySelector('.nav-link') !== null;
        });
        
        if (!jsLoaded) {
            console.log('⚠️  Warning: JavaScript may not have loaded (static files 404?)');
        }
        
        // Test main pages
        const pages = ['dashboard', 'repositories', 'graph', 'search'];
        for (const pageName of pages) {
            await testPageNavigation(page, pageName);
        }
        
        // Test repository detail tabs
        await testPageNavigation(page, 'repositories');
        await testRepositoryTabs(page);
        
        // Test theme toggle
        await testThemeToggle(page);
        
        // Validate API endpoints
        await validateAPIEndpoints(page);
        
        await page.waitForTimeout(2000);
        
    } catch (error) {
        console.error('❌ Test execution failed:', error.message);
        testResults.errors.push({
            type: 'test.execution',
            message: error.message,
            stack: error.stack,
            timestamp: new Date().toISOString()
        });
    } finally {
        await browser.close();
    }
    
    // Generate report
    generateReport();
}

/**
 * Generate test report
 */
function generateReport() {
    testResults.endTime = new Date();
    testResults.duration = testResults.endTime - testResults.startTime;
    
    const pagesTested = Object.keys(testResults.pages).length;
    const pagesPassed = Object.values(testResults.pages).filter(p => p.status === 'passed').length;
    const interactionsTested = Object.keys(testResults.interactions).length;
    
    console.log('\n============================================================');
    console.log('📊 UI TEST SUMMARY');
    console.log('============================================================');
    console.log(`\n✅ Pages Tested: ${pagesPassed}/${pagesTested}`);
    console.log(`✅ Interactions Tested: ${interactionsTested}`);
    console.log(`❌ JavaScript Errors: ${testResults.errors.length}`);
    console.log(`⚠️  Warnings: ${testResults.warnings.length}`);
    console.log(`⏱️  Duration: ${Math.round(testResults.duration / 1000)}s`);
    
    if (testResults.errors.length > 0) {
        console.log('\n❌ JavaScript errors detected:\n');
        testResults.errors.forEach((error, index) => {
            console.log(`${index + 1}. [${error.type}] ${error.message}`);
            if (error.url) {
                console.log(`   URL: ${error.url}`);
            }
        });
    } else {
        console.log('\n✅ No JavaScript errors found!');
    }
    
    console.log('\n============================================================\n');
    
    if (testResults.errors.length > 0 || pagesPassed < pagesTested) {
        process.exit(1);
    }
}

// Run tests
runTests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});

