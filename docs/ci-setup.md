# CI Setup Guide

This guide shows how to integrate Pixelguard with popular CI/CD platforms.

## GitHub Actions

### Basic Setup

Create `.github/workflows/visual-regression.yml`:

```yaml
name: Visual Regression

on:
  pull_request:
    branches: [main]

jobs:
  visual-regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install chromium

      - name: Start Storybook
        run: |
          npm run build-storybook
          npx http-server storybook-static --port 6006 &
          npx wait-on http://localhost:6006

      - name: Run visual tests
        run: npx pixelguard test --ci

      - name: Upload report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: pixelguard-report
          path: .pixelguard/
```

### With Caching

Speed up Playwright installation with caching:

```yaml
- name: Cache Playwright browsers
  uses: actions/cache@v4
  with:
    path: ~/.cache/ms-playwright
    key: playwright-${{ runner.os }}-${{ hashFiles('package-lock.json') }}

- name: Install Playwright
  run: npx playwright install chromium
```

### Updating Baseline in CI

To automatically update the baseline on main branch pushes:

```yaml
name: Update Baseline

on:
  push:
    branches: [main]

jobs:
  update-baseline:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # ... setup steps ...

      - name: Update baseline
        run: npx pixelguard test --update

      - name: Commit updated baseline
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add .pixelguard/baseline/
          git diff --staged --quiet || git commit -m "chore: update visual regression baseline"
          git push
```

## GitLab CI

Create `.gitlab-ci.yml`:

```yaml
visual-regression:
  image: mcr.microsoft.com/playwright:v1.40.0-jammy
  stage: test
  script:
    - npm ci
    - npm run build-storybook
    - npx http-server storybook-static --port 6006 &
    - npx wait-on http://localhost:6006
    - npx pixelguard test --ci
  artifacts:
    when: on_failure
    paths:
      - .pixelguard/report.html
      - .pixelguard/diff/
    expire_in: 1 week
  only:
    - merge_requests
```

## CircleCI

Create `.circleci/config.yml`:

```yaml
version: 2.1

executors:
  node:
    docker:
      - image: mcr.microsoft.com/playwright:v1.40.0-jammy

jobs:
  visual-regression:
    executor: node
    steps:
      - checkout
      - restore_cache:
          keys:
            - npm-deps-{{ checksum "package-lock.json" }}
      - run: npm ci
      - save_cache:
          key: npm-deps-{{ checksum "package-lock.json" }}
          paths:
            - node_modules
      - run:
          name: Build Storybook
          command: npm run build-storybook
      - run:
          name: Start Storybook
          command: npx http-server storybook-static --port 6006
          background: true
      - run:
          name: Wait for Storybook
          command: npx wait-on http://localhost:6006
      - run:
          name: Run visual tests
          command: npx pixelguard test --ci
      - store_artifacts:
          path: .pixelguard/
          destination: pixelguard-report

workflows:
  test:
    jobs:
      - visual-regression
```

## Jenkins

Create `Jenkinsfile`:

```groovy
pipeline {
    agent {
        docker {
            image 'mcr.microsoft.com/playwright:v1.40.0-jammy'
        }
    }

    stages {
        stage('Install') {
            steps {
                sh 'npm ci'
            }
        }

        stage('Build') {
            steps {
                sh 'npm run build-storybook'
            }
        }

        stage('Visual Regression') {
            steps {
                sh '''
                    npx http-server storybook-static --port 6006 &
                    npx wait-on http://localhost:6006
                    npx pixelguard test --ci
                '''
            }
        }
    }

    post {
        failure {
            archiveArtifacts artifacts: '.pixelguard/**', fingerprint: true
        }
    }
}
```

## Azure DevOps

Create `azure-pipelines.yml`:

```yaml
trigger:
  - main

pr:
  - main

pool:
  vmImage: 'ubuntu-latest'

steps:
  - task: NodeTool@0
    inputs:
      versionSpec: '20.x'

  - script: npm ci
    displayName: 'Install dependencies'

  - script: npx playwright install chromium
    displayName: 'Install Playwright'

  - script: npm run build-storybook
    displayName: 'Build Storybook'

  - script: |
      npx http-server storybook-static --port 6006 &
      npx wait-on http://localhost:6006
      npx pixelguard test --ci
    displayName: 'Run visual regression tests'

  - task: PublishBuildArtifacts@1
    condition: failed()
    inputs:
      pathToPublish: '.pixelguard/'
      artifactName: 'pixelguard-report'
```

## CI Mode Output

When using `--ci` flag, Pixelguard outputs JSON for machine parsing:

```json
{
  "status": "fail",
  "unchanged": 45,
  "changed": 2,
  "added": 0,
  "removed": 0,
  "report": ".pixelguard/report.html"
}
```

Exit codes:
- `0` - All tests passed (no visual differences)
- `1` - Visual differences detected

## Best Practices

### 1. Use a Consistent Environment

Use the same browser version in CI as locally:

```json
{
  "dependencies": {
    "playwright": "1.40.0"
  }
}
```

### 2. Disable Animations

In your Storybook or app, disable animations for consistent screenshots:

```css
*, *::before, *::after {
  animation-duration: 0s !important;
  animation-delay: 0s !important;
  transition-duration: 0s !important;
  transition-delay: 0s !important;
}
```

### 3. Use Deterministic Data

Replace random data with fixed values in tests:

```javascript
// In Storybook
export const Default = () => <Card date="2024-01-15" />
```

### 4. Handle Flaky Tests

Increase delay for flaky shots:

```json
{
  "shots": [
    {
      "name": "animated-component",
      "path": "/iframe.html?id=animated--default",
      "delay": 1000
    }
  ]
}
```

### 5. Parallelize Long Test Suites

Split shots across multiple jobs:

```yaml
jobs:
  visual-regression:
    strategy:
      matrix:
        shard: [1, 2, 3, 4]
    steps:
      # ... setup ...
      - run: npx pixelguard test --ci --filter "shard-${{ matrix.shard }}"
```
