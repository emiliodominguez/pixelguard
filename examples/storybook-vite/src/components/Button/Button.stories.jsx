import { Button } from "./Button";

export default {
	title: "Components/Button",
	component: Button,
	parameters: {
		layout: "centered",
	},
	tags: ["autodocs"],
	argTypes: {
		variant: {
			control: "select",
			options: ["primary", "secondary", "outline", "danger"],
		},
		size: {
			control: "select",
			options: ["small", "medium", "large"],
		},
	},
};

export const Primary = {
	args: {
		variant: "primary",
		children: "Primary Button",
	},
};

export const Secondary = {
	args: {
		variant: "secondary",
		children: "Secondary Button",
	},
};

export const Outline = {
	args: {
		variant: "outline",
		children: "Outline Button",
	},
};

export const Danger = {
	args: {
		variant: "danger",
		children: "Delete",
	},
};

export const Small = {
	args: {
		size: "small",
		children: "Small Button",
	},
};

export const Large = {
	args: {
		size: "large",
		children: "Large Button",
	},
};

export const Disabled = {
	args: {
		disabled: true,
		children: "Disabled Button",
	},
};

export const AllVariants = {
	render: () => (
		<div style={{ display: "flex", gap: "12px", flexWrap: "wrap" }}>
			<Button variant="primary">Primary</Button>
			<Button variant="secondary">Secondary</Button>
			<Button variant="outline">Outline</Button>
			<Button variant="danger">Danger</Button>
		</div>
	),
};

export const AllSizes = {
	render: () => (
		<div style={{ display: "flex", gap: "12px", alignItems: "center" }}>
			<Button size="small">Small</Button>
			<Button size="medium">Medium</Button>
			<Button size="large">Large</Button>
		</div>
	),
};
