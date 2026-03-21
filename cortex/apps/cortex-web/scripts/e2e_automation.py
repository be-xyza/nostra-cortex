import time
from playwright.sync_api import sync_playwright

def test_e2e_create_solicitation(page):
    print("Running test_e2e_create_solicitation...")
    page.goto('http://127.0.0.1:5173/heap')
    page.wait_for_load_state('networkidle')
    
    # Open Create Panel
    create_btn = page.locator('button:has-text("+ Create")')
    if create_btn.count() > 0:
        create_btn.first.click()
    else:
        # Fallback to any button containing "Create"
        page.locator('button', has_text="Create").first.click()

    # Select Solicit mode
    page.locator('button:has-text("Solicit Agent")').click()
    
    # Fill role
    page.locator('input[placeholder="e.g. steward.code"]').fill("test.e2e.steward")
    
    # Submit
    page.locator('button:has-text("Direct Solicit")').click()
    
    # Assert
    # We poll to see if Agent Solicitation block appears on the page gracefully
    page.wait_for_selector('text=test.e2e.steward', timeout=10000)
    print("test_e2e_create_solicitation passed!")

def test_e2e_space_switch(page):
    print("Running test_e2e_space_switch...")
    page.goto('http://127.0.0.1:5173/heap')
    page.wait_for_load_state('networkidle')
    
    # Find space selector
    space_selector = page.locator('select')
    space_selector.select_option('system')
    
    # Wait for URL to update
    page.wait_for_url('**/heap?space_id=system')
    
    page.wait_for_load_state('networkidle')
    print("test_e2e_space_switch passed!")

def main():
    print("Starting E2E tests...")
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={'width': 1280, 'height': 800})
        page = context.new_page()
        
        try:
            test_e2e_create_solicitation(page)
            test_e2e_space_switch(page)
            print("All unmocked E2E tests passed successfully.")
        except Exception as e:
            print(f"E2E testing failed: {e}")
            page.screenshot(path='/tmp/e2e_failure.png', full_page=True)
            raise e
        finally:
            browser.close()

if __name__ == "__main__":
    main()
