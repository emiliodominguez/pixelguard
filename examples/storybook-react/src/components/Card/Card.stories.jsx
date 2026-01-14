import { Card } from "./Card";
import { Button } from "../Button";

export default {
	title: "Components/Card",
	component: Card,
	parameters: {
		layout: "centered",
	},
	tags: ["autodocs"],
	argTypes: {
		variant: {
			control: "select",
			options: ["default", "elevated", "outlined"],
		},
	},
};

export const Default = {
	args: {
		title: "Card Title",
		description: "This is a card description that provides more context about the card content.",
	},
};

export const WithImage = {
	args: {
		title: "Mountain Landscape",
		description: "Beautiful view of the mountains at sunset.",
		image: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=225&fit=crop",
	},
};

export const WithFooter = {
	args: {
		title: "Card with Footer",
		description: "This card has a footer section for actions.",
		footer: (
			<div style={{ display: "flex", gap: "8px" }}>
				<Button size="small">Learn More</Button>
				<Button size="small" variant="outline">
					Share
				</Button>
			</div>
		),
	},
};

export const Elevated = {
	args: {
		variant: "elevated",
		title: "Elevated Card",
		description: "This card has a more prominent shadow.",
	},
};

export const Outlined = {
	args: {
		variant: "outlined",
		title: "Outlined Card",
		description: "This card has a visible border instead of shadow.",
	},
};

export const FullFeatured = {
	args: {
		variant: "elevated",
		title: "Product Name",
		description: "High-quality product with amazing features and benefits.",
		image: "https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=400&h=225&fit=crop",
		footer: (
			<div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
				<span style={{ fontWeight: "600", color: "#1f2937" }}>$99.00</span>
				<Button size="small">Add to Cart</Button>
			</div>
		),
	},
};
