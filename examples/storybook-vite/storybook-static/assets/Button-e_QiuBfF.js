import { j as l } from "./jsx-runtime-CDt2p4po.js";
import "./index-GiUgBvb1.js";
const n = ({ variant: t = "primary", size: a = "medium", disabled: e = !1, children: o, onClick: r, ...s }) => {
	const u = ["btn", `btn--${t}`, `btn--${a}`, e && "btn--disabled"].filter(Boolean).join(" ");
	return l.jsx("button", { className: u, disabled: e, onClick: r, ...s, children: o });
};
n.__docgenInfo = {
	description: "Primary button component for user interactions.",
	methods: [],
	displayName: "Button",
	props: {
		variant: { defaultValue: { value: '"primary"', computed: !1 }, required: !1 },
		size: { defaultValue: { value: '"medium"', computed: !1 }, required: !1 },
		disabled: { defaultValue: { value: "false", computed: !1 }, required: !1 },
	},
};
export { n as B };
