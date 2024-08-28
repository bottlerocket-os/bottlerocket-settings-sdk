/* [commitlint](https://github.com/conventional-changelog/commitlint) configuration */
import {
    RuleConfigSeverity,
} from '@commitlint/types';

export default {
    parserPreset: 'conventional-changelog-conventionalcommits',
    rules: {
        'header-max-length': [RuleConfigSeverity.Error, 'always', 72],
        'header-trim': [RuleConfigSeverity.Error, 'always'], // No leading/trailing whitespace in subject
        'subject-empty': [RuleConfigSeverity.Error, 'never'], // No empty subject
        'subject-case': [
            RuleConfigSeverity.Error,
            'never',
            ['sentence-case', 'start-case', 'pascal-case', 'upper-case']],
        'subject-full-stop': [RuleConfigSeverity.Error, 'never'], // No full-stop at end of subject
        'body-max-line-length': [RuleConfigSeverity.Error, 'always', 72],
        'body-leading-blank': [RuleConfigSeverity.Error, 'always'], // Empty line before body
        'type-case': [RuleConfigSeverity.Error, 'always', 'lower-case'],
        'type-empty': [RuleConfigSeverity.Error, 'never'],
        'type-enum': [
            RuleConfigSeverity.Error,
            'always',
            [
                'build',
                'chore',
                'ci',
                'docs',
                'feat',
                'fix',
                'perf',
                'refactor',
                'revert',
                'style',
                'test',
            ],
        ],
    },
    ignores: [
        (message) => message.includes("Merge pull request #"), // PR merges are allowed
    ],
};
