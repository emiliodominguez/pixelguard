# Storybook + Vite Example

A Storybook 8 project built with Vite demonstrating Pixelguard's automatic story discovery.

## Setup

```bash
npm install
npx playwright install chromium
```

## Using Pixelguard

```bash
# Start Storybook
npm run storybook

# In another terminal, create baseline screenshots
npm run pixelguard:test:visual:update

# Run visual regression tests
npm run pixelguard:test:visual
```

## Auto-Discovery

Unlike other examples, Storybook projects benefit from automatic story discovery. Pixelguard fetches all stories from Storybook's `/index.json` endpoint and generates shots automatically.

## Components Included

- **Button** - Primary, Secondary, Outline, Danger variants
- **Card** - Default, Elevated, Outlined variants
- **Badge** - Default, Primary, Success, Warning, Danger variants
- **Alert** - Info, Success, Warning, Error variants
- **Input** - With label, error state, disabled state

## Plugin Testing

This example also includes a plugins config for testing the plugin system:

```bash
npm run pixelguard:test:plugins:update
npm run pixelguard:test:plugins
```
