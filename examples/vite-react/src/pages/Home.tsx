import { Link } from "react-router-dom";

export function Home() {
	return (
		<main
			style={{
				display: "flex",
				flexDirection: "column",
				alignItems: "center",
				justifyContent: "center",
				minHeight: "100vh",
				padding: "2rem",
				background: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
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
				Welcome to Pixelguard
			</h1>
			<p
				style={{
					fontSize: "1.25rem",
					color: "rgba(255,255,255,0.9)",
					maxWidth: "600px",
					textAlign: "center",
					lineHeight: 1.6,
				}}
			>
				Visual regression testing for your Vite + React application. This is an example page that can be captured and compared.
			</p>
			<div style={{ display: "flex", gap: "1rem", marginTop: "2rem" }}>
				<Link
					to="/about"
					style={{
						padding: "0.75rem 1.5rem",
						background: "white",
						color: "#667eea",
						borderRadius: "8px",
						textDecoration: "none",
						fontWeight: 600,
					}}
				>
					About
				</Link>
				<Link
					to="/contact"
					style={{
						padding: "0.75rem 1.5rem",
						background: "rgba(255,255,255,0.2)",
						color: "white",
						borderRadius: "8px",
						textDecoration: "none",
						fontWeight: 600,
						border: "2px solid white",
					}}
				>
					Contact
				</Link>
			</div>
		</main>
	);
}
