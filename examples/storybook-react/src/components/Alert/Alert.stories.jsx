import { Alert } from "./Alert";

export default {
	title: "Components/Alert",
	component: Alert,
	parameters: {
		layout: "padded",
	},
	tags: ["autodocs"],
	argTypes: {
		variant: {
			control: "select",
			options: ["info", "success", "warning", "error"],
		},
	},
};

export const Info = {
	args: {
		variant: "info",
		title: "Information",
		children: "This is an informational message for the user.",
	},
};

export const Success = {
	args: {
		variant: "success",
		title: "Success",
		children: "Your changes have been saved successfully.",
	},
};

export const Warning = {
	args: {
		variant: "warning",
		title: "Warning",
		children: "Please review your settings before proceeding.",
	},
};

export const Error = {
	args: {
		variant: "error",
		title: "Error",
		children: "There was a problem processing your request.",
	},
};

export const WithoutTitle = {
	args: {
		variant: "info",
		children: "This is an alert without a title.",
	},
};

export const Dismissible = {
	args: {
		variant: "success",
		title: "Success",
		children: "Click the X to dismiss this alert.",
		onClose: () => alert("Alert dismissed!"),
	},
};

export const AllVariants = {
	render: () => (
		<div style={{ display: "flex", flexDirection: "column", gap: "12px", maxWidth: "500px" }}>
			<Alert variant="info" title="Info">
				This is an informational message.
			</Alert>
			<Alert variant="success" title="Success">
				Operation completed successfully.
			</Alert>
			<Alert variant="warning" title="Warning">
				Please review before continuing.
			</Alert>
			<Alert variant="error" title="Error">
				Something went wrong.
			</Alert>
		</div>
	),
};
