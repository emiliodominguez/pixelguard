import { Badge } from "./Badge";

export default {
	title: "Components/Badge",
	component: Badge,
	parameters: {
		layout: "centered",
	},
	tags: ["autodocs"],
	argTypes: {
		variant: {
			control: "select",
			options: ["default", "primary", "success", "warning", "danger"],
		},
		size: {
			control: "select",
			options: ["small", "medium", "large"],
		},
	},
};

export const Default = {
	args: {
		children: "Default",
	},
};

export const Primary = {
	args: {
		variant: "primary",
		children: "Primary",
	},
};

export const Success = {
	args: {
		variant: "success",
		children: "Success",
	},
};

export const Warning = {
	args: {
		variant: "warning",
		children: "Warning",
	},
};

export const Danger = {
	args: {
		variant: "danger",
		children: "Danger",
	},
};

export const Small = {
	args: {
		size: "small",
		variant: "primary",
		children: "Small",
	},
};

export const Large = {
	args: {
		size: "large",
		variant: "primary",
		children: "Large",
	},
};

export const AllVariants = {
	render: () => (
		<div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
			<Badge variant="default">Default</Badge>
			<Badge variant="primary">Primary</Badge>
			<Badge variant="success">Success</Badge>
			<Badge variant="warning">Warning</Badge>
			<Badge variant="danger">Danger</Badge>
		</div>
	),
};

export const StatusBadges = {
	render: () => (
		<div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
			<Badge variant="success">Active</Badge>
			<Badge variant="warning">Pending</Badge>
			<Badge variant="danger">Expired</Badge>
			<Badge variant="default">Draft</Badge>
		</div>
	),
};
