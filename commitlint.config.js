/**
 * @fileoverview Commitlint configuration for Pixelguard.
 *
 * Enforces conventional commit format:
 * <type>(<scope>): <description>
 *
 * Types: feat, fix, docs, refactor, test, chore, ci
 * Scopes: core, cli, npm, docs (optional)
 *
 * @see https://commitlint.js.org/
 * @see https://www.conventionalcommits.org/
 */

module.exports = {
	extends: ["@commitlint/config-conventional"],
	rules: {
		// Enforce specific types
		"type-enum": [
			2,
			"always",
			["feat", "fix", "docs", "refactor", "test", "chore", "ci", "perf", "style", "build", "revert"],
		],
		// Optional scope with specific values
		"scope-enum": [1, "always", ["core", "cli", "npm", "docs", "ci", "deps"]],
		// Subject must be lowercase
		"subject-case": [2, "always", "lower-case"],
		// No period at end of subject
		"subject-full-stop": [2, "never", "."],
		// Max header length
		"header-max-length": [2, "always", 100],
	},
};
