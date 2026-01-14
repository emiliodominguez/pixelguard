import { j as e } from "./jsx-runtime-CDt2p4po.js";
import "./index-GiUgBvb1.js";
import { B as d } from "./Button-e_QiuBfF.js";
const S = ({ title: i, description: c, image: l, footer: p, variant: E = "default", children: W }) => {
	const I = ["card", `card--${E}`].join(" ");
	return e.jsxs("div", {
		className: I,
		children: [
			l && e.jsx("div", { className: "card__image", children: e.jsx("img", { src: l, alt: i || "Card image" }) }),
			e.jsxs("div", {
				className: "card__content",
				children: [
					i && e.jsx("h3", { className: "card__title", children: i }),
					c && e.jsx("p", { className: "card__description", children: c }),
					W,
				],
			}),
			p && e.jsx("div", { className: "card__footer", children: p }),
		],
	});
};
S.__docgenInfo = {
	description: "Card component for displaying content in a contained box.",
	methods: [],
	displayName: "Card",
	props: { variant: { defaultValue: { value: '"default"', computed: !1 }, required: !1 } },
};
const q = {
		title: "Components/Card",
		component: S,
		parameters: { layout: "centered" },
		tags: ["autodocs"],
		argTypes: { variant: { control: "select", options: ["default", "elevated", "outlined"] } },
	},
	a = { args: { title: "Card Title", description: "This is a card description that provides more context about the card content." } },
	t = {
		args: {
			title: "Mountain Landscape",
			description: "Beautiful view of the mountains at sunset.",
			image: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=225&fit=crop",
		},
	},
	s = {
		args: {
			title: "Card with Footer",
			description: "This card has a footer section for actions.",
			footer: e.jsxs("div", {
				style: { display: "flex", gap: "8px" },
				children: [e.jsx(d, { size: "small", children: "Learn More" }), e.jsx(d, { size: "small", variant: "outline", children: "Share" })],
			}),
		},
	},
	r = { args: { variant: "elevated", title: "Elevated Card", description: "This card has a more prominent shadow." } },
	n = { args: { variant: "outlined", title: "Outlined Card", description: "This card has a visible border instead of shadow." } },
	o = {
		args: {
			variant: "elevated",
			title: "Product Name",
			description: "High-quality product with amazing features and benefits.",
			image: "https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=400&h=225&fit=crop",
			footer: e.jsxs("div", {
				style: { display: "flex", justifyContent: "space-between", alignItems: "center" },
				children: [
					e.jsx("span", { style: { fontWeight: "600", color: "#1f2937" }, children: "$99.00" }),
					e.jsx(d, { size: "small", children: "Add to Cart" }),
				],
			}),
		},
	};
var m, u, h;
a.parameters = {
	...a.parameters,
	docs: {
		...((m = a.parameters) == null ? void 0 : m.docs),
		source: {
			originalSource: `{
  args: {
    title: "Card Title",
    description: "This is a card description that provides more context about the card content."
  }
}`,
			...((h = (u = a.parameters) == null ? void 0 : u.docs) == null ? void 0 : h.source),
		},
	},
};
var f, g, v;
t.parameters = {
	...t.parameters,
	docs: {
		...((f = t.parameters) == null ? void 0 : f.docs),
		source: {
			originalSource: `{
  args: {
    title: "Mountain Landscape",
    description: "Beautiful view of the mountains at sunset.",
    image: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=225&fit=crop"
  }
}`,
			...((v = (g = t.parameters) == null ? void 0 : g.docs) == null ? void 0 : v.source),
		},
	},
};
var x, C, j;
s.parameters = {
	...s.parameters,
	docs: {
		...((x = s.parameters) == null ? void 0 : x.docs),
		source: {
			originalSource: `{
  args: {
    title: "Card with Footer",
    description: "This card has a footer section for actions.",
    footer: <div style={{
      display: "flex",
      gap: "8px"
    }}>
                <Button size="small">Learn More</Button>
                <Button size="small" variant="outline">
                    Share
                </Button>
            </div>
  }
}`,
			...((j = (C = s.parameters) == null ? void 0 : C.docs) == null ? void 0 : j.source),
		},
	},
};
var y, b, w;
r.parameters = {
	...r.parameters,
	docs: {
		...((y = r.parameters) == null ? void 0 : y.docs),
		source: {
			originalSource: `{
  args: {
    variant: "elevated",
    title: "Elevated Card",
    description: "This card has a more prominent shadow."
  }
}`,
			...((w = (b = r.parameters) == null ? void 0 : b.docs) == null ? void 0 : w.source),
		},
	},
};
var _, T, B;
n.parameters = {
	...n.parameters,
	docs: {
		...((_ = n.parameters) == null ? void 0 : _.docs),
		source: {
			originalSource: `{
  args: {
    variant: "outlined",
    title: "Outlined Card",
    description: "This card has a visible border instead of shadow."
  }
}`,
			...((B = (T = n.parameters) == null ? void 0 : T.docs) == null ? void 0 : B.source),
		},
	},
};
var N, z, F;
o.parameters = {
	...o.parameters,
	docs: {
		...((N = o.parameters) == null ? void 0 : N.docs),
		source: {
			originalSource: `{
  args: {
    variant: "elevated",
    title: "Product Name",
    description: "High-quality product with amazing features and benefits.",
    image: "https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=400&h=225&fit=crop",
    footer: <div style={{
      display: "flex",
      justifyContent: "space-between",
      alignItems: "center"
    }}>
                <span style={{
        fontWeight: "600",
        color: "#1f2937"
      }}>$99.00</span>
                <Button size="small">Add to Cart</Button>
            </div>
  }
}`,
			...((F = (z = o.parameters) == null ? void 0 : z.docs) == null ? void 0 : F.source),
		},
	},
};
const $ = ["Default", "WithImage", "WithFooter", "Elevated", "Outlined", "FullFeatured"];
export { a as Default, r as Elevated, o as FullFeatured, n as Outlined, s as WithFooter, t as WithImage, $ as __namedExportsOrder, q as default };
