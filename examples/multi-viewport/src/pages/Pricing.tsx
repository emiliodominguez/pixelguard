export function Pricing() {
	return (
		<main>
			<section className="page-header">
				<div className="container">
					<h1>Pricing</h1>
					<p>Open source and free forever</p>
				</div>
			</section>

			<section className="pricing-grid">
				<div className="pricing-card">
					<h3>Starter</h3>
					<div className="pricing-price">
						$0<span>/month</span>
					</div>
					<ul className="pricing-features">
						<li>Up to 100 screenshots</li>
						<li>Single viewport</li>
						<li>HTML reports</li>
						<li>Community support</li>
					</ul>
					<button className="btn btn-secondary" style={{ width: "100%" }}>
						Get Started
					</button>
				</div>

				<div className="pricing-card featured">
					<h3>Pro</h3>
					<div className="pricing-price">
						$0<span>/month</span>
					</div>
					<ul className="pricing-features">
						<li>Unlimited screenshots</li>
						<li>Multi-viewport testing</li>
						<li>Plugin system</li>
						<li>Priority support</li>
					</ul>
					<button className="btn btn-primary" style={{ width: "100%" }}>
						Get Started
					</button>
				</div>

				<div className="pricing-card">
					<h3>Enterprise</h3>
					<div className="pricing-price">
						$0<span>/month</span>
					</div>
					<ul className="pricing-features">
						<li>Everything in Pro</li>
						<li>Custom integrations</li>
						<li>On-premise support</li>
						<li>Dedicated support</li>
					</ul>
					<button className="btn btn-secondary" style={{ width: "100%" }}>
						Contact Us
					</button>
				</div>
			</section>
		</main>
	);
}
