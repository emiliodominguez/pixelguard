import type { Metadata } from "next";

export const metadata: Metadata = {
	title: "Pixelguard Next.js Example",
	description: "Example Next.js app for visual regression testing",
};

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html lang="en">
			<body style={{ margin: 0, fontFamily: "system-ui, sans-serif" }}>{children}</body>
		</html>
	);
}
