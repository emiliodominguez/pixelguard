import React from "react";
import "./Card.css";

/**
 * Card component for displaying content in a contained box.
 */
export const Card = ({ title, description, image, footer, variant = "default", children }) => {
	const className = ["card", `card--${variant}`].join(" ");

	return (
		<div className={className}>
			{image && (
				<div className="card__image">
					<img src={image} alt={title || "Card image"} />
				</div>
			)}
			<div className="card__content">
				{title && <h3 className="card__title">{title}</h3>}
				{description && <p className="card__description">{description}</p>}
				{children}
			</div>
			{footer && <div className="card__footer">{footer}</div>}
		</div>
	);
};
