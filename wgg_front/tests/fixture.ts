import { test as base } from '@playwright/test';
import { WggLoginPage } from './pageObjects/WggLoginPage.js';

type WggFixtures = {
    authPage: any;
};

export const test = base.extend<WggFixtures>({
    authPage: async ({ page }, use) => {
        const loginPage = new WggLoginPage(page);

        await loginPage.goto();

        await loginPage.login('admin@admin.com', 'admin');

        await use(page);
    }
});
