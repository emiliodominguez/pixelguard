import React from "react";
import "./Input.css";

/**
 * Input component for form fields.
 */
export const Input = ({
	label,
	placeholder,
	type = "text",
	error,
	disabled = false,
	value,
	onChange,
	...props
}) => {
	const className = ["input-wrapper", error && "input-wrapper--error", disabled && "input-wrapper--disabled"]
		.filter(Boolean)
		.join(" ");

	return (
		<div className={className}>
			{label && <label className="input__label">{label}</label>}
			<input
				type={type}
				className="input__field"
				placeholder={placeholder}
				disabled={disabled}
				value={value}
				onChange={onChange}
				{...props}
			/>
			{error && <span className="input__error">{error}</span>}
		</div>
	);
};
