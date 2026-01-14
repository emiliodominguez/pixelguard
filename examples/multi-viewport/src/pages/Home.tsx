import { Link } from "react-router-dom";

export function Home() {
	return (
		<main>
			<section className="hero">
				<div className="container">
					<h1>Visual Regression Testing Made Simple</h1>
					<p>
						Catch visual bugs before they reach production. Pixelguard
						automatically captures screenshots across multiple viewports and
						compares them against your baseline.
					</p>
					<div className="hero-buttons">
						<Link to="/features" className="btn btn-primary">
							Explore Features
						</Link>
						<Link to="/pricing" className="btn btn-secondary">
							View Pricing
						</Link>
					</div>
				</div>
			</section>

			<section className="features-grid container">
				<div className="feature-card">
					<div className="feature-icon">📱</div>
					<h3>Multi-Viewport Testing</h3>
					<p>
						Test your UI across desktop, tablet, and mobile viewports in a
						single command.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">⚡</div>
					<h3>Lightning Fast</h3>
					<p>
						Written in Rust for maximum performance. Parallel screenshot
						capture.
					</p>
				</div>
				<div className="feature-card">
					<div className="feature-icon">🔧</div>
					<h3>Zero Config</h3>
					<p>
						Auto-detects your project type and configures itself automatically.
					</p>
				</div>
			</section>
		</main>
	);
}
