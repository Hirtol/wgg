import { test, expect } from '@playwright/test';
import { WggLandingPage } from './pageObjects/WggLandingPage.js';
import { WggLoginPage } from './pageObjects/WggLoginPage.js';

test('index page is login page', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByText('Please sign in')).toBeVisible();
});

test.describe('auth flow', () => {
    let login: WggLoginPage;
    let landingPage: WggLandingPage;

    test.beforeEach(({ page }) => {
        login = new WggLoginPage(page);
        landingPage = new WggLandingPage(page);

        login.goto();
    });

    test('test default login', async ({ page }) => {
        await login.login('admin@admin.com', 'admin');

        // Should be logged in
        await expect(page).toHaveTitle(landingPage.title);

        // Try to log out
        await landingPage.logoutButton.click();
        await expect(page).toHaveTitle(/.*Login.*/);
        // We should no longer migrate to the landing page
        await landingPage.goto();
        await expect(page).toHaveTitle(/.*Login.*/);
    });

    test('test default login persistent', async ({ page }) => {
        await login.login('admin@admin.com', 'admin');

        page.reload();
        landingPage.goto();
        // Ensure we're logged in
        await expect(landingPage.logoutButton).toBeEnabled();
    });
});
