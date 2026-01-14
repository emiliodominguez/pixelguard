export default function Contact() {
	return (
		<main
			style={{
				display: "flex",
				flexDirection: "column",
				alignItems: "center",
				justifyContent: "center",
				minHeight: "100vh",
				padding: "2rem",
				background: "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)",
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
				Contact Us
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
				Have questions or feedback? We&apos;d love to hear from you.
			</p>
			<form
				style={{
					display: "flex",
					flexDirection: "column",
					gap: "1rem",
					width: "100%",
					maxWidth: "400px",
				}}
			>
				<input
					type="text"
					placeholder="Your Name"
					style={{
						padding: "0.75rem 1rem",
						borderRadius: "8px",
						border: "none",
						fontSize: "1rem",
					}}
				/>
				<input
					type="email"
					placeholder="Your Email"
					style={{
						padding: "0.75rem 1rem",
						borderRadius: "8px",
						border: "none",
						fontSize: "1rem",
					}}
				/>
				<textarea
					placeholder="Your Message"
					rows={4}
					style={{
						padding: "0.75rem 1rem",
						borderRadius: "8px",
						border: "none",
						fontSize: "1rem",
						resize: "vertical",
					}}
				/>
				<button
					type="submit"
					style={{
						padding: "0.75rem 1.5rem",
						background: "white",
						color: "#f5576c",
						borderRadius: "8px",
						border: "none",
						fontSize: "1rem",
						fontWeight: 600,
						cursor: "pointer",
					}}
				>
					Send Message
				</button>
			</form>
			<a
				href="/"
				style={{
					marginTop: "2rem",
					color: "white",
					textDecoration: "underline",
				}}
			>
				Back to Home
			</a>
		</main>
	);
}
