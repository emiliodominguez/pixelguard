import { j as a } from "./jsx-runtime-CDt2p4po.js";
import "./index-GiUgBvb1.js";
const r = ({ variant: I = "default", size: O = "medium", children: R }) => {
	const T = ["badge", `badge--${I}`, `badge--${O}`].join(" ");
	return a.jsx("span", { className: T, children: R });
};
r.__docgenInfo = {
	description: "Badge component for status indicators and labels.",
	methods: [],
	displayName: "Badge",
	props: {
		variant: { defaultValue: { value: '"default"', computed: !1 }, required: !1 },
		size: { defaultValue: { value: '"medium"', computed: !1 }, required: !1 },
	},
};
const G = {
		title: "Components/Badge",
		component: r,
		parameters: { layout: "centered" },
		tags: ["autodocs"],
		argTypes: {
			variant: { control: "select", options: ["default", "primary", "success", "warning", "danger"] },
			size: { control: "select", options: ["small", "medium", "large"] },
		},
	},
	e = { args: { children: "Default" } },
	s = { args: { variant: "primary", children: "Primary" } },
	n = { args: { variant: "success", children: "Success" } },
	i = { args: { variant: "warning", children: "Warning" } },
	t = { args: { variant: "danger", children: "Danger" } },
	c = { args: { size: "small", variant: "primary", children: "Small" } },
	d = { args: { size: "large", variant: "primary", children: "Large" } },
	o = {
		render: () =>
			a.jsxs("div", {
				style: { display: "flex", gap: "8px", flexWrap: "wrap" },
				children: [
					a.jsx(r, { variant: "default", children: "Default" }),
					a.jsx(r, { variant: "primary", children: "Primary" }),
					a.jsx(r, { variant: "success", children: "Success" }),
					a.jsx(r, { variant: "warning", children: "Warning" }),
					a.jsx(r, { variant: "danger", children: "Danger" }),
				],
			}),
	},
	l = {
		render: () =>
			a.jsxs("div", {
				style: { display: "flex", gap: "8px", flexWrap: "wrap" },
				children: [
					a.jsx(r, { variant: "success", children: "Active" }),
					a.jsx(r, { variant: "warning", children: "Pending" }),
					a.jsx(r, { variant: "danger", children: "Expired" }),
					a.jsx(r, { variant: "default", children: "Draft" }),
				],
			}),
	};
var g, p, m;
e.parameters = {
	...e.parameters,
	docs: {
		...((g = e.parameters) == null ? void 0 : g.docs),
		source: {
			originalSource: `{
  args: {
    children: "Default"
  }
}`,
			...((m = (p = e.parameters) == null ? void 0 : p.docs) == null ? void 0 : m.source),
		},
	},
};
var u, v, f;
s.parameters = {
	...s.parameters,
	docs: {
		...((u = s.parameters) == null ? void 0 : u.docs),
		source: {
			originalSource: `{
  args: {
    variant: "primary",
    children: "Primary"
  }
}`,
			...((f = (v = s.parameters) == null ? void 0 : v.docs) == null ? void 0 : f.source),
		},
	},
};
var x, h, y;
n.parameters = {
	...n.parameters,
	docs: {
		...((x = n.parameters) == null ? void 0 : x.docs),
		source: {
			originalSource: `{
  args: {
    variant: "success",
    children: "Success"
  }
}`,
			...((y = (h = n.parameters) == null ? void 0 : h.docs) == null ? void 0 : y.source),
		},
	},
};
var B, S, j;
i.parameters = {
	...i.parameters,
	docs: {
		...((B = i.parameters) == null ? void 0 : B.docs),
		source: {
			originalSource: `{
  args: {
    variant: "warning",
    children: "Warning"
  }
}`,
			...((j = (S = i.parameters) == null ? void 0 : S.docs) == null ? void 0 : j.source),
		},
	},
};
var D, w, W;
t.parameters = {
	...t.parameters,
	docs: {
		...((D = t.parameters) == null ? void 0 : D.docs),
		source: {
			originalSource: `{
  args: {
    variant: "danger",
    children: "Danger"
  }
}`,
			...((W = (w = t.parameters) == null ? void 0 : w.docs) == null ? void 0 : W.source),
		},
	},
};
var P, z, _;
c.parameters = {
	...c.parameters,
	docs: {
		...((P = c.parameters) == null ? void 0 : P.docs),
		source: {
			originalSource: `{
  args: {
    size: "small",
    variant: "primary",
    children: "Small"
  }
}`,
			...((_ = (z = c.parameters) == null ? void 0 : z.docs) == null ? void 0 : _.source),
		},
	},
};
var b, A, E;
d.parameters = {
	...d.parameters,
	docs: {
		...((b = d.parameters) == null ? void 0 : b.docs),
		source: {
			originalSource: `{
  args: {
    size: "large",
    variant: "primary",
    children: "Large"
  }
}`,
			...((E = (A = d.parameters) == null ? void 0 : A.docs) == null ? void 0 : E.source),
		},
	},
};
var L, V, q;
o.parameters = {
	...o.parameters,
	docs: {
		...((L = o.parameters) == null ? void 0 : L.docs),
		source: {
			originalSource: `{
  render: () => <div style={{
    display: "flex",
    gap: "8px",
    flexWrap: "wrap"
  }}>
            <Badge variant="default">Default</Badge>
            <Badge variant="primary">Primary</Badge>
            <Badge variant="success">Success</Badge>
            <Badge variant="warning">Warning</Badge>
            <Badge variant="danger">Danger</Badge>
        </div>
}`,
			...((q = (V = o.parameters) == null ? void 0 : V.docs) == null ? void 0 : q.source),
		},
	},
};
var N, $, C;
l.parameters = {
	...l.parameters,
	docs: {
		...((N = l.parameters) == null ? void 0 : N.docs),
		source: {
			originalSource: `{
  render: () => <div style={{
    display: "flex",
    gap: "8px",
    flexWrap: "wrap"
  }}>
            <Badge variant="success">Active</Badge>
            <Badge variant="warning">Pending</Badge>
            <Badge variant="danger">Expired</Badge>
            <Badge variant="default">Draft</Badge>
        </div>
}`,
			...((C = ($ = l.parameters) == null ? void 0 : $.docs) == null ? void 0 : C.source),
		},
	},
};
const H = ["Default", "Primary", "Success", "Warning", "Danger", "Small", "Large", "AllVariants", "StatusBadges"];
export {
	o as AllVariants,
	t as Danger,
	e as Default,
	d as Large,
	s as Primary,
	c as Small,
	l as StatusBadges,
	n as Success,
	i as Warning,
	H as __namedExportsOrder,
	G as default,
};
