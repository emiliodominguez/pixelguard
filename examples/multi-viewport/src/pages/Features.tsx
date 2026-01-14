export function Features() {
	return (
		<main>
			<section className="page-header">
				<div className="container">
					<h1>Features</h1>
					<p>Everything you need for visual regression testing</p>
				</div>
			</section>

			<section className="features-grid container">
				<div className="feature-card">
					<div className="feature-icon">🖼️</div>
					<h3>Pixel-Perfect Comparison</h3>
					<p>
						Compare screenshots pixel-by-pixel with configurable thresholds for
						anti-aliasing tolerance.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">📊</div>
					<h3>Beautiful Reports</h3>
					<p>
						Generate static HTML reports with side-by-side comparisons and
						visual diffs.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">🔌</div>
					<h3>Plugin System</h3>
					<p>
						Extend functionality with plugins for storage, notifications, and
						custom reporters.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">🚀</div>
					<h3>CI/CD Ready</h3>
					<p>
						Built for continuous integration with JSON output and exit codes.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">📖</div>
					<h3>Storybook Support</h3>
					<p>
						Automatic story discovery for Storybook projects. Zero manual
						configuration.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">🎯</div>
					<h3>Smart Detection</h3>
					<p>
						Auto-detects Next.js, Vite, and other frameworks with sensible
						defaults.
					</p>
				</div>
			</section>
		</main>
	);
}
