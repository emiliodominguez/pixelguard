import React from "react";
import "./Alert.css";

/**
 * Alert component for displaying important messages.
 */
export const Alert = ({ variant = "info", title, children, onClose }) => {
	const className = ["alert", `alert--${variant}`].join(" ");

	const icons = {
		info: "ℹ️",
		success: "✅",
		warning: "⚠️",
		error: "❌",
	};

	return (
		<div className={className} role="alert">
			<span className="alert__icon">{icons[variant]}</span>
			<div className="alert__content">
				{title && <div className="alert__title">{title}</div>}
				<div className="alert__message">{children}</div>
			</div>
			{onClose && (
				<button className="alert__close" onClick={onClose} aria-label="Close">
					×
				</button>
			)}
		</div>
	);
};
