import type { PlaywrightTestConfig } from '@playwright/test';

const config: PlaywrightTestConfig = {
    testDir: './tests',
    // fullyParallel: true,
    reporter: process.env.CI ? 'github' : 'list',
    webServer: {
        command: 'npm run build && cargo run',
        reuseExistingServer: !process.env.CI,
        env: { WGG__APP__STATIC_DIR: './build', WGG__APP__PORT: '4173', VITE_COVERAGE: 'true', WGG__DB__DB_PATH: './test.db', WGG__STARTUP_SALE_VALIDATION: 'false', WGG__DB__IN_MEMORY: 'true' },
        port: 4173
    }
};

export default config;
