import { j as e } from "./jsx-runtime-CDt2p4po.js";
import "./index-GiUgBvb1.js";
const a = ({ label: d, placeholder: P, type: S = "text", error: p, disabled: c = !1, value: F, onChange: W, ...C }) => {
	const k = ["input-wrapper", p && "input-wrapper--error", c && "input-wrapper--disabled"].filter(Boolean).join(" ");
	return e.jsxs("div", {
		className: k,
		children: [
			d && e.jsx("label", { className: "input__label", children: d }),
			e.jsx("input", { type: S, className: "input__field", placeholder: P, disabled: c, value: F, onChange: W, ...C }),
			p && e.jsx("span", { className: "input__error", children: p }),
		],
	});
};
a.__docgenInfo = {
	description: "Input component for form fields.",
	methods: [],
	displayName: "Input",
	props: {
		type: { defaultValue: { value: '"text"', computed: !1 }, required: !1 },
		disabled: { defaultValue: { value: "false", computed: !1 }, required: !1 },
	},
};
const J = { title: "Components/Input", component: a, parameters: { layout: "centered" }, tags: ["autodocs"] },
	r = { args: { placeholder: "Enter text..." } },
	s = { args: { label: "Email Address", placeholder: "you@example.com", type: "email" } },
	l = { args: { label: "Username", placeholder: "Enter username", error: "Username is already taken", value: "johndoe" } },
	o = { args: { label: "Disabled Input", placeholder: "Cannot edit", disabled: !0, value: "Disabled value" } },
	n = { args: { label: "Password", type: "password", placeholder: "Enter password" } },
	t = {
		render: () =>
			e.jsxs("div", {
				style: { display: "flex", flexDirection: "column", gap: "16px", width: "320px" },
				children: [
					e.jsx(a, { label: "Full Name", placeholder: "John Doe" }),
					e.jsx(a, { label: "Email", type: "email", placeholder: "john@example.com" }),
					e.jsx(a, { label: "Password", type: "password", placeholder: "••••••••" }),
					e.jsx(a, { label: "Username", placeholder: "johndoe", error: "Username must be at least 3 characters" }),
				],
			}),
	};
var m, i, u;
r.parameters = {
	...r.parameters,
	docs: {
		...((m = r.parameters) == null ? void 0 : m.docs),
		source: {
			originalSource: `{
  args: {
    placeholder: "Enter text..."
  }
}`,
			...((u = (i = r.parameters) == null ? void 0 : i.docs) == null ? void 0 : u.source),
		},
	},
};
var h, x, b;
s.parameters = {
	...s.parameters,
	docs: {
		...((h = s.parameters) == null ? void 0 : h.docs),
		source: {
			originalSource: `{
  args: {
    label: "Email Address",
    placeholder: "you@example.com",
    type: "email"
  }
}`,
			...((b = (x = s.parameters) == null ? void 0 : x.docs) == null ? void 0 : b.source),
		},
	},
};
var f, g, y;
l.parameters = {
	...l.parameters,
	docs: {
		...((f = l.parameters) == null ? void 0 : f.docs),
		source: {
			originalSource: `{
  args: {
    label: "Username",
    placeholder: "Enter username",
    error: "Username is already taken",
    value: "johndoe"
  }
}`,
			...((y = (g = l.parameters) == null ? void 0 : g.docs) == null ? void 0 : y.source),
		},
	},
};
var j, w, E;
o.parameters = {
	...o.parameters,
	docs: {
		...((j = o.parameters) == null ? void 0 : j.docs),
		source: {
			originalSource: `{
  args: {
    label: "Disabled Input",
    placeholder: "Cannot edit",
    disabled: true,
    value: "Disabled value"
  }
}`,
			...((E = (w = o.parameters) == null ? void 0 : w.docs) == null ? void 0 : E.source),
		},
	},
};
var v, D, I;
n.parameters = {
	...n.parameters,
	docs: {
		...((v = n.parameters) == null ? void 0 : v.docs),
		source: {
			originalSource: `{
  args: {
    label: "Password",
    type: "password",
    placeholder: "Enter password"
  }
}`,
			...((I = (D = n.parameters) == null ? void 0 : D.docs) == null ? void 0 : I.source),
		},
	},
};
var _, U, N;
t.parameters = {
	...t.parameters,
	docs: {
		...((_ = t.parameters) == null ? void 0 : _.docs),
		source: {
			originalSource: `{
  render: () => <div style={{
    display: "flex",
    flexDirection: "column",
    gap: "16px",
    width: "320px"
  }}>
            <Input label="Full Name" placeholder="John Doe" />
            <Input label="Email" type="email" placeholder="john@example.com" />
            <Input label="Password" type="password" placeholder="••••••••" />
            <Input label="Username" placeholder="johndoe" error="Username must be at least 3 characters" />
        </div>
}`,
			...((N = (U = t.parameters) == null ? void 0 : U.docs) == null ? void 0 : N.source),
		},
	},
};
const L = ["Default", "WithLabel", "WithError", "Disabled", "Password", "FormExample"];
export { r as Default, o as Disabled, t as FormExample, n as Password, l as WithError, s as WithLabel, L as __namedExportsOrder, J as default };
