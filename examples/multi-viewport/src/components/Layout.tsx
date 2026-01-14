import { Link, Outlet } from "react-router-dom";

export function Layout() {
	return (
		<>
			<nav className="nav">
				<Link to="/" className="nav-logo">
					Pixelguard
				</Link>
				<ul className="nav-links">
					<li>
						<Link to="/">Home</Link>
					</li>
					<li>
						<Link to="/features">Features</Link>
					</li>
					<li>
						<Link to="/pricing">Pricing</Link>
					</li>
				</ul>
				<button className="nav-menu-btn" aria-label="Menu">
					☰
				</button>
			</nav>
			<Outlet />
			<footer className="footer">
				<p>&copy; 2025 Pixelguard. Multi-viewport testing example.</p>
			</footer>
		</>
	);
}
