import { j as e } from "./jsx-runtime-CDt2p4po.js";
import "./index-GiUgBvb1.js";
const r = ({ variant: l = "info", title: d, children: Y, onClose: u }) => {
	const k = ["alert", `alert--${l}`].join(" "),
		q = { info: "ℹ️", success: "✅", warning: "⚠️", error: "❌" };
	return e.jsxs("div", {
		className: k,
		role: "alert",
		children: [
			e.jsx("span", { className: "alert__icon", children: q[l] }),
			e.jsxs("div", {
				className: "alert__content",
				children: [d && e.jsx("div", { className: "alert__title", children: d }), e.jsx("div", { className: "alert__message", children: Y })],
			}),
			u && e.jsx("button", { className: "alert__close", onClick: u, "aria-label": "Close", children: "×" }),
		],
	});
};
r.__docgenInfo = {
	description: "Alert component for displaying important messages.",
	methods: [],
	displayName: "Alert",
	props: { variant: { defaultValue: { value: '"info"', computed: !1 }, required: !1 } },
};
const V = {
		title: "Components/Alert",
		component: r,
		parameters: { layout: "padded" },
		tags: ["autodocs"],
		argTypes: { variant: { control: "select", options: ["info", "success", "warning", "error"] } },
	},
	s = { args: { variant: "success", title: "Success!!!!!", children: "Your changes have been saved successfully." } },
	a = { args: { variant: "success", title: "Success", children: "Your changes have been saved successfully." } },
	n = { args: { variant: "warning", title: "Warning", children: "Please review your settings before proceeding." } },
	t = { args: { variant: "error", title: "Error", children: "There was a problem processing your request." } },
	i = { args: { variant: "info", children: "This is an alert without a title." } },
	c = { args: { variant: "success", title: "Success", children: "Click the X to dismiss this alert.", onClose: () => alert("Alert dismissed!") } },
	o = {
		render: () =>
			e.jsxs("div", {
				style: { display: "flex", flexDirection: "column", gap: "12px", maxWidth: "500px" },
				children: [
					e.jsx(r, { variant: "info", title: "Info", children: "This is an informational message." }),
					e.jsx(r, { variant: "success", title: "Success", children: "Operation completed successfully." }),
					e.jsx(r, { variant: "warning", title: "Warning", children: "Please review before continuing." }),
					e.jsx(r, { variant: "error", title: "Error", children: "Something went wrong." }),
				],
			}),
	};
var m, p, g;
s.parameters = {
	...s.parameters,
	docs: {
		...((m = s.parameters) == null ? void 0 : m.docs),
		source: {
			originalSource: `{
  args: {
    variant: "success",
    title: "Success!!!!!",
    children: "Your changes have been saved successfully."
  }
}`,
			...((g = (p = s.parameters) == null ? void 0 : p.docs) == null ? void 0 : g.source),
		},
	},
};
var h, v, f;
a.parameters = {
	...a.parameters,
	docs: {
		...((h = a.parameters) == null ? void 0 : h.docs),
		source: {
			originalSource: `{
  args: {
    variant: "success",
    title: "Success",
    children: "Your changes have been saved successfully."
  }
}`,
			...((f = (v = a.parameters) == null ? void 0 : v.docs) == null ? void 0 : f.source),
		},
	},
};
var x, S, A;
n.parameters = {
	...n.parameters,
	docs: {
		...((x = n.parameters) == null ? void 0 : x.docs),
		source: {
			originalSource: `{
  args: {
    variant: "warning",
    title: "Warning",
    children: "Please review your settings before proceeding."
  }
}`,
			...((A = (S = n.parameters) == null ? void 0 : S.docs) == null ? void 0 : A.source),
		},
	},
};
var w, y, _;
t.parameters = {
	...t.parameters,
	docs: {
		...((w = t.parameters) == null ? void 0 : w.docs),
		source: {
			originalSource: `{
  args: {
    variant: "error",
    title: "Error",
    children: "There was a problem processing your request."
  }
}`,
			...((_ = (y = t.parameters) == null ? void 0 : y.docs) == null ? void 0 : _.source),
		},
	},
};
var b, j, W;
i.parameters = {
	...i.parameters,
	docs: {
		...((b = i.parameters) == null ? void 0 : b.docs),
		source: {
			originalSource: `{
  args: {
    variant: "info",
    children: "This is an alert without a title."
  }
}`,
			...((W = (j = i.parameters) == null ? void 0 : j.docs) == null ? void 0 : W.source),
		},
	},
};
var T, E, N;
c.parameters = {
	...c.parameters,
	docs: {
		...((T = c.parameters) == null ? void 0 : T.docs),
		source: {
			originalSource: `{
  args: {
    variant: "success",
    title: "Success",
    children: "Click the X to dismiss this alert.",
    onClose: () => alert("Alert dismissed!")
  }
}`,
			...((N = (E = c.parameters) == null ? void 0 : E.docs) == null ? void 0 : N.source),
		},
	},
};
var C, D, P;
o.parameters = {
	...o.parameters,
	docs: {
		...((C = o.parameters) == null ? void 0 : C.docs),
		source: {
			originalSource: `{
  render: () => <div style={{
    display: "flex",
    flexDirection: "column",
    gap: "12px",
    maxWidth: "500px"
  }}>
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
}`,
			...((P = (D = o.parameters) == null ? void 0 : D.docs) == null ? void 0 : P.source),
		},
	},
};
const X = ["Success", "SuccessAdd", "Warning", "Error", "WithoutTitle", "Dismissible", "AllVariants"];
export {
	o as AllVariants,
	c as Dismissible,
	t as Error,
	s as Success,
	a as SuccessAdd,
	n as Warning,
	i as WithoutTitle,
	X as __namedExportsOrder,
	V as default,
};
