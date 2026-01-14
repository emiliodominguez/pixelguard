import { j as r } from "./jsx-runtime-CDt2p4po.js";
import { B as e } from "./Button-e_QiuBfF.js";
import "./index-GiUgBvb1.js";
const q = {
		title: "Components/Button",
		component: e,
		parameters: { layout: "centered" },
		tags: ["autodocs"],
		argTypes: {
			variant: { control: "select", options: ["primary", "secondary", "outline", "danger"] },
			size: { control: "select", options: ["small", "medium", "large"] },
		},
	},
	a = { args: { variant: "primary", children: "Primary Button" } },
	n = { args: { variant: "secondary", children: "Secondary Button" } },
	s = { args: { variant: "outline", children: "Outline Button" } },
	t = { args: { variant: "danger", children: "Delete" } },
	o = { args: { size: "small", children: "Small Button" } },
	i = { args: { size: "large", children: "Large Button" } },
	l = { args: { disabled: !0, children: "Disabled Button" } },
	c = {
		render: () =>
			r.jsxs("div", {
				style: { display: "flex", gap: "12px", flexWrap: "wrap" },
				children: [
					r.jsx(e, { variant: "primary", children: "Primary" }),
					r.jsx(e, { variant: "secondary", children: "Secondary" }),
					r.jsx(e, { variant: "outline", children: "Outline" }),
					r.jsx(e, { variant: "danger", children: "Danger" }),
				],
			}),
	},
	d = {
		render: () =>
			r.jsxs("div", {
				style: { display: "flex", gap: "12px", alignItems: "center" },
				children: [
					r.jsx(e, { size: "small", children: "Small" }),
					r.jsx(e, { size: "medium", children: "Medium" }),
					r.jsx(e, { size: "large", children: "Large" }),
				],
			}),
	};
var m, u, p;
a.parameters = {
	...a.parameters,
	docs: {
		...((m = a.parameters) == null ? void 0 : m.docs),
		source: {
			originalSource: `{
  args: {
    variant: "primary",
    children: "Primary Button"
  }
}`,
			...((p = (u = a.parameters) == null ? void 0 : u.docs) == null ? void 0 : p.source),
		},
	},
};
var g, y, B;
n.parameters = {
	...n.parameters,
	docs: {
		...((g = n.parameters) == null ? void 0 : g.docs),
		source: {
			originalSource: `{
  args: {
    variant: "secondary",
    children: "Secondary Button"
  }
}`,
			...((B = (y = n.parameters) == null ? void 0 : y.docs) == null ? void 0 : B.source),
		},
	},
};
var h, v, x;
s.parameters = {
	...s.parameters,
	docs: {
		...((h = s.parameters) == null ? void 0 : h.docs),
		source: {
			originalSource: `{
  args: {
    variant: "outline",
    children: "Outline Button"
  }
}`,
			...((x = (v = s.parameters) == null ? void 0 : v.docs) == null ? void 0 : x.source),
		},
	},
};
var S, z, j;
t.parameters = {
	...t.parameters,
	docs: {
		...((S = t.parameters) == null ? void 0 : S.docs),
		source: {
			originalSource: `{
  args: {
    variant: "danger",
    children: "Delete"
  }
}`,
			...((j = (z = t.parameters) == null ? void 0 : z.docs) == null ? void 0 : j.source),
		},
	},
};
var D, f, O;
o.parameters = {
	...o.parameters,
	docs: {
		...((D = o.parameters) == null ? void 0 : D.docs),
		source: {
			originalSource: `{
  args: {
    size: "small",
    children: "Small Button"
  }
}`,
			...((O = (f = o.parameters) == null ? void 0 : f.docs) == null ? void 0 : O.source),
		},
	},
};
var b, L, P;
i.parameters = {
	...i.parameters,
	docs: {
		...((b = i.parameters) == null ? void 0 : b.docs),
		source: {
			originalSource: `{
  args: {
    size: "large",
    children: "Large Button"
  }
}`,
			...((P = (L = i.parameters) == null ? void 0 : L.docs) == null ? void 0 : P.source),
		},
	},
};
var A, _, w;
l.parameters = {
	...l.parameters,
	docs: {
		...((A = l.parameters) == null ? void 0 : A.docs),
		source: {
			originalSource: `{
  args: {
    disabled: true,
    children: "Disabled Button"
  }
}`,
			...((w = (_ = l.parameters) == null ? void 0 : _.docs) == null ? void 0 : w.source),
		},
	},
};
var E, I, M;
c.parameters = {
	...c.parameters,
	docs: {
		...((E = c.parameters) == null ? void 0 : E.docs),
		source: {
			originalSource: `{
  render: () => <div style={{
    display: "flex",
    gap: "12px",
    flexWrap: "wrap"
  }}>
            <Button variant="primary">Primary</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="outline">Outline</Button>
            <Button variant="danger">Danger</Button>
        </div>
}`,
			...((M = (I = c.parameters) == null ? void 0 : I.docs) == null ? void 0 : M.source),
		},
	},
};
var V, W, C;
d.parameters = {
	...d.parameters,
	docs: {
		...((V = d.parameters) == null ? void 0 : V.docs),
		source: {
			originalSource: `{
  render: () => <div style={{
    display: "flex",
    gap: "12px",
    alignItems: "center"
  }}>
            <Button size="small">Small</Button>
            <Button size="medium">Medium</Button>
            <Button size="large">Large</Button>
        </div>
}`,
			...((C = (W = d.parameters) == null ? void 0 : W.docs) == null ? void 0 : C.source),
		},
	},
};
const F = ["Primary", "Secondary", "Outline", "Danger", "Small", "Large", "Disabled", "AllVariants", "AllSizes"];
export {
	d as AllSizes,
	c as AllVariants,
	t as Danger,
	l as Disabled,
	i as Large,
	s as Outline,
	a as Primary,
	n as Secondary,
	o as Small,
	F as __namedExportsOrder,
	q as default,
};
