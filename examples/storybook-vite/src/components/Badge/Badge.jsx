import React from "react";
import "./Badge.css";

/**
 * Badge component for status indicators and labels.
 */
export const Badge = ({ variant = "default", size = "medium", children }) => {
	const className = ["badge", `badge--${variant}`, `badge--${size}`].join(" ");

	return <span className={className}>{children}</span>;
};
