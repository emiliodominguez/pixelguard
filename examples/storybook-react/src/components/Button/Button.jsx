import React from "react";
import "./Button.css";

/**
 * Primary button component for user interactions.
 */
export const Button = ({
	variant = "primary",
	size = "medium",
	disabled = false,
	children,
	onClick,
	...props
}) => {
	const className = ["btn", `btn--${variant}`, `btn--${size}`, disabled && "btn--disabled"]
		.filter(Boolean)
		.join(" ");

	return (
		<button className={className} disabled={disabled} onClick={onClick} {...props}>
			{children}
		</button>
	);
};
