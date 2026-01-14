import { Input } from "./Input";

export default {
	title: "Components/Input",
	component: Input,
	parameters: {
		layout: "centered",
	},
	tags: ["autodocs"],
};

export const Default = {
	args: {
		placeholder: "Enter text...",
	},
};

export const WithLabel = {
	args: {
		label: "Email Address",
		placeholder: "you@example.com",
		type: "email",
	},
};

export const WithError = {
	args: {
		label: "Username",
		placeholder: "Enter username",
		error: "Username is already taken",
		value: "johndoe",
	},
};

export const Disabled = {
	args: {
		label: "Disabled Input",
		placeholder: "Cannot edit",
		disabled: true,
		value: "Disabled value",
	},
};

export const Password = {
	args: {
		label: "Password",
		type: "password",
		placeholder: "Enter password",
	},
};

export const FormExample = {
	render: () => (
		<div style={{ display: "flex", flexDirection: "column", gap: "16px", width: "320px" }}>
			<Input label="Full Name" placeholder="John Doe" />
			<Input label="Email" type="email" placeholder="john@example.com" />
			<Input label="Password" type="password" placeholder="••••••••" />
			<Input label="Username" placeholder="johndoe" error="Username must be at least 3 characters" />
		</div>
	),
};
