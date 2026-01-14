import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { Home } from "./pages/Home";
import { Features } from "./pages/Features";
import { Pricing } from "./pages/Pricing";
import "./index.css";

createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<BrowserRouter>
			<Routes>
				<Route element={<Layout />}>
					<Route path="/" element={<Home />} />
					<Route path="/features" element={<Features />} />
					<Route path="/pricing" element={<Pricing />} />
				</Route>
			</Routes>
		</BrowserRouter>
	</StrictMode>,
);
