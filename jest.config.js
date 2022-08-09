/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
    preset: 'ts-jest',
    testEnvironment: 'node',
    reporters: [
        'default',
        [
            'jest-junit',
            {
                suiteName: 'swc rewriter tests',
                suiteNameTemplate: 'swc rewriter',
                outputDirectory: './build',
                outputName: 'junit.xml',
            },
        ],
    ],
}
