import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import { Button, Card, Badge, Alert, Input } from "./components";

function App() {
	return (
		<div style={{ padding: "40px", maxWidth: "800px", margin: "0 auto" }}>
			<h1 style={{ marginBottom: "32px" }}>Pixelguard Example Components</h1>

			<section style={{ marginBottom: "40px" }}>
				<h2 style={{ marginBottom: "16px" }}>Buttons</h2>
				<div style={{ display: "flex", gap: "12px", flexWrap: "wrap" }}>
					<Button variant="primary">Primary</Button>
					<Button variant="secondary">Secondary</Button>
					<Button variant="outline">Outline</Button>
					<Button variant="danger">Danger</Button>
				</div>
			</section>

			<section style={{ marginBottom: "40px" }}>
				<h2 style={{ marginBottom: "16px" }}>Badges</h2>
				<div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
					<Badge variant="primary">Primary</Badge>
					<Badge variant="success">Success</Badge>
					<Badge variant="warning">Warning</Badge>
					<Badge variant="danger">Danger</Badge>
				</div>
			</section>

			<section style={{ marginBottom: "40px" }}>
				<h2 style={{ marginBottom: "16px" }}>Cards</h2>
				<div style={{ display: "flex", gap: "16px", flexWrap: "wrap" }}>
					<Card title="Card Title" description="This is a card description." footer={<Button size="small">Learn More</Button>} />
				</div>
			</section>

			<section style={{ marginBottom: "40px" }}>
				<h2 style={{ marginBottom: "16px" }}>Alerts</h2>
				<div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
					<Alert variant="info" title="Info">
						This is an informational message.
					</Alert>
					<Alert variant="success" title="Success">
						Operation completed successfully.
					</Alert>
				</div>
			</section>

			<section style={{ marginBottom: "40px" }}>
				<h2 style={{ marginBottom: "16px" }}>Inputs</h2>
				<div style={{ display: "flex", flexDirection: "column", gap: "16px", maxWidth: "320px" }}>
					<Input label="Email" placeholder="you@example.com" />
					<Input label="Password" type="password" placeholder="••••••••" />
				</div>
			</section>
		</div>
	);
}

ReactDOM.createRoot(document.getElementById("root")).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
