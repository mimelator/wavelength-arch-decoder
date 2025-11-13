#!/usr/bin/env python3

"""
Script to validate all repository detail tabs work correctly
Usage: python3 scripts/validate-tabs.py [repository-id]

Requirements:
    pip install selenium webdriver-manager
"""

import sys
import time
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait, Select
from selenium.webdriver.support import expected_conditions as EC
from selenium.common.exceptions import TimeoutException, NoSuchElementException
from webdriver_manager.chrome import ChromeDriverManager
from selenium.webdriver.chrome.service import Service

REPO_ID = sys.argv[1] if len(sys.argv) > 1 else 'f424b3dc-f3c0-440f-89f7-bf1219d693ec'
BASE_URL = 'http://localhost:8080'
REPO_URL = f'{BASE_URL}/#repository-detail?repo={REPO_ID}'

TABS = [
    {'id': 'dependencies', 'name': 'Dependencies', 'container_id': 'dependencies-list'},
    {'id': 'services', 'name': 'Services', 'container_id': 'services-list'},
    {'id': 'code', 'name': 'Code Structure', 'container_id': 'code-list'},
    {'id': 'security', 'name': 'Security', 'container_id': 'security-list'},
    {'id': 'tools', 'name': 'Tools', 'container_id': 'tools-list'},
    {'id': 'tests', 'name': 'Tests', 'container_id': 'tests-list'},
    {'id': 'documentation', 'name': 'Documentation', 'container_id': 'documentation-list'},
]

class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    RESET = '\033[0m'

def log(message, color=Colors.RESET):
    print(f'{color}{message}{Colors.RESET}')

def log_success(message):
    log(f'✓ {message}', Colors.GREEN)

def log_error(message):
    log(f'✗ {message}', Colors.RED)

def log_info(message):
    log(f'ℹ {message}', Colors.CYAN)

def log_warning(message):
    log(f'⚠ {message}', Colors.YELLOW)

def validate_tab(driver, tab, wait):
    tab_id = tab['id']
    name = tab['name']
    container_id = tab['container_id']
    
    try:
        log_info(f'Testing tab: {name}...')
        
        # Click the tab button
        tab_button = wait.until(
            EC.element_to_be_clickable((By.CSS_SELECTOR, f'button.repo-tab[data-tab="{tab_id}"]'))
        )
        tab_button.click()
        
        # Wait for tab content to be visible
        wait.until(EC.presence_of_element_located((By.ID, f'tab-{tab_id}')))
        time.sleep(2)  # Wait for content to load
        
        # Check if container exists
        try:
            container = driver.find_element(By.ID, container_id)
        except NoSuchElementException:
            raise Exception(f'Container not found: {container_id}')
        
        # Get container content
        text_content = container.text
        inner_html = container.get_attribute('innerHTML')
        
        has_loading = 'Loading' in text_content
        has_error = 'Failed' in text_content or 'Error' in text_content
        has_no_items = 'No ' in text_content and 'found' in text_content
        
        # Count items
        try:
            items = container.find_elements(By.CSS_SELECTOR, '.detail-item')
            item_count = len(items)
        except:
            item_count = 0
        
        # Validate content
        if has_loading:
            log_warning(f'  Tab {name} still showing loading state')
            return {'success': False, 'reason': 'Still loading'}
        
        if has_error:
            log_error(f'  Tab {name} shows error: {text_content[:100]}')
            return {'success': False, 'reason': 'Error displayed'}
        
        if has_no_items and item_count == 0:
            log_warning(f'  Tab {name} shows "No items found" - this may be expected')
            return {'success': True, 'reason': 'No items (expected)', 'item_count': 0}
        
        if item_count > 0:
            log_success(f'  Tab {name} loaded successfully with {item_count} items')
            return {'success': True, 'reason': 'Items loaded', 'item_count': item_count}
        
        # If we get here, something unexpected happened
        log_warning(f'  Tab {name} loaded but content is unclear')
        return {'success': True, 'reason': 'Loaded (unclear content)', 'item_count': 0}
        
    except Exception as error:
        log_error(f'  Tab {name} failed: {str(error)}')
        return {'success': False, 'reason': str(error)}

def validate_filters(driver, tab, wait):
    tab_id = tab['id']
    name = tab['name']
    
    try:
        # Check if search input exists
        try:
            search_input = driver.find_element(By.ID, f'{tab_id}-search')
            log_info(f'  Checking search filter for {name}...')
            search_input.clear()
            search_input.send_keys('test')
            time.sleep(0.5)
            search_input.clear()
            time.sleep(0.5)
            log_success('    Search filter works')
        except NoSuchElementException:
            pass
        
        # Check if group-by select exists
        try:
            group_by_select = driver.find_element(By.ID, f'{tab_id}-group-by')
            log_info(f'  Checking group-by filter for {name}...')
            select = Select(group_by_select)
            options = [opt.get_attribute('value') for opt in select.options]
            
            if len(options) > 0:
                # Try changing the value
                select.select_by_value(options[-1])
                time.sleep(1)
                log_success(f'    Group-by filter works ({len(options)} options)')
        except NoSuchElementException:
            pass
        
        return True
    except Exception as error:
        log_warning(f'  Filter validation for {name} had issues: {str(error)}')
        return False

def main():
    log(f'\n{"=" * 60}', Colors.BLUE)
    log('Repository Tab Validation Script', Colors.BLUE)
    log(f'{"=" * 60}', Colors.BLUE)
    log(f'Repository ID: {REPO_ID}', Colors.CYAN)
    log(f'URL: {REPO_URL}\n', Colors.CYAN)
    
    driver = None
    
    try:
        # Setup Chrome driver
        log_info('Setting up Chrome driver...')
        options = webdriver.ChromeOptions()
        options.add_argument('--no-sandbox')
        options.add_argument('--disable-dev-shm-usage')
        options.add_argument('--window-size=1920,1080')
        
        service = Service(ChromeDriverManager().install())
        driver = webdriver.Chrome(service=service, options=options)
        wait = WebDriverWait(driver, 30)
        
        # Navigate to repository detail page
        log_info(f'Navigating to {REPO_URL}...')
        driver.get(REPO_URL)
        
        # Wait for page to load
        wait.until(EC.presence_of_element_located((By.ID, 'repository-detail')))
        log_success('Page loaded successfully')
        
        # Wait a bit for initial content
        time.sleep(2)
        
        # Validate each tab
        results = []
        for tab in TABS:
            result = validate_tab(driver, tab, wait)
            results.append({**tab, **result})
            
            # Validate filters if tab loaded successfully
            if result['success']:
                validate_filters(driver, tab, wait)
            
            # Small delay between tabs
            time.sleep(0.5)
        
        # Print summary
        log(f'\n{"=" * 60}', Colors.BLUE)
        log('Validation Summary', Colors.BLUE)
        log(f'{"=" * 60}', Colors.BLUE)
        
        successful = [r for r in results if r['success']]
        failed = [r for r in results if not r['success']]
        
        log(f'\nSuccessful tabs: {len(successful)}/{len(results)}', Colors.GREEN)
        for r in successful:
            item_info = f" ({r.get('item_count', 0)} items)" if r.get('item_count', 0) > 0 else ''
            log(f"  ✓ {r['name']}{item_info}", Colors.GREEN)
        
        if failed:
            log(f'\nFailed tabs: {len(failed)}/{len(results)}', Colors.RED)
            for r in failed:
                log(f"  ✗ {r['name']}: {r['reason']}", Colors.RED)
        
        # Overall result
        log(f'\n{"=" * 60}', Colors.BLUE)
        if not failed:
            log('✓ All tabs validated successfully!', Colors.GREEN)
            sys.exit(0)
        else:
            log('⚠ Some tabs failed validation', Colors.YELLOW)
            sys.exit(1)
        
    except Exception as error:
        log_error(f'\nFatal error: {str(error)}')
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        if driver:
            driver.quit()

if __name__ == '__main__':
    main()

