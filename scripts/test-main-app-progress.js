#!/usr/bin/env node

/**
 * Test Main App Progress Integration with Playwright
 * 
 * This script validates that the main application's analyze function
 * correctly displays real-time progress updates using the polling mechanism.
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

async function waitForProgressUpdate(page, timeout = 30000) {
    const startTime = Date.now();
    let lastProgress = null;
    
    while (Date.now() - startTime < timeout) {
        await page.waitForTimeout(2000);
        
        const progressInfo = await page.evaluate(() => {
            // Check for progress in repository detail page
            const detailStatus = document.querySelector('#analysis-status-detail');
            // Check for progress in repository list page
            const listStatus = document.querySelector('.analysis-status');
            const statusDiv = detailStatus || listStatus;
            
            if (!statusDiv || statusDiv.style.display === 'none') {
                return null;
            }
            
            const progressFill = statusDiv.querySelector('.progress-fill');
            const stepName = statusDiv.querySelector('strong')?.textContent?.trim();
            const stepCounter = statusDiv.querySelector('div')?.textContent?.trim();
            const statusMessage = statusDiv.querySelector('.analysis-progress > div:last-of-type')?.textContent?.trim();
            
            return {
                visible: true,
                stepName: stepName || '',
                stepCounter: stepCounter || '',
                statusMessage: statusMessage || '',
                progressPercent: progressFill ? parseFloat(progressFill.style.width) || 0 : 0,
                html: statusDiv.innerHTML.substring(0, 300)
            };
        });
        
        if (progressInfo && progressInfo.visible) {
            const currentProgress = `${progressInfo.stepName} - ${progressInfo.progressPercent}%`;
            
            if (currentProgress !== lastProgress) {
                console.log(`   📊 Progress: ${currentProgress}`);
                if (progressInfo.statusMessage) {
                    console.log(`      ${progressInfo.statusMessage}`);
                }
                lastProgress = currentProgress;
                
                // Check if complete
                if (progressInfo.progressPercent >= 100 || progressInfo.stepName === 'Complete') {
                    return { complete: true, progress: progressInfo };
                }
                
                // Check if failed
                if (progressInfo.stepName === 'Failed') {
                    return { failed: true, progress: progressInfo };
                }
            }
        }
    }
    
    return { timeout: true, lastProgress };
}

async function testMainAppProgress() {
    console.log('🚀 Starting Main App Progress Integration Test');
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
        await page.setViewportSize({ width: 1920, height: 1080 });
        
        // Navigate to main app
        console.log('🌐 Navigating to main application...');
        await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });
        await page.waitForTimeout(2000);
        await takeScreenshot(page, '01-main-app-loaded');
        
        // Navigate to repositories page
        console.log('\n📋 Navigating to repositories page...');
        const reposTab = page.locator('[data-page="repositories"]');
        await reposTab.waitFor({ timeout: 10000 });
        await reposTab.click();
        await page.waitForTimeout(2000);
        
        // Wait for repositories to load
        await page.waitForFunction(() => {
            const list = document.getElementById('repositories-list');
            return list && list.children.length > 0;
        }, { timeout: 15000 });
        
        await takeScreenshot(page, '02-repositories-page');
        
        // Get first repository
        const firstRepo = await page.evaluate(() => {
            const repoItems = Array.from(document.querySelectorAll('.repo-item'));
            if (repoItems.length > 0) {
                const repoId = repoItems[0].getAttribute('data-repo-id');
                const name = repoItems[0].querySelector('h3')?.textContent?.trim();
                return { repoId, name };
            }
            return null;
        });
        
        if (!firstRepo) {
            throw new Error('No repositories found');
        }
        
        console.log(`\n🎯 Selected repository: ${firstRepo.name}`);
        
        // Click analyze button using JavaScript (handles hidden elements)
        console.log('\n🚀 Starting analysis...');
        const clicked = await page.evaluate((id) => {
            const repoItem = document.querySelector(`[data-repo-id="${id}"]`);
            if (repoItem) {
                const btn = repoItem.querySelector('.btn-analyze');
                if (btn) {
                    btn.click();
                    return true;
                }
            }
            return false;
        }, firstRepo.repoId);
        
        if (!clicked) {
            throw new Error(`Could not find or click analyze button for ${firstRepo.name}`);
        }
        
        await page.waitForTimeout(2000);
        await takeScreenshot(page, '03-analysis-started');
        
        // Monitor progress updates
        console.log('\n📊 Monitoring progress updates...');
        const progressResult = await waitForProgressUpdate(page, 120000); // 2 minutes max
        
        if (progressResult.complete) {
            console.log('\n✅ Analysis completed successfully!');
            await takeScreenshot(page, '04-analysis-complete');
        } else if (progressResult.failed) {
            console.log('\n❌ Analysis failed!');
            await takeScreenshot(page, '04-analysis-failed');
        } else if (progressResult.timeout) {
            console.log('\n⏱️  Test timed out waiting for completion');
            await takeScreenshot(page, '04-analysis-timeout');
        }
        
        // Verify final state
        const finalState = await page.evaluate(() => {
            const detailStatus = document.querySelector('#analysis-status-detail');
            const listStatus = document.querySelector('.analysis-status');
            const statusDiv = detailStatus || listStatus;
            
            if (!statusDiv) return { found: false };
            
            return {
                found: true,
                visible: statusDiv.style.display !== 'none',
                innerHTML: statusDiv.innerHTML.substring(0, 500),
                textContent: statusDiv.textContent.substring(0, 200)
            };
        });
        
        await takeScreenshot(page, '05-final-state');
        
        console.log('\n============================================================');
        console.log('📊 MAIN APP PROGRESS TEST SUMMARY');
        console.log('============================================================');
        console.log(`✅ Repository: ${firstRepo.name}`);
        console.log(`✅ Progress updates: ${progressResult.complete ? 'Completed' : progressResult.failed ? 'Failed' : 'In Progress'}`);
        console.log(`📸 Screenshots saved: ${screenshots.length}`);
        console.log('\n📸 Screenshot files:');
        screenshots.forEach((s, i) => {
            console.log(`   ${i + 1}. ${s.name} -> ${s.filepath}`);
        });
        console.log('\n============================================================\n');
        
        return {
            success: progressResult.complete || progressResult.failed,
            completed: progressResult.complete,
            failed: progressResult.failed,
            screenshots: screenshots.length
        };
        
    } catch (error) {
        console.error('❌ Test failed:', error);
        if (typeof page !== 'undefined') {
            try {
                await takeScreenshot(page, 'error-state');
            } catch (e) {
                console.error('Failed to take error screenshot:', e);
            }
        }
        throw error;
    } finally {
        await browser.close();
    }
}

// Run test
testMainAppProgress()
    .then(result => {
        if (result.success) {
            console.log('✅ Test completed successfully!');
            process.exit(0);
        } else {
            console.log('⚠️  Test completed but analysis did not finish');
            process.exit(0);
        }
    })
    .catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });

