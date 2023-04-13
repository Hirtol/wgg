import { expect } from '@playwright/test';
import { test } from './fixture.js';
import { WggLandingPage } from './pageObjects/WggLandingPage.js';
import { WggLoginPage } from './pageObjects/WggLoginPage.js';

let landingPage: WggLandingPage;

test.beforeEach(async ({ page }) => {
    const loginPage = new WggLoginPage(page);

    await loginPage.goto();

    await loginPage.loginAdmin();
    landingPage = new WggLandingPage(page);
    await landingPage.goto();
});

test('go to product page and add to cart', async ({ page, authPage }) => {
    const loginPage = new WggLoginPage(page);

    await loginPage.goto();

    await loginPage.loginAdmin();
    landingPage = new WggLandingPage(page);
    await landingPage.goto();
    
    await page.getByPlaceholder('Product Text').click();

    await page.getByPlaceholder('Product Text').fill('melk');

    await page.getByPlaceholder('Product Text').press('Enter');

    await page.getByText('Jumbo Verse Halfvolle Melk 1L').click();
    await expect(page).toHaveURL(/.*498518PAK/);

    await page.getByRole('button', { name: 'Add to cart' }).click();
    await page.getByRole('link', { name: 'Cart' }).click();

    await expect(page).toHaveURL(/.*cart/);
    await expect(page.getByRole('link', { name: 'Jumbo Verse Halfvolle Melk 1L' })).toBeVisible();
    // Remove the item again
    await page.locator('.btn-icon').first().click();
    await page.getByRole('button', { name: 'Subtract Quantity' }).click();

    await expect(page.getByRole('link', { name: 'Jumbo Verse Halfvolle Melk 1L' })).toHaveCount(0);
});
