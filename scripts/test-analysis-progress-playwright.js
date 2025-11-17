#!/usr/bin/env node

/**
 * Test Analysis Progress with Playwright
 * 
 * This script:
 * 1. Lists all repositories
 * 2. Validates we have repositories
 * 3. Picks one and starts analysis
 * 4. Captures progress updates with screenshots
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

// Configuration
const BASE_URL = process.env.BASE_URL || 'http://localhost:8080';
const HEADLESS = !process.argv.includes('--headless=false');
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots');

// Create screenshots directory
if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

let screenshots = [];

async function takeScreenshot(page, name) {
    const filename = `${Date.now()}-${name}.png`;
    const filepath = path.join(SCREENSHOT_DIR, filename);
    await page.screenshot({ path: filepath, fullPage: true });
    screenshots.push({ name, filepath });
    console.log(`📸 Screenshot saved: ${name} -> ${filepath}`);
    return filepath;
}

async function listRepositories(page) {
    console.log('\n📋 Listing repositories...');
    
    // Navigate to base URL first
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(2000);
    
    // Click on Repositories tab
    const reposTab = page.locator('[data-page="repositories"]');
    await reposTab.waitFor({ timeout: 10000 });
    await reposTab.click();
    await page.waitForTimeout(2000);
    
    // Wait for repositories list to load (check if it has content)
    await page.waitForFunction(() => {
        const list = document.getElementById('repositories-list');
        return list && (list.children.length > 0 || list.textContent.trim() !== '');
    }, { timeout: 15000 });
    
    await page.waitForTimeout(1000);
    
    // Get repository count and details (deduplicate by ID)
    const repos = await page.evaluate(() => {
        const repoItems = Array.from(document.querySelectorAll('.repo-item'));
        const seen = new Set();
        const unique = [];
        
        repoItems.forEach(item => {
            const repoId = item.getAttribute('data-repo-id');
            if (!seen.has(repoId)) {
                seen.add(repoId);
                const name = item.querySelector('h3')?.textContent?.trim() || 'Unknown';
                const url = item.querySelector('p')?.textContent?.trim() || '';
                const analyzeBtn = item.querySelector('.btn-analyze');
                unique.push({
                    id: repoId,
                    name,
                    url,
                    hasAnalyzeButton: !!analyzeBtn
                });
            }
        });
        
        return unique;
    });
    
    console.log(`✅ Found ${repos.length} unique repository(ies):`);
    repos.forEach((repo, index) => {
        console.log(`   ${index + 1}. ${repo.name} (ID: ${repo.id.substring(0, 8)}...)`);
    });
    
    await takeScreenshot(page, '01-repositories-list');
    
    return repos;
}

async function startAnalysis(page, repoId, repoName) {
    console.log(`\n🚀 Starting analysis for: ${repoName}...`);
    
    // Use JavaScript to click the button directly (handles hidden elements)
    const clicked = await page.evaluate((id) => {
        const repoItem = document.querySelector(`[data-repo-id="${id}"]`);
        if (repoItem) {
            const analyzeBtn = repoItem.querySelector('.btn-analyze');
            if (analyzeBtn) {
                analyzeBtn.click();
                return true;
            }
        }
        return false;
    }, repoId);
    
    if (!clicked) {
        throw new Error(`Could not find or click analyze button for ${repoName}`);
    }
    
    await page.waitForTimeout(2000);
    
    console.log('✅ Analysis started');
    await takeScreenshot(page, '02-analysis-started');
    
    // Monitor progress for up to 60 seconds
    const maxWaitTime = 60000; // 60 seconds
    const startTime = Date.now();
    let lastProgress = null;
    let progressCount = 0;
    
    console.log('\n📊 Monitoring progress updates...');
    
    while (Date.now() - startTime < maxWaitTime) {
        await page.waitForTimeout(2000); // Wait 2 seconds between checks
        
        // Check for progress display
        const progressInfo = await page.evaluate(() => {
            const statusDiv = document.querySelector('.analysis-status, #analysis-status-detail');
            if (!statusDiv) return null;
            
            const progressBar = statusDiv.querySelector('.progress-fill');
            const stepName = statusDiv.querySelector('strong')?.textContent?.trim();
            const statusMessage = statusDiv.querySelector('.analysis-progress > div:last-child')?.textContent?.trim();
            
            return {
                visible: statusDiv.style.display !== 'none',
                stepName: stepName || '',
                statusMessage: statusMessage || '',
                progressPercent: progressBar ? parseFloat(progressBar.style.width) || 0 : 0,
                html: statusDiv.innerHTML.substring(0, 500)
            };
        });
        
        if (progressInfo && progressInfo.visible) {
            const currentProgress = `${progressInfo.stepName} - ${progressInfo.progressPercent}%`;
            
            // Only log if progress changed
            if (currentProgress !== lastProgress) {
                progressCount++;
                console.log(`   [${progressCount}] ${currentProgress}`);
                if (progressInfo.statusMessage) {
                    console.log(`       ${progressInfo.statusMessage}`);
                }
                
                await takeScreenshot(page, `03-progress-${progressCount.toString().padStart(2, '0')}-${progressInfo.stepName.replace(/[^a-zA-Z0-9]/g, '-').substring(0, 20)}`);
                lastProgress = currentProgress;
            }
            
            // Check if analysis is complete
            const isComplete = await page.evaluate(() => {
                const statusDiv = document.querySelector('.analysis-status, #analysis-status-detail');
                if (!statusDiv) return false;
                const html = statusDiv.innerHTML;
                return html.includes('Analysis Complete') || 
                       html.includes('analysis-success') ||
                       (html.includes('progress_percent') && html.includes('100'));
            });
            
            if (isComplete) {
                console.log('\n✅ Analysis completed!');
                await takeScreenshot(page, '04-analysis-complete');
                break;
            }
            
            // Check if analysis failed
            const isFailed = await page.evaluate(() => {
                const statusDiv = document.querySelector('.analysis-status, #analysis-status-detail');
                if (!statusDiv) return false;
                const html = statusDiv.innerHTML;
                return html.includes('Analysis Failed') || 
                       html.includes('analysis-error') ||
                       html.includes('Failed');
            });
            
            if (isFailed) {
                console.log('\n❌ Analysis failed!');
                await takeScreenshot(page, '04-analysis-failed');
                break;
            }
        }
    }
    
    // Final screenshot
    await takeScreenshot(page, '05-final-state');
    
    return {
        progressUpdates: progressCount,
        duration: Date.now() - startTime
    };
}

async function runTest() {
    console.log('🚀 Starting Analysis Progress Test');
    console.log(`📍 Base URL: ${BASE_URL}`);
    console.log(`👁️  Headless: ${HEADLESS}`);
    console.log(`📸 Screenshots will be saved to: ${SCREENSHOT_DIR}`);
    console.log('============================================================\n');
    
    const browser = await chromium.launch({
        headless: HEADLESS,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        const page = await browser.newPage();
        
        // Set viewport size for consistent screenshots
        await page.setViewportSize({ width: 1920, height: 1080 });
        
        // List repositories
        const repos = await listRepositories(page);
        
        if (repos.length === 0) {
            console.log('\n❌ No repositories found! Cannot test analysis progress.');
            return;
        }
        
        // Pick the first repository
        const selectedRepo = repos[0];
        console.log(`\n🎯 Selected repository: ${selectedRepo.name}`);
        
        // Start analysis and monitor progress
        const result = await startAnalysis(page, selectedRepo.id, selectedRepo.name);
        
        console.log('\n============================================================');
        console.log('📊 ANALYSIS PROGRESS TEST SUMMARY');
        console.log('============================================================');
        console.log(`✅ Repository: ${selectedRepo.name}`);
        console.log(`✅ Progress updates captured: ${result.progressUpdates}`);
        console.log(`⏱️  Duration: ${Math.round(result.duration / 1000)}s`);
        console.log(`📸 Screenshots saved: ${screenshots.length}`);
        console.log('\n📸 Screenshot files:');
        screenshots.forEach((s, i) => {
            console.log(`   ${i + 1}. ${s.name} -> ${s.filepath}`);
        });
        console.log('\n============================================================\n');
        
    } catch (error) {
        console.error('❌ Test failed:', error);
        if (typeof page !== 'undefined') {
            try {
                await takeScreenshot(page, 'error-state');
            } catch (e) {
                console.error('Failed to take error screenshot:', e);
            }
        }
    } finally {
        await browser.close();
    }
}

// Run test
runTest().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});

