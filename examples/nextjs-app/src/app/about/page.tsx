export default function About() {
	return (
		<main
			style={{
				display: "flex",
				flexDirection: "column",
				alignItems: "center",
				justifyContent: "center",
				minHeight: "100vh",
				padding: "2rem",
				background: "linear-gradient(135deg, #11998e 0%, #38ef7d 100%)",
			}}
		>
			<h1
				style={{
					fontSize: "3rem",
					color: "white",
					marginBottom: "1rem",
					textShadow: "2px 2px 4px rgba(0,0,0,0.3)",
				}}
			>
				About Pixelguard
			</h1>
			<p
				style={{
					fontSize: "1.25rem",
					color: "rgba(255,255,255,0.9)",
					maxWidth: "600px",
					textAlign: "center",
					lineHeight: 1.6,
					marginBottom: "2rem",
				}}
			>
				Pixelguard is an open-source visual regression testing tool that helps you catch unintended UI changes before they reach production.
			</p>
			<ul
				style={{
					listStyle: "none",
					padding: 0,
					display: "flex",
					flexDirection: "column",
					gap: "0.75rem",
				}}
			>
				{["Zero configuration required", "Git-friendly screenshot storage", "Beautiful HTML reports", "CI/CD integration ready"].map(
					(feature) => (
						<li
							key={feature}
							style={{
								display: "flex",
								alignItems: "center",
								gap: "0.5rem",
								color: "white",
								fontSize: "1.1rem",
							}}
						>
							<span style={{ color: "#fff", fontWeight: "bold" }}>✓</span>
							{feature}
						</li>
					),
				)}
			</ul>
			<a
				href="/"
				style={{
					marginTop: "2rem",
					padding: "0.75rem 1.5rem",
					background: "white",
					color: "#11998e",
					borderRadius: "8px",
					textDecoration: "none",
					fontWeight: 600,
				}}
			>
				Back to Home
			</a>
		</main>
	);
}
