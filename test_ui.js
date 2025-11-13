const puppeteer = require('puppeteer');

async function testUI() {
    const browser = await puppeteer.launch({ headless: false });
    const page = await browser.newPage();
    
    // Navigate to the repository detail page
    const repoId = 'dc84c5af-e975-4696-98ea-7ec0ca1711d3';
    await page.goto(`http://localhost:8080/#repository-detail?repo=${repoId}&tab=code`, {
        waitUntil: 'networkidle2'
    });
    
    // Wait for the code list to load
    await page.waitForSelector('#code-list', { timeout: 10000 });
    
    // Wait a bit for rendering
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    // Get console logs
    page.on('console', msg => {
        console.log('BROWSER LOG:', msg.text());
    });
    
    // Check what's in the code list
    const codeListHTML = await page.evaluate(() => {
        const list = document.getElementById('code-list');
        return list ? list.innerHTML.substring(0, 2000) : 'NOT FOUND';
    });
    
    // Check filter values
    const filterValues = await page.evaluate(() => {
        const typeFilter = document.getElementById('code-filter-type');
        const langFilter = document.getElementById('code-filter-language');
        return {
            typeFilter: typeFilter ? typeFilter.value : 'NOT FOUND',
            langFilter: langFilter ? langFilter.value : 'NOT FOUND',
            typeFilterOptions: typeFilter ? Array.from(typeFilter.options).map(o => ({ value: o.value, text: o.text })) : [],
            langFilterOptions: langFilter ? Array.from(langFilter.options).map(o => ({ value: o.value, text: o.text })) : []
        };
    });
    
    // Count visible elements
    const elementCounts = await page.evaluate(() => {
        const items = document.querySelectorAll('#code-list .detail-item, #code-list .entity-item');
        const sections = document.querySelectorAll('#code-list .detail-section, #code-list .detail-group');
        
        // Try to get text content
        const allText = document.getElementById('code-list') ? document.getElementById('code-list').innerText : '';
        
        return {
            itemCount: items.length,
            sectionCount: sections.length,
            textLength: allText.length,
            sampleText: allText.substring(0, 500)
        };
    });
    
    // Check for CAF/MWS in the rendered content and get all groups
    const cafMwsCheck = await page.evaluate(() => {
        const list = document.getElementById('code-list');
        if (!list) return { found: false, html: 'NOT FOUND' };
        
        const html = list.innerHTML;
        const text = list.innerText;
        
        // Get all section headers (groups)
        const sections = list.querySelectorAll('.section-header');
        const groups = Array.from(sections).map(s => {
            const headerText = s.textContent.trim();
            const match = headerText.match(/^[▼▶]\s*(.+?)\s*\((\d+)\)/);
            return match ? { name: match[1], count: parseInt(match[2]) } : { name: headerText, count: 0 };
        });
        
        return {
            found: true,
            hasCAF: html.includes('CAF') || text.includes('CAF'),
            hasMWS: html.includes('MWS') || text.includes('MWS'),
            cafCount: (html.match(/CAF/g) || []).length,
            mwsCount: (html.match(/MWS/g) || []).length,
            groups: groups,
            totalGroups: groups.length,
            sampleHTML: html.substring(0, 1000),
            sampleText: text.substring(0, 500)
        };
    });
    
    // Test language filter - filter to show only CAF assets
    await page.select('#code-filter-language', 'webmethods-caf');
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    const cafFilterCheck = await page.evaluate(() => {
        const list = document.getElementById('code-list');
        if (!list) return { found: false };
        
        const items = list.querySelectorAll('.detail-item');
        const cafItems = Array.from(items).filter(item => {
            const badges = item.querySelectorAll('.detail-badge');
            return Array.from(badges).some(b => b.textContent.includes('webmethods-caf'));
        });
        
        return {
            found: true,
            totalItems: items.length,
            cafItems: cafItems.length,
            sampleNames: Array.from(cafItems).slice(0, 5).map(item => {
                const name = item.querySelector('strong');
                return name ? name.textContent : 'Unknown';
            })
        };
    });
    
    console.log('\n=== UI TEST RESULTS ===\n');
    console.log('Filter Values:');
    console.log(JSON.stringify(filterValues, null, 2));
    console.log('\nElement Counts:');
    console.log(JSON.stringify(elementCounts, null, 2));
    console.log('\nCAF/MWS Check (All Elements):');
    console.log(JSON.stringify(cafMwsCheck, null, 2));
    console.log('\nCAF Filter Test:');
    console.log(JSON.stringify(cafFilterCheck, null, 2));
    console.log('\nCode List HTML (first 2000 chars):');
    console.log(codeListHTML.substring(0, 2000));
    
    // Take a screenshot
    await page.screenshot({ path: '/tmp/ui_test_screenshot.png', fullPage: true });
    console.log('\nScreenshot saved to /tmp/ui_test_screenshot.png');
    
    await browser.close();
}

testUI().catch(console.error);

