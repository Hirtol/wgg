export class WggLandingPage {
    readonly page;
    readonly logoutButton;
    readonly title: string;

    constructor(page) {
        this.page = page;
        this.logoutButton = page.getByRole('link', { name: 'Logout' });
        this.title = "Wgg";
    }

    async goto() {
        await this.page.goto('/');
    }
}
