import { Hono } from 'hono';
import { type Language, languages } from '../data/languages';
import { providers } from '../data/providers';

interface GeneratePipelineRequest {
  languages: string[];
  provider: string;
  options: {
    lint: boolean;
    test: boolean;
    build: boolean;
    cache: boolean;
  };
}

const app = new Hono();

function generateGitHubActionsYaml(
  selectedLanguages: Language[],
  options: GeneratePipelineRequest['options'],
): string {
  const jobs: string[] = [];

  for (const lang of selectedLanguages) {
    const steps: string[] = [];
    steps.push(`      - uses: actions/checkout@v4`);

    if (options.cache) {
      steps.push(`      - name: Setup ${lang.name} cache
        uses: actions/cache@v4
        with:
          path: ~/.cache
          key: \${{ runner.os }}-${lang.id}-\${{ hashFiles('**/*.lock') }}`);
    }

    if (options.lint && lang.lintCommands.length > 0) {
      steps.push(`      - name: Lint
        run: ${lang.lintCommands[0]}`);
    }

    if (options.build && lang.buildTools.length > 0) {
      steps.push(`      - name: Build
        run: ${lang.buildTools[0]} build`);
    }

    if (options.test && lang.testCommands.length > 0) {
      steps.push(`      - name: Test
        run: ${lang.testCommands[0]}`);
    }

    jobs.push(`  ${lang.id}:
    runs-on: ubuntu-latest
    steps:
${steps.join('\n')}`);
  }

  return `name: CI Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
${jobs.join('\n\n')}`;
}

function generateTektonYaml(
  selectedLanguages: Language[],
  options: GeneratePipelineRequest['options'],
): string {
  const tasks: string[] = [];

  for (const lang of selectedLanguages) {
    const steps: string[] = [];

    if (options.lint && lang.lintCommands.length > 0) {
      steps.push(`        - name: lint
          image: ${lang.id}:latest
          script: |
            ${lang.lintCommands[0]}`);
    }

    if (options.build && lang.buildTools.length > 0) {
      steps.push(`        - name: build
          image: ${lang.id}:latest
          script: |
            ${lang.buildTools[0]} build`);
    }

    if (options.test && lang.testCommands.length > 0) {
      steps.push(`        - name: test
          image: ${lang.id}:latest
          script: |
            ${lang.testCommands[0]}`);
    }

    tasks.push(`  - name: ${lang.id}-pipeline
    taskSpec:
      steps:
${steps.join('\n')}`);
  }

  return `apiVersion: tekton.dev/v1beta1
kind: Pipeline
metadata:
  name: ci-pipeline
spec:
  tasks:
${tasks.join('\n')}`;
}

function generateGitLabCIYaml(
  selectedLanguages: Language[],
  options: GeneratePipelineRequest['options'],
): string {
  const stages: string[] = [];
  const jobs: string[] = [];

  if (options.lint) stages.push('lint');
  if (options.build) stages.push('build');
  if (options.test) stages.push('test');

  for (const lang of selectedLanguages) {
    if (options.lint && lang.lintCommands.length > 0) {
      jobs.push(`${lang.id}-lint:
  stage: lint
  image: ${lang.id}:latest
  script:
    - ${lang.lintCommands[0]}${options.cache ? `\n  cache:\n    paths:\n      - .cache/` : ''}`);
    }

    if (options.build && lang.buildTools.length > 0) {
      jobs.push(`${lang.id}-build:
  stage: build
  image: ${lang.id}:latest
  script:
    - ${lang.buildTools[0]} build`);
    }

    if (options.test && lang.testCommands.length > 0) {
      jobs.push(`${lang.id}-test:
  stage: test
  image: ${lang.id}:latest
  script:
    - ${lang.testCommands[0]}`);
    }
  }

  return `stages:
  - ${stages.join('\n  - ')}

${jobs.join('\n\n')}`;
}

function generateCircleCIYaml(
  selectedLanguages: Language[],
  options: GeneratePipelineRequest['options'],
): string {
  const jobs: string[] = [];
  const workflowJobs: string[] = [];

  for (const lang of selectedLanguages) {
    const steps: string[] = [];
    steps.push('      - checkout');

    if (options.cache) {
      steps.push(`      - restore_cache:
          keys:
            - ${lang.id}-deps-{{ checksum "*.lock" }}`);
    }

    if (options.lint && lang.lintCommands.length > 0) {
      steps.push(`      - run:
          name: Lint
          command: ${lang.lintCommands[0]}`);
    }

    if (options.build && lang.buildTools.length > 0) {
      steps.push(`      - run:
          name: Build
          command: ${lang.buildTools[0]} build`);
    }

    if (options.test && lang.testCommands.length > 0) {
      steps.push(`      - run:
          name: Test
          command: ${lang.testCommands[0]}`);
    }

    if (options.cache) {
      steps.push(`      - save_cache:
          paths:
            - ~/.cache
          key: ${lang.id}-deps-{{ checksum "*.lock" }}`);
    }

    jobs.push(`  ${lang.id}:
    docker:
      - image: cimg/${lang.id}:latest
    steps:
${steps.join('\n')}`);

    workflowJobs.push(`          - ${lang.id}`);
  }

  return `version: 2.1

jobs:
${jobs.join('\n\n')}

workflows:
  version: 2
  ci:
    jobs:
${workflowJobs.join('\n')}`;
}

app.post('/generate', async (c) => {
  const body = await c.req.json<GeneratePipelineRequest>();

  const selectedLanguages = languages.filter((l) =>
    body.languages.includes(l.id),
  );
  const provider = providers.find((p) => p.id === body.provider);

  if (selectedLanguages.length === 0) {
    return c.json({ error: 'No valid languages selected' }, 400);
  }

  if (!provider) {
    return c.json({ error: 'Invalid provider' }, 400);
  }

  let yaml: string;

  switch (provider.id) {
    case 'github-actions':
      yaml = generateGitHubActionsYaml(selectedLanguages, body.options);
      break;
    case 'tekton':
      yaml = generateTektonYaml(selectedLanguages, body.options);
      break;
    case 'gitlab-ci':
      yaml = generateGitLabCIYaml(selectedLanguages, body.options);
      break;
    case 'circleci':
      yaml = generateCircleCIYaml(selectedLanguages, body.options);
      break;
    default:
      return c.json({ error: 'Unsupported provider' }, 400);
  }

  return c.json({
    configFile: provider.configFile,
    yaml,
    languages: selectedLanguages.map((l) => l.name),
    provider: provider.name,
  });
});

export default app;
