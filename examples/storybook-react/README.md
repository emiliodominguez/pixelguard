# Pixelguard Storybook React Example

This is an example project demonstrating how to use Pixelguard with a Storybook React application.

## Project Structure

```
storybook-react/
├── .storybook/
│   ├── main.js           # Storybook configuration
│   └── preview.js        # Storybook preview settings
├── src/
│   ├── components/       # React components
│   │   ├── Alert/
│   │   ├── Badge/
│   │   ├── Button/
│   │   ├── Card/
│   │   └── Input/
│   └── styles.css        # Global styles
├── pixelguard.config.json # Pixelguard configuration
└── package.json
```

## Getting Started

### 1. Install Dependencies

```bash
npm install
```

### 2. Install Playwright Browsers

```bash
npx playwright install chromium
```

### 3. Start Storybook

```bash
npm run storybook
```

Storybook will be available at http://localhost:6006

### 4. Run Visual Regression Tests

In a new terminal (while Storybook is running):

```bash
# First run - create baseline
npm run pixelguard:update

# Subsequent runs - compare against baseline
npm run pixelguard:test
```

### 5. View the Report

If there are visual differences, open the report:

```bash
open .pixelguard/report.html
```

## Components Included

### Button
- Primary, Secondary, Outline, Danger variants
- Small, Medium, Large sizes
- Disabled state

### Card
- Default, Elevated, Outlined variants
- Optional image, footer support

### Badge
- Default, Primary, Success, Warning, Danger variants
- Small, Medium, Large sizes

### Alert
- Info, Success, Warning, Error variants
- Optional title and dismiss button

### Input
- Label support
- Error state
- Disabled state

## Available Scripts

| Script | Description |
|--------|-------------|
| `npm run storybook` | Start Storybook dev server |
| `npm run build-storybook` | Build static Storybook |
| `npm run pixelguard:init` | Initialize Pixelguard config |
| `npm run pixelguard:test` | Run visual regression tests |
| `npm run pixelguard:update` | Update baseline screenshots |
| `npm run pixelguard:list` | List configured shots |

## CI Integration

See the example workflow at `../../.github/workflows/visual-regression.yml` for GitHub Actions integration.

## Customizing

### Adding New Stories

1. Create a new component in `src/components/`
2. Add a `.stories.jsx` file for your component
3. Run `npx pixelguard init --force` to rediscover stories, or manually add shots to `pixelguard.config.json`

### Adjusting Threshold

If you're getting false positives, increase the threshold in `pixelguard.config.json`:

```json
{
  "threshold": 0.5
}
```

### Changing Viewport

Modify the viewport in `pixelguard.config.json`:

```json
{
  "viewport": {
    "width": 1920,
    "height": 1080
  }
}
```
