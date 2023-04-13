import { expect } from '@playwright/test';
import { test } from './fixture.js';
import { WggLandingPage } from './pageObjects/WggLandingPage.js';
import { WggLoginPage } from './pageObjects/WggLoginPage.js';

let loginPage: WggLoginPage;
let landingPage: WggLandingPage;

test.beforeEach(async ({ page }) => {
    loginPage = new WggLoginPage(page);

    await loginPage.goto();

    await loginPage.loginAdmin();
    landingPage = new WggLandingPage(page);
    await landingPage.goto();
});

test('create aggregate product and delete', async ({ page, authPage }) => {
    const loginPage = new WggLoginPage(page);

    await loginPage.goto();

    await loginPage.loginAdmin();
    landingPage = new WggLandingPage(page);
    await landingPage.goto();
    
    await page.getByRole('link', { name: 'Aggregates' }).click();
    await expect(page).toHaveURL(/.*aggregates/);
    await page.getByRole('button', { name: 'Create new' }).click();
    await page.getByPlaceholder('Enter name...').fill('Two Products');
    await page.getByPlaceholder('Enter name...').press('Enter');
    await page.getByText('Two Products').click();

    await page.getByRole('button', { name: 'Delete aggregate ingredient' }).click();
    await page.getByRole('button', { name: 'Confirm' }).click();
    await expect(page).toHaveURL(/.*aggregates/);

    // Delete the new ingredient
    // await page.getByTitle(/Delete aggregate.*/).click();

    // await expect(page).toHaveURL(/.*aggregates/);
    await expect(page.getByRole('link', { name: 'Two Products' })).toHaveCount(0);
});
