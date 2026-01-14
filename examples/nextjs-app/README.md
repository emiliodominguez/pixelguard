# Next.js Example

A minimal Next.js 15 application demonstrating Pixelguard with manual shot configuration.

## Setup

```bash
npm install
npm run dev
```

## Using Pixelguard

```bash
# Create baseline screenshots
npx pixelguard test --update

# Run visual regression tests
npx pixelguard test
```

## Configured Shots

- `home` - Homepage (`/`)
- `about` - About page (`/about`)
- `contact` - Contact page (`/contact`)
