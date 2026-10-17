// Pixelguard report behavior. Plain DOM, no dependencies, works from file:// and from `pixelguard serve`.
(function () {
	"use strict";

	const root = document.documentElement;
	const body = document.body;
	const shots = Array.from(document.querySelectorAll("[data-review]"));
	const indexLinks = Array.from(document.querySelectorAll(".index a[data-target]"));
	const searchInput = document.getElementById("search");
	const saveState = document.getElementById("save-state");
	const reviewProgress = document.getElementById("review-progress");
	const progressFill = document.getElementById("progress-fill");
	const exportButton = document.getElementById("export-decisions");

	/**
	 * Reads a value from localStorage, tolerating disabled storage.
	 *
	 * @param {string} key - Storage key
	 * @returns {string | null} Stored value
	 */
	function readStorage(key) {
		try {
			return localStorage.getItem(key);
		} catch {
			return null;
		}
	}

	/**
	 * Writes a value to localStorage, tolerating disabled storage.
	 *
	 * @param {string} key - Storage key
	 * @param {string} value - Value to store
	 */
	function writeStorage(key, value) {
		try {
			localStorage.setItem(key, value);
		} catch {
			// Storage is unavailable in some file:// contexts; the report still works without it.
		}
	}

	/**
	 * Marks the button in a segmented group that matches the value.
	 *
	 * @param {NodeListOf<HTMLButtonElement>} buttons - Group buttons
	 * @param {string} attribute - Data attribute holding each button's value
	 * @param {string} value - Selected value
	 */
	function pressOne(buttons, attribute, value) {
		buttons.forEach((button) => button.setAttribute("aria-pressed", String(button.dataset[attribute] === value)));
	}

	// Theme

	const themeButtons = document.querySelectorAll("[data-theme-choice]");

	/**
	 * Applies a theme choice: auto follows the operating system.
	 *
	 * @param {string} choice - "auto", "light" or "dark"
	 */
	function setTheme(choice) {
		if (choice === "light" || choice === "dark") {
			root.dataset.theme = choice;
		} else {
			delete root.dataset.theme;
			choice = "auto";
		}
		pressOne(themeButtons, "themeChoice", choice);
		writeStorage("pg-theme", choice);
	}

	themeButtons.forEach((button) => button.addEventListener("click", () => setTheme(button.dataset.themeChoice)));
	setTheme(readStorage("pg-theme") || "auto");

	// Building blocks for the comparison views

	/**
	 * Pixel size of a screenshot.
	 *
	 * @typedef {object} Size
	 * @property {number} width - Width in pixels
	 * @property {number} height - Height in pixels
	 */

	/**
	 * A changed area of a screenshot, in image pixels.
	 *
	 * @typedef {object} Region
	 * @property {number} x - Left edge
	 * @property {number} y - Top edge
	 * @property {number} width - Width
	 * @property {number} height - Height
	 */

	/**
	 * Creates an element with attributes and children. Text is always set as
	 * text, never parsed as HTML, so screenshot names cannot inject markup.
	 *
	 * @param {string} tag - Element name
	 * @param {Record<string, string | number | boolean | undefined>} [attributes] - Attributes; `class` and `text` are shorthands
	 * @param {(Node | string | null)[]} [children] - Child nodes or text
	 * @returns {HTMLElement} The element
	 */
	function el(tag, attributes = {}, children = []) {
		const element = document.createElement(tag);
		for (const [name, value] of Object.entries(attributes)) {
			if (value === undefined || value === false) {
				continue;
			}
			if (name === "class") {
				element.className = String(value);
			} else if (name === "text") {
				element.textContent = String(value);
			} else {
				element.setAttribute(name, value === true ? "" : String(value));
			}
		}
		for (const child of children) {
			if (child !== null) {
				element.append(child);
			}
		}
		return element;
	}

	/**
	 * Reads a "WIDTHxHEIGHT" attribute.
	 *
	 * @param {string | undefined} text - Attribute value
	 * @returns {Size | null} Size, or null when unknown
	 */
	function parseSize(text) {
		const [width, height] = (text || "").split("x").map(Number);
		return width > 0 && height > 0 ? { width, height } : null;
	}

	/**
	 * Reads the "x,y,width,height x,y,width,height" list of changed areas.
	 *
	 * @param {string | undefined} text - Attribute value
	 * @returns {Region[]} Regions in reading order
	 */
	function parseRegions(text) {
		return (text || "")
			.split(" ")
			.filter(Boolean)
			.map((part) => {
				const [x, y, width, height] = part.split(",").map(Number);
				return { x, y, width, height };
			})
			.filter((region) => region.width > 0 && region.height > 0);
	}

	/**
	 * Formats a size for a caption.
	 *
	 * @param {Size | null} size - Image size
	 * @returns {string} "W × H", or "file missing"
	 */
	function sizeText(size) {
		return size ? `${size.width} × ${size.height}` : "file missing";
	}

	/**
	 * A screenshot image that reserves its space before loading and fades in.
	 *
	 * @param {string} src - Image path
	 * @param {Size | null} size - Known size
	 * @param {string} [alt] - Alternative text
	 * @returns {HTMLImageElement} The image
	 */
	function screenshot(src, size, alt = "") {
		const image = el("img", { src, alt, loading: "lazy", decoding: "async" });
		if (size) {
			image.width = size.width;
			image.height = size.height;
		}
		image.addEventListener("load", () => image.classList.add("is-loaded"), { once: true });
		image.addEventListener(
			"error",
			() => {
				image.classList.add("is-loaded", "is-broken");
				const caption = image.closest("figure") && image.closest("figure").querySelector(".frame-size");
				if (caption) {
					caption.textContent = "file missing";
					caption.classList.add("is-flagged");
				}
			},
			{ once: true },
		);
		return image;
	}

	/**
	 * A labeled screenshot that enlarges when clicked.
	 *
	 * @param {string} label - Caption, e.g. "Baseline"
	 * @param {string} src - Image path
	 * @param {Size | null} size - Known size
	 * @param {string} name - Screenshot name
	 * @param {string} [note] - Appended to the size, e.g. "size changed"
	 * @returns {HTMLElement} The figure
	 */
	function frame(label, src, size, name, note) {
		const lower = label.toLowerCase();
		return el("figure", { class: "frame" }, [
			el("figcaption", {}, [
				el("span", { text: label }),
				el("span", { class: `frame-size${note || !size ? " is-flagged" : ""}`, text: note ? `${sizeText(size)}, ${note}` : sizeText(size) }),
			]),
			el(
				"button",
				{
					type: "button",
					class: "frame-image",
					"data-zoom": src,
					"data-label": `${name} ${lower}`,
					"aria-label": `Enlarge ${lower} of ${name}`,
				},
				[screenshot(src, size, `${label} of ${name}`)],
			),
		]);
	}

	// Comparison views. The view is global (body[data-view]); each changed shot
	// only builds it while near the viewport, so long reports stay light.

	const VIEWS = ["side", "slider", "flicker", "heatmap", "diff", "changes"];
	const viewButtons = document.querySelectorAll("[data-view-choice]");
	const compares = Array.from(document.querySelectorAll("[data-compare]"));
	const heatCell = Number(body.dataset.heatCell) || 8;
	const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
	let view = "side";

	/** @type {WeakMap<HTMLElement, object>} */
	const details = new WeakMap();

	/**
	 * What a changed shot's views need, read once from its attributes.
	 *
	 * @param {HTMLElement} compare - The shot's comparison container
	 * @returns {object} Paths, sizes, regions and the selected region
	 */
	function detailsOf(compare) {
		let data = details.get(compare);
		if (!data) {
			const baselineSize = parseSize(compare.dataset.baselineSize);
			const currentSize = parseSize(compare.dataset.currentSize);
			data = {
				name: compare.dataset.name || "",
				baseline: compare.dataset.srcBaseline,
				current: compare.dataset.srcCurrent,
				diff: compare.dataset.srcDiff,
				heatmap: compare.dataset.srcHeatmap,
				baselineSize,
				currentSize,
				sizeChanged:
					!!baselineSize && !!currentSize && (baselineSize.width !== currentSize.width || baselineSize.height !== currentSize.height),
				regions: parseRegions(compare.dataset.regions),
				region: 0,
				mounted: false,
			};
			details.set(compare, data);
		}
		return data;
	}

	/**
	 * The larger of the two screenshots, which the overlaid views are drawn at.
	 *
	 * @param {object} data - Shot details
	 * @returns {Size | null} Canvas size
	 */
	function canvasSize(data) {
		const sizes = [data.baselineSize, data.currentSize].filter(Boolean);
		if (sizes.length === 0) {
			return null;
		}
		return { width: Math.max(...sizes.map((s) => s.width)), height: Math.max(...sizes.map((s) => s.height)) };
	}

	/**
	 * Baseline and current next to each other.
	 *
	 * @param {object} data - Shot details
	 * @returns {Node[]} View content
	 */
	function sideView(data) {
		return [
			frame("Baseline", data.baseline, data.baselineSize, data.name),
			frame("Current", data.current, data.currentSize, data.name, data.sizeChanged ? "size changed" : undefined),
		];
	}

	/**
	 * Baseline under current, with a divider that reveals one or the other.
	 *
	 * @param {object} data - Shot details
	 * @returns {Node[]} View content
	 */
	function sliderView(data) {
		const slider = el(
			"div",
			{
				class: "swipe",
				tabindex: 0,
				role: "slider",
				"aria-label": `Slide between baseline and current of ${data.name}`,
				"aria-valuemin": 0,
				"aria-valuemax": 100,
				"aria-valuenow": 50,
			},
			[
				screenshot(data.baseline, data.baselineSize, `Baseline of ${data.name}`),
				el("div", { class: "swipe-current" }, [screenshot(data.current, data.currentSize, `Current of ${data.name}`)]),
				el("div", { class: "swipe-handle" }),
				el("span", { class: "swipe-label swipe-label--left", text: "Baseline" }),
				el("span", { class: "swipe-label swipe-label--right", text: "Current" }),
			],
		);
		return [slider];
	}

	/**
	 * Baseline and current stacked, alternating on a shared timer so every
	 * visible shot flickers in step. Clicking swaps by hand.
	 *
	 * @param {object} data - Shot details
	 * @returns {Node[]} View content
	 */
	function flickerView(data) {
		const size = canvasSize(data);
		const stage = el(
			"button",
			{ type: "button", class: "flicker", "aria-label": `Swap baseline and current of ${data.name}`, title: "Click to swap" },
			[
				screenshot(data.baseline, data.baselineSize, `Baseline of ${data.name}`),
				screenshot(data.current, data.currentSize, `Current of ${data.name}`),
				el("span", { class: "flicker-label flicker-label--baseline", text: "Baseline" }),
				el("span", { class: "flicker-label flicker-label--current", text: "Current" }),
			],
		);
		if (size) {
			stage.style.aspectRatio = `${size.width} / ${size.height}`;
			stage.style.width = `min(100%, ${size.width}px)`;
			// Both images are drawn at their own size on the larger canvas, top-left aligned.
			[data.baselineSize, data.currentSize].forEach((own, index) => {
				if (own) {
					stage.querySelectorAll("img")[index].style.width = `${(own.width / size.width) * 100}%`;
				}
			});
		} else {
			stage.querySelectorAll("img").forEach((image) => (image.style.position = "static"));
		}
		return [stage];
	}

	/**
	 * The current screenshot, faded, under a heatmap of where pixels changed.
	 *
	 * @param {object} data - Shot details
	 * @returns {Node[]} View content
	 */
	function heatmapView(data) {
		if (!data.heatmap || !data.currentSize) {
			return [
				el("p", {
					class: "view-note",
					text: data.sizeChanged
						? "No heatmap: the screenshots differ in size, so every pixel counts as changed."
						: "No heatmap was drawn for this screenshot.",
				}),
				frame("Diff", data.diff, canvasSize(data), data.name),
			];
		}
		const { width, height } = data.currentSize;
		const heat = el("img", { class: "heatmap-heat", src: data.heatmap, alt: "", decoding: "async" });
		// Each heatmap pixel covers a cell; the last row and column can stick out past the screenshot.
		heat.style.width = `${((Math.ceil(width / heatCell) * heatCell) / width) * 100}%`;
		heat.style.height = `${((Math.ceil(height / heatCell) * heatCell) / height) * 100}%`;
		return [
			el("div", { class: "heatmap", style: `width: min(100%, ${width}px)` }, [
				screenshot(data.current, data.currentSize, `Current of ${data.name}`),
				heat,
				el("span", { class: "swipe-label swipe-label--left", text: "Heatmap over current" }),
			]),
		];
	}

	/**
	 * Changed pixels in red over a dimmed copy of the current screenshot.
	 *
	 * @param {object} data - Shot details
	 * @returns {Node[]} View content
	 */
	function diffView(data) {
		return [frame("Diff", data.diff, canvasSize(data), data.name)];
	}

	/**
	 * A close crop of one region of a screenshot, with the changed area outlined.
	 *
	 * @param {string} label - "Baseline" or "Current"
	 * @param {string} src - Image path
	 * @param {Size} size - Image size
	 * @param {Region} region - Changed area
	 * @returns {HTMLElement} The crop
	 */
	function crop(label, src, size, region) {
		const padding = Math.max(24, Math.round(Math.max(region.width, region.height) * 0.15));
		const left = Math.max(0, region.x - padding);
		const top = Math.max(0, region.y - padding);
		const right = Math.min(size.width, region.x + region.width + padding);
		const bottom = Math.min(size.height, region.y + region.height + padding);
		const width = Math.max(1, right - left);
		const height = Math.max(1, bottom - top);

		const image = screenshot(src, size, "");
		image.style.width = `${(size.width / width) * 100}%`;
		image.style.left = `${(-left / width) * 100}%`;
		image.style.top = `${(-top / height) * 100}%`;

		const mark = el("div", { class: "crop-mark" });
		mark.style.left = `${((region.x - left) / width) * 100}%`;
		mark.style.top = `${((region.y - top) / height) * 100}%`;
		mark.style.width = `${(Math.min(region.width, size.width - region.x) / width) * 100}%`;
		mark.style.height = `${(Math.min(region.height, size.height - region.y) / height) * 100}%`;

		const viewport = el("div", { class: "crop-window" }, [image, mark]);
		viewport.style.aspectRatio = `${width} / ${height}`;
		// Small areas are enlarged up to 3×; tall ones are kept within the screen.
		viewport.style.width = `min(100%, ${width * 3}px, ${(60 * width) / height}vh)`;

		return el("figure", { class: "frame crop" }, [
			el("figcaption", {}, [
				el("span", { text: label }),
				el("span", { class: "frame-size", text: `${region.width} × ${region.height} at ${region.x}, ${region.y}` }),
			]),
			viewport,
		]);
	}

	/**
	 * One changed area at a time, baseline next to current, with a strip to
	 * jump between areas. `n` and `p` step through every area of the report.
	 *
	 * @param {object} data - Shot details
	 * @returns {Node[]} View content
	 */
	function changesView(data) {
		const regions = data.regions;
		if (regions.length === 0 || !data.baselineSize || !data.currentSize) {
			return [el("p", { class: "view-note", text: "No changed areas were located for this screenshot." }), ...diffView(data)];
		}
		data.region = Math.max(0, Math.min(regions.length - 1, data.region));
		const region = regions[data.region];

		const strip = el(
			"div",
			{ class: "regions", role: "group", "aria-label": "Changed areas" },
			regions.map((_, index) =>
				el("button", {
					type: "button",
					class: "region-chip",
					"data-region": index,
					"aria-pressed": String(index === data.region),
					"aria-label": `Changed area ${index + 1} of ${regions.length}`,
					text: index + 1,
				}),
			),
		);

		return [
			el("div", { class: "changes-head" }, [
				el("span", { class: "changes-position", text: `Area ${data.region + 1} of ${regions.length}` }),
				regions.length > 1 ? strip : null,
			]),
			el("div", { class: "changes-crops" }, [
				crop("Baseline", data.baseline, data.baselineSize, region),
				crop("Current", data.current, data.currentSize, region),
			]),
		];
	}

	const builders = { side: sideView, slider: sliderView, flicker: flickerView, heatmap: heatmapView, diff: diffView, changes: changesView };

	/**
	 * Builds the active view inside a shot's comparison container.
	 *
	 * @param {HTMLElement} compare - Comparison container
	 */
	function mount(compare) {
		const data = detailsOf(compare);
		const content = builders[view](data);
		compare.dataset.mounted = view;
		compare.replaceChildren(...content);
		compare.style.minHeight = "";
		data.mounted = true;
	}

	/**
	 * Drops a far-away shot's view, keeping its height so the page never jumps.
	 *
	 * @param {HTMLElement} compare - Comparison container
	 */
	function unmount(compare) {
		const data = detailsOf(compare);
		if (!data.mounted) {
			return;
		}
		compare.style.minHeight = `${compare.offsetHeight}px`;
		compare.replaceChildren();
		delete compare.dataset.mounted;
		data.mounted = false;
	}

	/**
	 * Estimated height of a shot's view before it is built, from the image
	 * sizes, so the scroll position and the index links land where they should.
	 *
	 * @param {object} data - Shot details
	 * @param {number} available - Width of the comparison area
	 * @returns {number} Height in pixels
	 */
	function estimateHeight(data, available) {
		const caption = 26;
		const fit = (size, width) => (size ? (Math.min(width, size.width) / size.width) * size.height : 240);
		switch (view) {
			case "side": {
				const column = (available - 16) / 2;
				return caption + Math.max(fit(data.baselineSize, column), fit(data.currentSize, column));
			}
			case "changes":
				return 360;
			case "diff":
				return caption + fit(canvasSize(data), available);
			default:
				return fit(canvasSize(data), available);
		}
	}

	/**
	 * Reserves space for every shot whose view is not built.
	 */
	function reserveSpace() {
		const reference = compares.find((compare) => compare.offsetParent !== null);
		const available = reference ? reference.clientWidth : 800;
		for (const compare of compares) {
			const data = detailsOf(compare);
			if (!data.mounted) {
				compare.style.minHeight = `${Math.round(estimateHeight(data, available))}px`;
			}
		}
	}

	const nearby = new Set();
	const proximity =
		"IntersectionObserver" in window
			? new IntersectionObserver(
					(entries) => {
						for (const entry of entries) {
							if (entry.isIntersecting) {
								nearby.add(entry.target);
								mount(entry.target);
							} else {
								nearby.delete(entry.target);
								unmount(entry.target);
							}
						}
					},
					{ rootMargin: "1500px 0px" },
				)
			: null;

	/**
	 * Switches every changed shot to a comparison view.
	 *
	 * @param {string} choice - One of VIEWS; "swipe" is the old name of "slider"
	 */
	function setView(choice) {
		if (choice === "swipe") {
			choice = "slider";
		}
		if (!VIEWS.includes(choice)) {
			choice = "side";
		}
		view = choice;
		body.dataset.view = choice;
		pressOne(viewButtons, "viewChoice", choice);
		writeStorage("pg-view", choice);

		if (proximity) {
			nearby.forEach((compare) => mount(compare));
			reserveSpace();
		} else {
			compares.forEach((compare) => mount(compare));
		}
		updateFlicker();
	}

	viewButtons.forEach((button) => button.addEventListener("click", () => setView(button.dataset.viewChoice)));

	// Slider: delegated, so views built later need no listeners of their own.

	/**
	 * Moves a slider's divider to a percentage of its width.
	 *
	 * @param {HTMLElement} slider - Slider element
	 * @param {number} percent - Divider position, 0 to 100
	 */
	function setSlider(slider, percent) {
		const clamped = Math.max(0, Math.min(100, percent));
		slider.style.setProperty("--position", `${clamped}%`);
		slider.setAttribute("aria-valuenow", String(Math.round(clamped)));
	}

	/**
	 * Moves a slider's divider to the pointer.
	 *
	 * @param {HTMLElement} slider - Slider element
	 * @param {PointerEvent} event - Pointer event
	 */
	function followPointer(slider, event) {
		const rect = slider.getBoundingClientRect();
		setSlider(slider, ((event.clientX - rect.left) / rect.width) * 100);
	}

	document.addEventListener("pointerdown", (event) => {
		const slider = event.target instanceof Element && event.target.closest(".swipe");
		if (slider) {
			slider.setPointerCapture(event.pointerId);
			followPointer(slider, event);
		}
	});
	document.addEventListener("pointermove", (event) => {
		const slider = event.target instanceof Element && event.target.closest(".swipe");
		if (slider && slider.hasPointerCapture(event.pointerId)) {
			followPointer(slider, event);
		}
	});
	document.addEventListener("keydown", (event) => {
		const slider = event.target instanceof Element && event.target.closest(".swipe");
		if (!slider || (event.key !== "ArrowLeft" && event.key !== "ArrowRight")) {
			return;
		}
		const step = (event.shiftKey ? 10 : 2) * (event.key === "ArrowLeft" ? -1 : 1);
		setSlider(slider, Number(slider.getAttribute("aria-valuenow")) + step);
		event.preventDefault();
		event.stopPropagation();
	});

	// Flicker: one timer for the whole page, running only in the flicker view.

	const flickerToggle = document.getElementById("flicker-toggle");
	const flickerSpeed = document.getElementById("flicker-speed");
	let flickerPlaying = !reducedMotion.matches;
	let flickerTimer = null;
	body.dataset.flick = "baseline";

	/**
	 * Shows the other screenshot in every flicker view.
	 */
	function swapFlicker() {
		body.dataset.flick = body.dataset.flick === "baseline" ? "current" : "baseline";
	}

	/**
	 * Starts or stops the flicker timer to match the view and the play state.
	 */
	function updateFlicker() {
		clearInterval(flickerTimer);
		flickerTimer = null;
		if (view === "flicker" && flickerPlaying) {
			flickerTimer = setInterval(swapFlicker, Number(flickerSpeed ? flickerSpeed.value : 500));
		}
		if (flickerToggle) {
			flickerToggle.textContent = flickerPlaying ? "Pause" : "Play";
			flickerToggle.setAttribute("aria-pressed", String(!flickerPlaying));
		}
	}

	/**
	 * Pauses or resumes flickering.
	 */
	function toggleFlicker() {
		flickerPlaying = !flickerPlaying;
		updateFlicker();
	}

	if (flickerToggle) {
		flickerToggle.addEventListener("click", toggleFlicker);
	}
	if (flickerSpeed) {
		flickerSpeed.value = readStorage("pg-flicker-speed") || flickerSpeed.value;
		flickerSpeed.addEventListener("change", () => {
			writeStorage("pg-flicker-speed", flickerSpeed.value);
			updateFlicker();
		});
	}

	// Clicks inside comparisons: zoom, flicker swaps and changed-area chips.

	const zoom = document.getElementById("zoom");
	const zoomBody = document.getElementById("zoom-body");
	const zoomImage = document.getElementById("zoom-image");
	const zoomLabel = document.getElementById("zoom-label");

	document.addEventListener("click", (event) => {
		const target = event.target instanceof Element ? event.target : null;
		if (!target) {
			return;
		}
		const zoomButton = target.closest("[data-zoom]");
		if (zoomButton) {
			zoomImage.src = zoomButton.dataset.zoom;
			zoomImage.alt = zoomButton.dataset.label;
			zoomLabel.textContent = zoomButton.dataset.label;
			zoomBody.classList.remove("is-actual");
			zoom.showModal();
			return;
		}
		if (target.closest(".flicker")) {
			if (flickerPlaying) {
				toggleFlicker();
			}
			swapFlicker();
			return;
		}
		const chip = target.closest("[data-region]");
		if (chip) {
			const compare = chip.closest("[data-compare]");
			detailsOf(compare).region = Number(chip.dataset.region);
			mount(compare);
			const shot = compare.closest("[data-review]");
			if (shot) {
				setCurrent(shot);
			}
			return;
		}
		const step = target.closest("[data-step-change]");
		if (step) {
			stepChange(Number(step.dataset.stepChange));
		}
	});
	zoomImage.addEventListener("click", () => zoomBody.classList.toggle("is-actual"));
	document.getElementById("zoom-close").addEventListener("click", () => zoom.close());
	zoom.addEventListener("click", (event) => {
		if (event.target === zoom) {
			zoom.close();
		}
	});

	// Filtering. Lookups are prepared once; typing is batched to one pass per frame.

	const filterButtons = document.querySelectorAll("[data-filter]");
	const indexItems = indexLinks.map((link) => ({ item: link.parentElement, shot: document.getElementById(link.dataset.target) }));
	const groups = Array.from(document.querySelectorAll("[data-group]")).map((group) => ({
		group,
		members: shots.filter((shot) => group.contains(shot)),
	}));
	const indexGroups = Array.from(document.querySelectorAll(".index-group")).map((group) => ({
		group,
		members: indexItems.filter(({ item }) => group.contains(item)),
	}));
	const unchangedItems = Array.from(document.querySelectorAll("[data-unchanged]")).map((item) => ({
		item,
		name: item.dataset.unchanged.toLowerCase(),
	}));
	const unchangedGroups = Array.from(document.querySelectorAll("[data-unchanged-group]")).map((group) => ({
		group,
		members: unchangedItems.filter(({ item }) => group.contains(item)),
	}));
	const shotNames = new Map(shots.map((shot) => [shot, shot.dataset.name.toLowerCase()]));
	const noMatches = document.getElementById("no-matches");
	let statusFilter = "all";
	let filterFrame = 0;

	/**
	 * Shows only shots matching the status filter and the search text.
	 */
	function applyFilters() {
		filterFrame = 0;
		const query = (searchInput.value || "").trim().toLowerCase();
		let visible = 0;

		for (const shot of shots) {
			const matches = (statusFilter === "all" || shot.dataset.status === statusFilter) && shotNames.get(shot).includes(query);
			if (shot.hidden === matches) {
				shot.hidden = !matches;
			}
			if (matches) {
				visible++;
			}
		}
		for (const { item, shot } of indexItems) {
			item.hidden = shot.hidden;
		}
		for (const { group, members } of groups) {
			group.hidden = !members.some((shot) => !shot.hidden);
		}
		for (const { group, members } of indexGroups) {
			group.hidden = !members.some(({ item }) => !item.hidden);
		}
		for (const { item, name } of unchangedItems) {
			item.hidden = !name.includes(query);
		}
		for (const { group, members } of unchangedGroups) {
			group.hidden = !members.some(({ item }) => !item.hidden);
		}
		if (noMatches) {
			noMatches.hidden = visible > 0 || shots.length === 0;
		}
	}

	/**
	 * Runs the filters once on the next frame, however many keystrokes arrive.
	 */
	function scheduleFilters() {
		if (!filterFrame) {
			filterFrame = requestAnimationFrame(applyFilters);
		}
	}

	filterButtons.forEach((button) =>
		button.addEventListener("click", () => {
			statusFilter = button.dataset.filter;
			pressOne(filterButtons, "filter", statusFilter);
			applyFilters();
		}),
	);
	searchInput.addEventListener("input", scheduleFilters);

	// Sorting changed shots

	const sortSelect = document.getElementById("sort");
	const changedList = document.getElementById("changed-list");

	if (sortSelect && changedList) {
		sortSelect.addEventListener("change", () => {
			const items = Array.from(changedList.children);
			items.sort((a, b) =>
				sortSelect.value === "diff" ? parseFloat(b.dataset.diff) - parseFloat(a.dataset.diff) : a.dataset.name.localeCompare(b.dataset.name),
			);
			const reordered = document.createDocumentFragment();
			items.forEach((item) => reordered.append(item));
			changedList.append(reordered);
		});
	}

	// Decisions

	let decisions = {};
	let serverMode = false;

	/**
	 * Builds the decisions document that `pixelguard apply` reads.
	 *
	 * @returns {object} Decisions file contents
	 */
	function decisionsDocument() {
		return { version: "1.0", exportedAt: new Date().toISOString(), decisions };
	}

	/**
	 * Returns the decision for a shot, ignoring decisions made on a different image.
	 *
	 * @param {HTMLElement} shot - Shot article
	 * @returns {object | undefined} The decision, if it still applies
	 */
	function decisionFor(shot) {
		const button = shot.querySelector("[data-action]");
		const decision = button && decisions[button.dataset.shot];
		return decision && decision.hash === (button.dataset.hash || "") ? decision : undefined;
	}

	const metas = new Map(Array.from(document.querySelectorAll("[data-meta-for]")).map((meta) => [meta.dataset.metaFor, meta]));
	const actionButtons = new Map(shots.map((shot) => [shot, Array.from(shot.querySelectorAll("[data-action]"))]));
	const reviewable = shots.filter((shot) => shot.querySelector("[data-action='approve']")).length;

	/**
	 * Reflects every decision in the buttons and the index.
	 */
	function renderDecisions() {
		let decided = 0;
		shots.forEach((shot) => {
			const decision = decisionFor(shot);
			actionButtons.get(shot).forEach((button) => {
				button.setAttribute("aria-pressed", String(!!decision && decision.action === button.dataset.action));
			});
			const meta = metas.get(shot.id);
			if (meta && shot.dataset.status === "changed") {
				meta.dataset.decision = decision ? decision.action : "";
				meta.textContent = decision ? (decision.action === "approve" ? "approved" : "rejected") : meta.dataset.diff;
			}
			if (decision) {
				decided++;
			}
		});

		saveState.classList.remove("is-saved");
		reviewProgress.hidden = reviewable === 0;
		reviewProgress.classList.toggle("is-complete", reviewable > 0 && decided === reviewable);
		progressFill.style.transform = `scaleX(${reviewable === 0 ? 0 : decided / reviewable})`;
		if (reviewable === 0) {
			saveState.textContent = "";
		} else if (decided === reviewable) {
			saveState.innerHTML = serverMode
				? `All ${reviewable} reviewed. Run <code>pixelguard apply</code>`
				: `All ${reviewable} reviewed. Export, then run <code>pixelguard apply</code>`;
		} else {
			saveState.textContent = `${decided} of ${reviewable} reviewed`;
		}
	}

	let savedTimer;

	/**
	 * Briefly acknowledges that a decision was stored.
	 */
	function flashSaved() {
		clearTimeout(savedTimer);
		const previous = saveState.innerHTML;
		saveState.textContent = serverMode ? "Saved to decisions.json" : "Saved in this browser";
		saveState.classList.add("is-saved");
		savedTimer = setTimeout(() => {
			saveState.innerHTML = previous;
			saveState.classList.remove("is-saved");
		}, 1400);
	}

	/**
	 * Persists decisions to the server, or to this browser when opened from disk.
	 *
	 * @returns {Promise<void>}
	 */
	async function persist() {
		if (serverMode) {
			const response = await fetch("/api/decisions", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(decisionsDocument()),
			});
			if (!response.ok) {
				saveState.textContent = "Could not save decisions. Is pixelguard serve still running?";
				return;
			}
		} else {
			writeStorage("pg-decisions", JSON.stringify(decisions));
		}
		flashSaved();
	}

	/**
	 * Toggles a decision for a shot; choosing the active action again clears it.
	 *
	 * @param {HTMLElement} shot - Shot article
	 * @param {string} action - "approve" or "reject"
	 */
	async function decide(shot, action) {
		const button = shot.querySelector(`[data-action="${action}"]`);
		if (!button) {
			return;
		}
		const current = decisionFor(shot);
		if (current && current.action === action) {
			delete decisions[button.dataset.shot];
		} else {
			decisions[button.dataset.shot] = {
				action,
				timestamp: new Date().toISOString(),
				source: "browser",
				hash: button.dataset.hash || "",
			};
		}
		renderDecisions();
		await persist();
	}

	document
		.querySelectorAll("[data-action]")
		.forEach((button) => button.addEventListener("click", () => decide(button.closest("[data-review]"), button.dataset.action)));

	exportButton.addEventListener("click", () => {
		const blob = new Blob([JSON.stringify(decisionsDocument(), null, 2)], { type: "application/json" });
		const url = URL.createObjectURL(blob);
		const link = document.createElement("a");
		link.href = url;
		link.download = "decisions.json";
		link.click();
		URL.revokeObjectURL(url);
	});

	/**
	 * Loads decisions from `pixelguard serve` when available, otherwise from this browser.
	 *
	 * @returns {Promise<void>}
	 */
	async function loadDecisions() {
		try {
			if (location.protocol === "file:") {
				throw new Error("Opened from disk; decisions stay in this browser");
			}
			const response = await fetch("/api/decisions");
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			}
			decisions = (await response.json()).decisions || {};
			serverMode = true;
			exportButton.hidden = true;
		} catch {
			try {
				decisions = JSON.parse(readStorage("pg-decisions") || "{}");
			} catch {
				decisions = {};
			}
		}
		renderDecisions();
	}

	// Keyboard review and current-shot tracking

	let current = null;

	/**
	 * Marks a shot as the one keyboard actions apply to.
	 *
	 * @param {HTMLElement | null} shot - Shot article
	 */
	function setCurrent(shot) {
		current = shot;
		shots.forEach((item) => item.classList.toggle("is-current", item === shot));
		indexLinks.forEach((link) => link.setAttribute("aria-current", String(!!shot && link.dataset.target === shot.id)));
	}

	/**
	 * Scrolls a shot to the top of the page.
	 *
	 * @param {HTMLElement} shot - Shot article
	 */
	function reveal(shot) {
		shot.scrollIntoView({ block: "start", behavior: reducedMotion.matches ? "auto" : "smooth" });
	}

	/**
	 * Moves to the next or previous visible shot.
	 *
	 * @param {number} step - 1 for next, -1 for previous
	 */
	function move(step) {
		const visible = shots.filter((shot) => !shot.hidden);
		if (visible.length === 0) {
			return;
		}
		const position = visible.indexOf(current);
		const next = visible[position === -1 ? 0 : Math.max(0, Math.min(visible.length - 1, position + step))];
		setCurrent(next);
		reveal(next);
	}

	/**
	 * Steps to the next or previous changed area across the whole report,
	 * crossing into the neighboring changed screenshot at either end.
	 *
	 * @param {number} step - 1 for next, -1 for previous
	 */
	function stepChange(step) {
		if (view !== "changes") {
			setView("changes");
		}
		const changed = shots.filter((shot) => !shot.hidden && shot.querySelector("[data-compare]"));
		if (changed.length === 0) {
			return;
		}
		let position = changed.indexOf(current);
		if (position === -1) {
			position = 0;
		} else {
			const compare = changed[position].querySelector("[data-compare]");
			const data = detailsOf(compare);
			const next = data.region + step;
			if (next >= 0 && next < data.regions.length) {
				data.region = next;
				mount(compare);
				return;
			}
			position += step;
		}
		if (position < 0 || position >= changed.length) {
			return;
		}
		const shot = changed[position];
		const data = detailsOf(shot.querySelector("[data-compare]"));
		data.region = step < 0 ? Math.max(0, data.regions.length - 1) : 0;
		setCurrent(shot);
		mount(shot.querySelector("[data-compare]"));
		reveal(shot);
	}

	document.addEventListener("keydown", (event) => {
		const target = event.target;
		if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.altKey || zoom.open) {
			return;
		}
		if (target instanceof HTMLInputElement || target instanceof HTMLSelectElement) {
			if (event.key === "Escape") {
				target.blur();
			}
			return;
		}
		const actions = {
			j: () => move(1),
			k: () => move(-1),
			a: () => current && decide(current, "approve"),
			r: () => current && decide(current, "reject"),
			n: () => stepChange(1),
			p: () => stepChange(-1),
			f: () => view === "flicker" && toggleFlicker(),
			"/": () => searchInput.focus(),
		};
		VIEWS.forEach((name, index) => {
			actions[String(index + 1)] = () => compares.length > 0 && setView(name);
		});
		const action = actions[event.key];
		if (action) {
			event.preventDefault();
			action();
		}
	});

	indexLinks.forEach((link) => link.addEventListener("click", () => setCurrent(document.getElementById(link.dataset.target))));

	if ("IntersectionObserver" in window) {
		const observer = new IntersectionObserver(
			(entries) => {
				const entered = entries.filter((entry) => entry.isIntersecting).map((entry) => entry.target);
				if (entered.length > 0) {
					setCurrent(entered[0]);
				}
			},
			{ rootMargin: "-120px 0px -60% 0px" },
		);
		shots.forEach((shot) => observer.observe(shot));
	}

	// Start: pick the saved view, reserve space for every shot, then build
	// views only as shots come near.

	setView(readStorage("pg-view") || "side");
	if (proximity) {
		compares.forEach((compare) => proximity.observe(compare));
		let resizeFrame = 0;
		window.addEventListener("resize", () => {
			cancelAnimationFrame(resizeFrame);
			resizeFrame = requestAnimationFrame(reserveSpace);
		});
	}
	applyFilters();
	loadDecisions();
})();
