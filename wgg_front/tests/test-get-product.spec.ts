import { expect } from '@playwright/test';
import { test } from './fixture.js';
import { WggLandingPage } from './pageObjects/WggLandingPage.js';

let landingPage: WggLandingPage;

test.beforeEach(async ({ authPage }) => {
    landingPage = new WggLandingPage(authPage);
});

test('go to product page and add to cart', async ({ authPage }) => {
    await landingPage.goto();
    await authPage.getByPlaceholder('Product Text').click();

    await authPage.getByPlaceholder('Product Text').fill('melk');

    await authPage.getByPlaceholder('Product Text').press('Enter');

    await authPage.getByText('Jumbo Verse Halfvolle Melk 1L').click();
    await expect(authPage).toHaveURL(/.*498518PAK/);

    await authPage.getByRole('button', { name: 'Add to cart' }).click();
    await authPage.getByRole('link', { name: 'Cart' }).click();

    await expect(authPage).toHaveURL(/.*cart/);
    await expect(authPage.getByRole('link', { name: 'Jumbo Verse Halfvolle Melk 1L' })).toBeVisible();
    // Remove the item again
    await authPage.locator('.btn-icon').first().click();
    await authPage.getByRole('button', { name: 'Subtract Quantity' }).click();

    await expect(authPage.getByRole('link', { name: 'Jumbo Verse Halfvolle Melk 1L' })).toHaveCount(0);
});
