/**@type {import('@graphql-codegen/cli').CodegenConfig} */
const config = {
    schema: './gen/schema.graphql',
    documents: ['src/**/*.tsx', './src/**/*.graphql'],
    generates: {
        'src/lib/generated/graphql.ts': {
            preset: 'gql-tag-operations-preset',
            plugins: [
                'typescript',
                'typescript-operations',
                'typed-document-node',
                { add: { content: ['//THIS FILE IS GENERATED', '/* eslint-disable */'] } }
            ],
            config: {
                dedupeOperationSuffix: true,
                maybeValue: 'T | null'
            }
        }
    },
    overwrite: true
};

export default config;
