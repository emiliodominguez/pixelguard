# Pixelguard Examples

This directory contains example projects demonstrating how to use Pixelguard with different frameworks.

## Available Examples

| Example | Framework | Port | Auto-Discovery |
|---------|-----------|------|----------------|
| [storybook-react](./storybook-react) | Storybook + React | 6006 | Yes (stories) |
| [nextjs-app](./nextjs-app) | Next.js 15 | 3000 | No (manual shots) |
| [vite-react](./vite-react) | Vite 6 + React 19 | 5173 | No (manual shots) |
| [astro-site](./astro-site) | Astro 5 | 4321 | No (manual shots) |

## Running an Example

Each example follows the same pattern:

```bash
# Navigate to the example directory
cd examples/<example-name>

# Install dependencies
npm install

# Start the dev server
npm run dev

# In another terminal, run Pixelguard
npx pixelguard test
```

## Storybook Example

The `storybook-react` example demonstrates Pixelguard's automatic story discovery. When you run `pixelguard init`, it will:

1. Detect the `.storybook/` directory
2. Connect to Storybook at `http://localhost:6006`
3. Fetch all stories from `/index.json`
4. Auto-generate shots for each story

```bash
cd examples/storybook-react
npm install
npm run storybook  # Start Storybook

# In another terminal
npx pixelguard init  # Auto-discovers stories
npx pixelguard test --update  # Create baseline
```

## Framework Examples

The `nextjs-app`, `vite-react`, and `astro-site` examples demonstrate manual shot configuration for non-Storybook projects.

Each includes a `pixelguard.config.json` with manually configured shots:

```json
{
  "baseUrl": "http://localhost:3000",
  "shots": [
    { "name": "home", "path": "/" },
    { "name": "about", "path": "/about" },
    { "name": "contact", "path": "/contact" }
  ]
}
```

To run these examples:

```bash
cd examples/nextjs-app  # or vite-react, astro-site
npm install
npm run dev  # Start the dev server

# In another terminal
npx pixelguard test --update  # Create baseline
```

## Creating Your Own Config

For non-Storybook projects, you can either:

1. **Use `pixelguard init`** - Detects your dev server and creates a minimal config
2. **Create manually** - Create `pixelguard.config.json` with your shots

See the [Configuration Documentation](../docs/configuration.md) for all available options.
