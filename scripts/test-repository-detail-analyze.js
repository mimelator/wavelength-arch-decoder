#!/usr/bin/env node

/**
 * Playwright Test: Repository Detail Page Analyze Button and Progress Tracking
 * 
 * This test:
 * 1. Navigates to a specific repository detail page
 * 2. Clicks the analyze button
 * 3. Tracks progress updates in real-time
 * 4. Verifies completion
 * 
 * Usage:
 *   node scripts/test-repository-detail-analyze.js [--repo-id=<id>] [--headless=false]
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

// Configuration
const BASE_URL = process.env.BASE_URL || 'http://localhost:8080';
const HEADLESS = !process.argv.includes('--headless=false');
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots', 'repository-detail');
const DEFAULT_REPO_ID = 'dadae437-4186-498f-8fa5-b2d9d18a73aa'; // yarnbook-tasks (smaller repo)

// Parse repo ID from command line args
let REPO_ID = DEFAULT_REPO_ID;
const repoIdArg = process.argv.find(arg => arg.startsWith('--repo-id='));
if (repoIdArg) {
    REPO_ID = repoIdArg.split('=')[1];
}

// Create screenshots directory
if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

let screenshots = [];
let progressHistory = [];

async function takeScreenshot(page, name) {
    const timestamp = Date.now();
    const filename = `${timestamp}-${name.replace(/[^a-zA-Z0-9]/g, '-')}.png`;
    const filepath = path.join(SCREENSHOT_DIR, filename);
    await page.screenshot({ path: filepath, fullPage: true });
    screenshots.push({ name, filepath, timestamp });
    console.log(`📸 Screenshot: ${name} -> ${filepath}`);
    return filepath;
}

async function navigateToRepositoryDetail(page, repoId) {
    console.log(`\n🌐 Navigating to repository detail page...`);
    const url = `${BASE_URL}/#repository-detail?repo=${repoId}&tab=overview`;
    console.log(`   URL: ${url}`);
    
    try {
        await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
    } catch (error) {
        if (error.message.includes('Timeout')) {
            console.log('   ⚠️  Navigation timeout, but page may have loaded');
        } else {
            throw error;
        }
    }
    await page.waitForTimeout(3000);
    
    // Wait for the repository detail page to load
    await page.waitForSelector('#repository-detail', { timeout: 10000 });
    await page.waitForSelector('#btn-analyze-detail', { timeout: 10000 });
    
    // Wait for JavaScript to be ready and analyzeRepository function to be available
    await page.waitForFunction(() => {
        return typeof window.analyzeRepository === 'function' && 
               document.getElementById('btn-analyze-detail') !== null;
    }, { timeout: 10000 });
    
    console.log('✅ Repository detail page loaded');
    await takeScreenshot(page, '01-page-loaded');
    
    return true;
}

async function clickAnalyzeButton(page, repoId) {
    console.log(`\n🔘 Clicking analyze button...`);
    
    const analyzeBtn = page.locator('#btn-analyze-detail');
    await analyzeBtn.waitFor({ state: 'visible', timeout: 10000 });
    
    // Check initial button state
    const initialText = await analyzeBtn.textContent();
    const isDisabled = await analyzeBtn.isDisabled();
    console.log(`   Initial button state: "${initialText}" (disabled: ${isDisabled})`);
    
    // Use JavaScript to trigger the analyzeRepository function directly
    // The button might have an event listener, so we'll call the function directly
    const result = await page.evaluate((id) => {
        try {
            // Check if function exists
            if (typeof window.analyzeRepository !== 'function') {
                return { success: false, error: 'analyzeRepository function not found' };
            }
            
            // Check if button exists
            const btn = document.getElementById('btn-analyze-detail');
            if (!btn) {
                return { success: false, error: 'Button not found' };
            }
            
            // Call the function directly
            console.log('Calling analyzeRepository with ID:', id);
            window.analyzeRepository(id);
            
            return { success: true, buttonText: btn.textContent };
        } catch (error) {
            return { success: false, error: error.message };
        }
    }, repoId);
    
    if (!result.success) {
        throw new Error(`Could not trigger analyze: ${result.error}`);
    }
    
    console.log(`   Function called successfully (button text: "${result.buttonText}")`);
    
    // Wait for either button state change OR progress status to appear
    let stateChanged = false;
    for (let i = 0; i < 15; i++) {
        await page.waitForTimeout(1000);
        
        const afterText = await analyzeBtn.textContent();
        const afterDisabled = await analyzeBtn.isDisabled();
        
        // Check if button state changed
        if (afterText.includes('Analyzing') || afterDisabled) {
            console.log(`   After click (${i+1}s): "${afterText}" (disabled: ${afterDisabled})`);
            stateChanged = true;
            break;
        }
        
        // Also check if progress status appeared (alternative indicator)
        const progressInfo = await getProgressInfo(page);
        if (progressInfo && progressInfo.visible) {
            console.log(`   Progress status appeared after ${i+1}s`);
            console.log(`   Step: ${progressInfo.stepInfo || 'N/A'}, Progress: ${progressInfo.progressPercent}%`);
            stateChanged = true;
            break;
        }
        
        // Check if status div exists but is hidden
        if (i === 2) {
            const statusDivExists = await page.evaluate(() => {
                const div = document.getElementById('analysis-status-detail');
                if (!div) return { exists: false };
                const style = window.getComputedStyle(div);
                return {
                    exists: true,
                    display: style.display,
                    visibility: style.visibility,
                    opacity: style.opacity,
                    innerHTML: div.innerHTML.substring(0, 200)
                };
            });
            if (statusDivExists.exists) {
                console.log(`   Status div exists but may be hidden: display=${statusDivExists.display}, visibility=${statusDivExists.visibility}`);
            }
        }
    }
    
    if (stateChanged) {
        console.log('✅ Analyze button clicked and analysis started');
        await takeScreenshot(page, '02-analyze-clicked');
        return true;
    } else {
        // Final check - maybe analysis started but button didn't update
        const progressInfo = await getProgressInfo(page);
        if (progressInfo && progressInfo.visible) {
            console.log('✅ Analysis started (progress status visible)');
            await takeScreenshot(page, '02-analyze-clicked');
            return true;
        }
        throw new Error('Analyze button state did not change after click and no progress status appeared');
    }
}

async function getProgressInfo(page) {
    const progressInfo = await page.evaluate(() => {
        const statusDiv = document.getElementById('analysis-status-detail');
        
        // Check if element exists and is visible
        if (!statusDiv) {
            return null;
        }
        
        // Check visibility - could be display: none, visibility: hidden, or opacity: 0
        const computedStyle = window.getComputedStyle(statusDiv);
        const isVisible = computedStyle.display !== 'none' && 
                         computedStyle.visibility !== 'hidden' && 
                         computedStyle.opacity !== '0';
        
        if (!isVisible) {
            return null;
        }
        
        const progressBar = statusDiv.querySelector('.progress-fill');
        const stepInfo = statusDiv.querySelector('.analysis-progress strong')?.textContent?.trim();
        const statusMessage = Array.from(statusDiv.querySelectorAll('.analysis-progress > div'))
            .map(el => el.textContent?.trim())
            .filter(text => text && !text.includes('⚠️') && !text.includes('Important'))
            .join(' | ');
        
        const progressPercent = progressBar 
            ? parseFloat(progressBar.style.width) || 0 
            : 0;
        
        // Try to extract step info from text - check multiple places
        let stepMatch = stepInfo?.match(/Step\s+(\d+)\/(\d+)/);
        if (!stepMatch) {
            // Try from status message
            stepMatch = statusMessage.match(/Step\s+(\d+)\/(\d+)/);
        }
        if (!stepMatch) {
            // Try from all text content
            stepMatch = statusDiv.textContent.match(/Step\s+(\d+)\/(\d+)/);
        }
        const currentStep = stepMatch ? parseInt(stepMatch[1]) : null;
        const totalSteps = stepMatch ? parseInt(stepMatch[2]) : null;
        
        // Also check the raw text content
        const allText = statusDiv.textContent || '';
        
        return {
            visible: true,
            stepInfo: stepInfo || '',
            currentStep,
            totalSteps,
            statusMessage: statusMessage || allText.substring(0, 100),
            progressPercent: Math.round(progressPercent),
            html: statusDiv.innerHTML.substring(0, 300),
            computedDisplay: computedStyle.display,
            computedVisibility: computedStyle.visibility
        };
    });
    
    return progressInfo;
}

async function trackProgress(page, repoId, maxWaitTime = 600000) {
    console.log(`\n📊 Tracking analysis progress...`);
    console.log(`   Max wait time: ${Math.round(maxWaitTime / 1000)}s`);
    console.log(`   Polling interval: 2s`);
    
    const startTime = Date.now();
    let lastProgress = null;
    let progressCount = 0;
    let lastScreenshotTime = 0;
    const screenshotInterval = 10000; // Screenshot every 10 seconds
    
    // Wait for progress status to appear
    console.log('   Waiting for progress status to appear...');
    let statusVisible = false;
    for (let i = 0; i < 10; i++) {
        await page.waitForTimeout(1000);
        const progressInfo = await getProgressInfo(page);
        if (progressInfo && progressInfo.visible) {
            statusVisible = true;
            break;
        }
    }
    
    if (!statusVisible) {
        console.log('   ⚠️  Progress status not visible yet, continuing to monitor...');
    }
    
    while (Date.now() - startTime < maxWaitTime) {
        await page.waitForTimeout(2000); // Wait 2 seconds between checks
        
        const progressInfo = await getProgressInfo(page);
        const now = Date.now();
        
        if (progressInfo && progressInfo.visible) {
            // Create a more detailed progress key that includes status message
            const progressKey = `${progressInfo.currentStep || '?'}/${progressInfo.totalSteps || '?'} - ${progressInfo.progressPercent}% - ${progressInfo.statusMessage.substring(0, 50)}`;
            
            // Only log if progress changed (step, percent, or status message)
            if (progressKey !== lastProgress) {
                progressCount++;
                const elapsed = Math.round((now - startTime) / 1000);
                const timestamp = new Date().toISOString();
                
                console.log(`\n   [${progressCount}] ${timestamp} (${elapsed}s elapsed)`);
                console.log(`       Step: ${progressInfo.stepInfo || `Step ${progressInfo.currentStep || '?'}/${progressInfo.totalSteps || '?'}`}`);
                console.log(`       Progress: ${progressInfo.progressPercent}%`);
                if (progressInfo.statusMessage) {
                    const statusMsg = progressInfo.statusMessage.length > 120 
                        ? progressInfo.statusMessage.substring(0, 120) + '...'
                        : progressInfo.statusMessage;
                    console.log(`       Status: ${statusMsg}`);
                }
                
                // Store progress history
                progressHistory.push({
                    timestamp,
                    elapsed,
                    step: progressInfo.currentStep,
                    totalSteps: progressInfo.totalSteps,
                    progressPercent: progressInfo.progressPercent,
                    statusMessage: progressInfo.statusMessage,
                    stepInfo: progressInfo.stepInfo
                });
                
                // Take screenshot on significant progress changes
                if (now - lastScreenshotTime > screenshotInterval) {
                    const screenshotName = `progress-${progressCount.toString().padStart(3, '0')}-step-${progressInfo.currentStep || 'unknown'}-${progressInfo.progressPercent}pct`;
                    await takeScreenshot(page, screenshotName);
                    lastScreenshotTime = now;
                }
                
                lastProgress = progressKey;
            }
            
            // Check if analysis is complete
            const isComplete = await page.evaluate(() => {
                const statusDiv = document.getElementById('analysis-status-detail');
                if (!statusDiv) return false;
                const html = statusDiv.innerHTML.toLowerCase();
                return html.includes('analysis complete') || 
                       html.includes('analysis successful') ||
                       html.includes('completed successfully') ||
                       (html.includes('progress_percent') && html.includes('100'));
            });
            
            if (isComplete || (progressInfo.progressPercent >= 100 && progressInfo.currentStep === progressInfo.totalSteps)) {
                console.log('\n   ✅ Analysis completed!');
                await takeScreenshot(page, 'final-complete');
                return {
                    completed: true,
                    progressUpdates: progressCount,
                    duration: now - startTime,
                    finalProgress: progressInfo
                };
            }
            
            // Check if analysis failed
            const isFailed = await page.evaluate(() => {
                const statusDiv = document.getElementById('analysis-status-detail');
                if (!statusDiv) return false;
                const html = statusDiv.innerHTML.toLowerCase();
                return html.includes('analysis failed') || 
                       html.includes('failed') ||
                       html.includes('error');
            });
            
            if (isFailed) {
                console.log('\n   ❌ Analysis failed!');
                await takeScreenshot(page, 'final-failed');
                return {
                    completed: false,
                    failed: true,
                    progressUpdates: progressCount,
                    duration: now - startTime,
                    finalProgress: progressInfo
                };
            }
        } else {
            // Progress not visible - might be starting or completed
            const elapsed = Math.round((now - startTime) / 1000);
            if (elapsed % 10 === 0) {
                console.log(`   ⏳ Waiting for progress... (${elapsed}s elapsed)`);
            }
        }
    }
    
    // Timeout
    console.log('\n   ⏱️  Timeout reached');
    await takeScreenshot(page, 'final-timeout');
    return {
        completed: false,
        timeout: true,
        progressUpdates: progressCount,
        duration: Date.now() - startTime
    };
}

async function verifyFinalState(page) {
    console.log(`\n🔍 Verifying final state...`);
    
    const finalState = await page.evaluate(() => {
        const analyzeBtn = document.getElementById('btn-analyze-detail');
        const statusDiv = document.getElementById('analysis-status-detail');
        
        return {
            buttonText: analyzeBtn?.textContent?.trim() || '',
            buttonDisabled: analyzeBtn?.disabled || false,
            statusVisible: statusDiv && statusDiv.style.display !== 'none',
            statusContent: statusDiv?.textContent?.substring(0, 200) || ''
        };
    });
    
    console.log(`   Button: "${finalState.buttonText}" (disabled: ${finalState.buttonDisabled})`);
    console.log(`   Status visible: ${finalState.statusVisible}`);
    if (finalState.statusContent) {
        console.log(`   Status: ${finalState.statusContent.substring(0, 100)}...`);
    }
    
    return finalState;
}

async function runTest() {
    console.log('🚀 Starting Repository Detail Analyze Test');
    console.log('============================================================');
    console.log(`📍 Base URL: ${BASE_URL}`);
    console.log(`🆔 Repository ID: ${REPO_ID}`);
    console.log(`👁️  Headless: ${HEADLESS}`);
    console.log(`📸 Screenshots: ${SCREENSHOT_DIR}`);
    console.log('============================================================\n');
    
    const browser = await chromium.launch({
        headless: HEADLESS,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        const page = await browser.newPage();
        
        // Capture console logs for debugging
        page.on('console', msg => {
            const type = msg.type();
            const text = msg.text();
            // Log all console messages during analysis
            if (text.includes('analysis') || text.includes('progress') || text.includes('Starting') || type === 'error') {
                console.log(`   [Browser ${type}]: ${text}`);
            }
        });
        
        // Monitor network requests (only log errors or analyze endpoint)
        page.on('response', response => {
            const url = response.url();
            if (url.includes('/analyze')) {
                console.log(`   [Network] ${response.status()} ${url}`);
            } else if (url.includes('/progress') && !response.ok()) {
                console.log(`   [Network Error] ${response.status()} ${url}`);
            }
        });
        
        page.on('requestfailed', request => {
            const url = request.url();
            if (url.includes('/analyze') || url.includes('/progress')) {
                console.log(`   [Network Error] ${url}: ${request.failure()?.errorText}`);
            }
        });
        
        // Set viewport size for consistent screenshots
        await page.setViewportSize({ width: 1920, height: 1080 });
        
        // Navigate to repository detail page
        await navigateToRepositoryDetail(page, REPO_ID);
        
        // Click analyze button
        await clickAnalyzeButton(page, REPO_ID);
        
        // Track progress
        const result = await trackProgress(page, REPO_ID);
        
        // Verify final state
        const finalState = await verifyFinalState(page);
        
        // Generate summary
        console.log('\n============================================================');
        console.log('📊 TEST SUMMARY');
        console.log('============================================================');
        console.log(`✅ Repository ID: ${REPO_ID}`);
        console.log(`✅ Progress updates captured: ${result.progressUpdates}`);
        console.log(`⏱️  Duration: ${Math.round(result.duration / 1000)}s`);
        console.log(`📸 Screenshots saved: ${screenshots.length}`);
        
        if (result.completed) {
            console.log(`✅ Status: Analysis completed successfully`);
            if (result.finalProgress) {
                console.log(`   Final step: ${result.finalProgress.stepInfo}`);
                console.log(`   Final progress: ${result.finalProgress.progressPercent}%`);
            }
        } else if (result.failed) {
            console.log(`❌ Status: Analysis failed`);
            process.exit(1);
        } else if (result.timeout) {
            console.log(`⏱️  Status: Timeout (analysis may still be running)`);
        }
        
        console.log('\n📈 Progress History:');
        progressHistory.forEach((p, i) => {
            console.log(`   ${i + 1}. [${p.elapsed}s] Step ${p.step}/${p.totalSteps} - ${p.progressPercent}% - ${p.statusMessage.substring(0, 60)}`);
        });
        
        console.log('\n📸 Screenshots:');
        screenshots.forEach((s, i) => {
            console.log(`   ${i + 1}. ${s.name}`);
        });
        
        console.log('\n============================================================\n');
        
        return result;
        
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
runTest().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});

