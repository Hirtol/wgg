export class WggLoginPage {
    readonly page;

    constructor(page) {
        this.page = page;
    }

    async goto() {
        await this.page.goto('/login');
    }

    async login(email: string, password: string) {
        await this.page.getByPlaceholder('name@email.com').click();
        await this.page.getByPlaceholder('name@email.com').fill(email);
        await this.page.getByPlaceholder('name@email.com').press('Tab');
        await this.page.getByPlaceholder('•••••').fill(password);
        await this.page.getByRole('button', { name: 'Login' }).click();
    }

    // async createUser(email: string, password: string, username: string): number {
    //     const query = `mutation createUser($user: UserCreateInput!) {
    //         userCreate(input: $user) {
    //             user {
    //                 id
    //             }
    //         }
    //     }`
    //     let fullQuery = {
    //         query: query,
    //         variables: {
    //             username: username,
    //             email: email,
    //             password: password,
    //             isAdmin: false,
    //         }
    //     }

    //     await fetch("/api/graphql", {method:"post", body: JSON.stringify(fullQuery)});
    // }
}
