---
"pixelguard": minor
"@pixelguard/plugin-types": minor
---

Initial release of Pixelguard v0.1.0

Features:
- Zero-config visual regression testing for Storybook projects
- Automatic story discovery from Storybook's index.json
- Side-by-side diff reports with zoom and comparison slider
- Browser-based review workflow with auto-saving decisions
- Terminal-based interactive review (`pixelguard review`)
- Multi-viewport support for responsive testing
- Plugin system for custom storage, capture, diff, reporters, and notifiers
- CI mode with JSON output and exit codes
- Environment validation (`pixelguard validate`)

Commands:
- `pixelguard init` - Auto-detect project and create config
- `pixelguard test` - Capture and compare screenshots
- `pixelguard list` - List configured shots
- `pixelguard review` - Interactive terminal review
- `pixelguard apply` - Apply decisions from browser review
- `pixelguard serve` - Serve existing report for review
- `pixelguard validate` - Check environment prerequisites
- `pixelguard plugins` - List installed plugins
