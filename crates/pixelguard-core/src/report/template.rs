//! HTML rendering for the static report.
//!
//! The stylesheet and script live in `report.css` and `report.js` next to this
//! file and are embedded at compile time, so the report is one self-contained
//! file that works offline from `file://` as well as from `pixelguard serve`.

use std::fmt::Write;

use super::Assets;
use crate::capture::FailedShot;
use crate::diff::{ChangedShot, DiffResult};

const STYLES: &str = include_str!("report.css");
const SCRIPT: &str = include_str!("report.js");

/// Applies the saved theme before first paint so dark mode never flashes light,
/// and marks scripting as available so images can fade in once loaded.
const THEME_BOOTSTRAP: &str = r#"document.documentElement.classList.add("js");try{var t=localStorage.getItem("pg-theme");if(t==="light"||t==="dark")document.documentElement.dataset.theme=t}catch(e){}"#;

/// The Pixelguard mark: a 2x2 pixel grid where one pixel slipped out of place,
/// leaving a faint outline where it used to be. Colors come from the theme.
const LOGO: &str = r#"<svg class="logo" viewBox="0 0 32 32" width="20" height="20" fill="none" aria-hidden="true"><rect class="logo-ink" x="3" y="3" width="11" height="11" rx="2"/><rect class="logo-ink" x="17" y="3" width="11" height="11" rx="2"/><rect class="logo-ink" x="3" y="17" width="11" height="11" rx="2"/><rect class="logo-ghost" x="17.75" y="17.75" width="9.5" height="9.5" rx="1.6" stroke-width="1.5"/><rect class="logo-changed" x="20" y="20" width="11" height="11" rx="2"/></svg>"#;

/// The mark as a self-contained favicon that follows the system color scheme.
const FAVICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Cstyle%3E.i%7Bfill:%2318181b%7D.g%7Bfill:none;stroke:%2318181b;stroke-opacity:.4%7D.c%7Bfill:%23b45309%7D@media (prefers-color-scheme:dark)%7B.i%7Bfill:%23ececef%7D.g%7Bstroke:%23ececef%7D.c%7Bfill:%23e0a43a%7D%7D%3C/style%3E%3Crect class='i' x='3' y='3' width='11' height='11' rx='2'/%3E%3Crect class='i' x='17' y='3' width='11' height='11' rx='2'/%3E%3Crect class='i' x='3' y='17' width='11' height='11' rx='2'/%3E%3Crect class='g' x='17.75' y='17.75' width='9.5' height='9.5' rx='1.6' stroke-width='1.5'/%3E%3Crect class='c' x='20' y='20' width='11' height='11' rx='2'/%3E%3C/svg%3E";

/// A screenshot that needs attention, in the order it appears on the page.
enum Entry<'a> {
    Failed(&'a FailedShot),
    Changed(&'a ChangedShot),
    Added(&'a str),
    Removed(&'a str),
}

impl Entry<'_> {
    fn name(&self) -> &str {
        match self {
            Entry::Failed(failure) => &failure.name,
            Entry::Changed(shot) => &shot.name,
            Entry::Added(name) | Entry::Removed(name) => name,
        }
    }

    fn status(&self) -> &'static str {
        match self {
            Entry::Failed(_) => "failed",
            Entry::Changed(_) => "changed",
            Entry::Added(_) => "added",
            Entry::Removed(_) => "removed",
        }
    }

    /// Git-style status letter shown in the index.
    fn letter(&self) -> &'static str {
        match self {
            Entry::Failed(_) => "F",
            Entry::Changed(_) => "M",
            Entry::Added(_) => "A",
            Entry::Removed(_) => "D",
        }
    }
}

/// Group headings, in page order.
const GROUPS: [(&str, &str); 4] = [
    ("failed", "Failed"),
    ("changed", "Changed"),
    ("added", "Added"),
    ("removed", "Removed"),
];

/// Comparison views for changed shots: value, label and keyboard shortcut.
const VIEWS: [(&str, &str, &str); 6] = [
    ("side", "Side by side", "1"),
    ("slider", "Slider", "2"),
    ("flicker", "Flicker", "3"),
    ("heatmap", "Heatmap", "4"),
    ("diff", "Diff", "5"),
    ("changes", "Changes", "6"),
];

/// Renders the complete report document.
///
/// `assets.hashes` maps changed screenshot names to the SHA-256 of their
/// current image; approvals carry it so they only apply to the image that was
/// reviewed. `assets.sizes` lets every image reserve its space before loading.
pub(super) fn render(result: &DiffResult, assets: &Assets, generated_at: &str) -> String {
    let mut entries: Vec<Entry> = Vec::new();
    entries.extend(result.failed.iter().map(Entry::Failed));
    entries.extend(result.changed.iter().map(Entry::Changed));
    entries.extend(result.added.iter().map(|name| Entry::Added(name)));
    entries.extend(result.removed.iter().map(|name| Entry::Removed(name)));

    let mut html = String::with_capacity(64 * 1024);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<meta name=\"color-scheme\" content=\"light dark\">\n");
    let _ = writeln!(html, "<title>{}</title>", html_escape(&title(result)));
    let _ = writeln!(html, "<link rel=\"icon\" href=\"{}\">", FAVICON);
    let _ = writeln!(html, "<script>{}</script>", THEME_BOOTSTRAP);
    let _ = writeln!(html, "<style>\n{}</style>\n</head>", STYLES);
    let _ = writeln!(
        html,
        "<body data-view=\"side\" data-heat-cell=\"{}\">",
        crate::diff::CELL
    );

    render_topbar(&mut html);
    html.push_str("<div class=\"layout\">\n");
    render_index(&mut html, &entries);
    html.push_str("<main>\n");
    render_summary(&mut html, result, generated_at);
    render_toolbar(&mut html, result);
    render_groups(&mut html, &entries, assets);
    if !entries.is_empty() {
        html.push_str(
            "<p class=\"empty\" id=\"no-matches\" hidden>No screenshots match this filter.</p>\n",
        );
    }
    render_unchanged(&mut html, result);
    render_footer(&mut html);
    html.push_str("</main>\n</div>\n");
    render_zoom(&mut html);
    let _ = writeln!(html, "<script>\n{}</script>\n</body>\n</html>", SCRIPT);

    html
}

fn title(result: &DiffResult) -> String {
    format!("{} - Pixelguard", headline(result))
}

/// One-line outcome, e.g. "2 changed, 1 failed".
fn headline(result: &DiffResult) -> String {
    let parts: Vec<String> = [
        (result.changed.len(), "changed"),
        (result.added.len(), "added"),
        (result.removed.len(), "removed"),
        (result.failed.len(), "failed"),
    ]
    .iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{} {}", count, label))
    .collect();

    if !parts.is_empty() {
        parts.join(", ")
    } else if result.unchanged.is_empty() {
        "Nothing was compared".to_string()
    } else {
        "No visual changes".to_string()
    }
}

fn render_topbar(html: &mut String) {
    let _ = writeln!(
        html,
        "<header class=\"topbar\">\n<a class=\"wordmark\" href=\"\">{}<span>pixelguard</span></a>",
        LOGO
    );
    html.push_str(
        r#"<div class="topbar-actions">
<div class="review-progress" id="review-progress" hidden><span class="progress-track" aria-hidden="true"><span class="progress-fill" id="progress-fill"></span></span><span class="save-state" id="save-state" aria-live="polite"></span></div>
<div class="segmented" role="group" aria-label="Theme">
<button type="button" data-theme-choice="auto" aria-pressed="true">Auto</button>
<button type="button" data-theme-choice="light" aria-pressed="false">Light</button>
<button type="button" data-theme-choice="dark" aria-pressed="false">Dark</button>
</div>
<button type="button" class="button" id="export-decisions" title="Download decisions.json for pixelguard apply">Export</button>
</div>
</header>
"#,
    );
}

fn render_index(html: &mut String, entries: &[Entry]) {
    html.push_str(
        "<nav class=\"index\" aria-label=\"Screenshots to review\">\n<div class=\"index-list\">\n",
    );

    if entries.is_empty() {
        html.push_str("<p class=\"index-empty\">Nothing to review.</p>\n");
    }

    for (status, heading) in GROUPS {
        let members: Vec<(usize, &Entry)> = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.status() == status)
            .collect();
        if members.is_empty() {
            continue;
        }

        let _ = writeln!(
            html,
            "<div class=\"index-group\">\n<h2 class=\"index-heading\">{}<span class=\"count\">{}</span></h2>\n<ol>",
            heading,
            members.len(),
        );
        for (position, entry) in members {
            let meta = match entry {
                Entry::Changed(shot) => format!("{:.2}%", shot.diff_percentage),
                _ => String::new(),
            };
            let (component, story, viewport) = name_parts(entry.name());
            let mut name = html_escape(story);
            if !viewport.is_empty() {
                let _ = write!(
                    name,
                    "<span class=\"name-quiet\">@{}</span>",
                    html_escape(viewport)
                );
            }
            if !component.is_empty() {
                let _ = write!(
                    name,
                    "<span class=\"index-component\">{}</span>",
                    html_escape(component)
                );
            }
            let _ = writeln!(
                html,
                "<li><a href=\"#shot-{id}\" data-target=\"shot-{id}\" title=\"{full}\"><span class=\"status status--{status}\" title=\"{heading}\">{letter}</span><span class=\"index-name\">{name}</span><span class=\"index-meta\" data-meta-for=\"shot-{id}\" data-diff=\"{meta}\">{meta}</span></a></li>",
                id = position,
                full = html_escape(entry.name()),
                status = status,
                heading = heading,
                letter = entry.letter(),
                name = name,
                meta = meta,
            );
        }
        html.push_str("</ol>\n</div>\n");
    }
    html.push_str("</div>\n");

    if entries
        .iter()
        .any(|entry| matches!(entry, Entry::Changed(_)))
    {
        html.push_str(
            r#"<dl class="keys" aria-label="Keyboard shortcuts">
<div title="Next and previous screenshot"><dt><kbd>j</kbd><kbd>k</kbd></dt><dd>Move</dd></div>
<div title="Approve or reject the current screenshot"><dt><kbd>a</kbd><kbd>r</kbd></dt><dd>Decide</dd></div>
<div title="Switch comparison view, 1 to 6"><dt><kbd>1</kbd><kbd>6</kbd></dt><dd>View</dd></div>
<div title="Next and previous changed area"><dt><kbd>n</kbd><kbd>p</kbd></dt><dd>Areas</dd></div>
<div title="Pause or resume flicker"><dt><kbd>f</kbd></dt><dd>Flicker</dd></div>
<div title="Filter by name"><dt><kbd>/</kbd></dt><dd>Filter</dd></div>
</dl>
"#,
        );
    }

    html.push_str("</nav>\n");
}

fn render_summary(html: &mut String, result: &DiffResult, generated_at: &str) {
    let checked = result.unchanged.len()
        + result.changed.len()
        + result.added.len()
        + result.removed.len()
        + result.failed.len();

    let _ = writeln!(
        html,
        "<section class=\"summary\">\n<h1>{}</h1>\n{}<p>{} {} checked, {} unchanged. Generated {}.</p>",
        html_escape(&headline(result)),
        distribution(result, checked),
        checked,
        if checked == 1 { "screenshot" } else { "screenshots" },
        result.unchanged.len(),
        html_escape(generated_at),
    );
    if !result.changed.is_empty() {
        html.push_str(
            "<p class=\"next-step\">Approve or reject each change, then run <code>pixelguard apply</code> to update the baseline.</p>\n",
        );
    } else if !result.added.is_empty() || !result.removed.is_empty() {
        html.push_str(
            "<p class=\"next-step\">Run <code>pixelguard test --update</code> to accept added and removed screenshots.</p>\n",
        );
    }
    html.push_str("</section>\n");
}

/// A thin stacked strip showing how the run splits across statuses.
fn distribution(result: &DiffResult, checked: usize) -> String {
    if checked == 0 {
        return String::new();
    }

    let segments = [
        ("unchanged", result.unchanged.len()),
        ("changed", result.changed.len()),
        ("added", result.added.len()),
        ("removed", result.removed.len()),
        ("failed", result.failed.len()),
    ];
    let label: Vec<String> = segments
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(status, count)| format!("{} {}", count, status))
        .collect();

    let mut strip = format!(
        "<div class=\"distribution\" role=\"img\" aria-label=\"{}\">",
        label.join(", ")
    );
    for (status, count) in segments {
        if count > 0 {
            let _ = write!(
                strip,
                "<span class=\"distribution-segment segment--{status}\" style=\"flex-grow: {count}\" title=\"{count} {status}\"></span>",
                status = status,
                count = count,
            );
        }
    }
    strip.push_str("</div>\n");
    strip
}

/// A plain-language next step for failures with a recognizable cause.
fn failure_hint(error: &str) -> Option<&'static str> {
    if error.contains("waitForSelector") && error.contains("Timeout") {
        Some(
            "The waitFor selector never appeared. Check that the story renders, or adjust waitFor.",
        )
    } else if error.contains("ERR_CONNECTION_REFUSED") || error.contains("ECONNREFUSED") {
        Some("Nothing answered at baseUrl. Start the dev server or Storybook and run again.")
    } else if error.contains("page.goto") && error.contains("Timeout") {
        Some("The page never went idle within 30 seconds. Look for polling, websockets or requests that never finish.")
    } else if error.contains("Executable doesn't exist") {
        Some("The browser is missing. Run npx playwright install chromium.")
    } else if error.contains("No screenshot was produced") {
        Some("The capture step finished without writing this file.")
    } else {
        None
    }
}

fn render_toolbar(html: &mut String, result: &DiffResult) {
    html.push_str("<div class=\"toolbar\">\n<div class=\"search-field\"><input class=\"search\" id=\"search\" type=\"search\" placeholder=\"Filter by name\" aria-label=\"Filter screenshots by name\" autocomplete=\"off\"><kbd aria-hidden=\"true\">/</kbd></div>\n");

    let counts = [
        ("failed", "Failed", result.failed.len()),
        ("changed", "Changed", result.changed.len()),
        ("added", "Added", result.added.len()),
        ("removed", "Removed", result.removed.len()),
    ];
    let active: Vec<_> = counts.iter().filter(|(_, _, count)| *count > 0).collect();
    if active.len() > 1 {
        html.push_str("<div class=\"segmented\" role=\"group\" aria-label=\"Status\">\n<button type=\"button\" data-filter=\"all\" aria-pressed=\"true\">All</button>\n");
        for (status, label, count) in active {
            let _ = writeln!(
                html,
                "<button type=\"button\" data-filter=\"{}\" aria-pressed=\"false\">{}<span class=\"count\">{}</span></button>",
                status, label, count
            );
        }
        html.push_str("</div>\n");
    }

    if !result.changed.is_empty() {
        html.push_str("<div class=\"toolbar-views\">\n<div class=\"segmented\" role=\"group\" aria-label=\"Comparison view\">\n");
        for (view, label, key) in VIEWS {
            let _ = writeln!(
                html,
                "<button type=\"button\" data-view-choice=\"{view}\" aria-pressed=\"{pressed}\" title=\"{label} ({key})\">{label}</button>",
                view = view,
                label = label,
                key = key,
                pressed = view == "side",
            );
        }
        html.push_str("</div>\n");
        html.push_str(
            r#"<div class="view-options" data-view-options="flicker">
<button type="button" class="button" id="flicker-toggle" aria-pressed="false">Pause</button>
<select class="sort" id="flicker-speed" aria-label="Flicker speed">
<option value="250">Fast</option>
<option value="500" selected>Normal</option>
<option value="1000">Slow</option>
</select>
</div>
<div class="view-options" data-view-options="changes">
<button type="button" class="button" data-step-change="-1" title="Previous changed area (p)">Previous</button>
<button type="button" class="button" data-step-change="1" title="Next changed area (n)">Next area</button>
</div>
</div>
"#,
        );
    }

    html.push_str("</div>\n");
}

fn render_groups(html: &mut String, entries: &[Entry], assets: &Assets) {
    for (status, heading) in GROUPS {
        let members: Vec<(usize, &Entry)> = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.status() == status)
            .collect();
        if members.is_empty() {
            continue;
        }

        let sort = if status == "changed" && members.len() > 1 {
            r#"<select class="sort" id="sort" aria-label="Sort changed screenshots">
<option value="name">By name</option>
<option value="diff">Largest change first</option>
</select>"#
        } else {
            ""
        };
        let _ = writeln!(
            html,
            "<section class=\"group\" data-group=\"{status}\">\n<div class=\"group-head\"><h2 class=\"group-title\"><span class=\"status status--{status}\" aria-hidden=\"true\">{letter}</span>{heading}<span class=\"count\">{count}</span></h2>{sort}</div>\n<div id=\"{status}-list\">",
            status = status,
            letter = members[0].1.letter(),
            heading = heading,
            count = members.len(),
            sort = sort,
        );
        for (position, entry) in members {
            render_entry(html, position, entry, assets);
        }
        html.push_str("</div>\n</section>\n");
    }
}

fn render_entry(html: &mut String, position: usize, entry: &Entry, assets: &Assets) {
    let name = entry.name();
    let escaped = html_escape(name);
    let file = html_escape(&url_path_segment(name));
    let diff = match entry {
        Entry::Changed(shot) => shot.diff_percentage,
        _ => 0.0,
    };

    let _ = writeln!(
        html,
        "<article class=\"shot\" id=\"shot-{position}\" data-review data-name=\"{escaped}\" data-status=\"{status}\" data-diff=\"{diff}\">",
        position = position,
        escaped = escaped,
        status = entry.status(),
        diff = diff,
    );

    match entry {
        Entry::Failed(failure) => {
            let _ = writeln!(
                html,
                "<div class=\"shot-head\"><h3 class=\"shot-name\">{}</h3><span class=\"shot-meta\">Could not be captured or compared</span></div>",
                styled_name(name),
            );
            if let Some(hint) = failure_hint(&failure.error) {
                let _ = writeln!(html, "<p class=\"failure-hint\">{}</p>", hint);
            }
            let _ = writeln!(
                html,
                "<pre class=\"failure-message\">{}</pre>",
                html_escape(&failure.error)
            );
        }
        Entry::Changed(shot) => {
            let hash = html_escape(
                assets
                    .hashes
                    .get(&shot.name)
                    .map(String::as_str)
                    .unwrap_or(""),
            );
            let areas = match shot.regions.len() {
                0 => String::new(),
                1 => " in 1 area".to_string(),
                count => format!(" in {} areas", count),
            };
            let _ = writeln!(
                html,
                "<div class=\"shot-head\"><h3 class=\"shot-name\">{styled}</h3><span class=\"shot-diff\">{percent:.2}% of pixels differ{areas}</span>\n<div class=\"decision\"><button type=\"button\" class=\"button\" data-action=\"approve\" data-shot=\"{escaped}\" data-hash=\"{hash}\" aria-pressed=\"false\">Approve<kbd>a</kbd></button><button type=\"button\" class=\"button\" data-action=\"reject\" data-shot=\"{escaped}\" data-hash=\"{hash}\" aria-pressed=\"false\">Reject<kbd>r</kbd></button></div></div>",
                escaped = escaped,
                styled = styled_name(name),
                percent = shot.diff_percentage,
                areas = areas,
                hash = hash,
            );
            render_changed(html, shot, &file, &escaped, assets);
        }
        Entry::Added(name) => {
            let _ = writeln!(
                html,
                "<div class=\"shot-head\"><h3 class=\"shot-name\">{}</h3><span class=\"shot-meta\">New screenshot without a baseline</span></div>\n<div class=\"compare compare--single\">",
                styled_name(name),
            );
            render_frame(
                html,
                "Current",
                &format!("current/{}.png", file),
                &escaped,
                assets.sizes.get(&format!("current/{}", name)).copied(),
                None,
            );
            html.push_str("</div>\n");
        }
        Entry::Removed(name) => {
            let _ = writeln!(
                html,
                "<div class=\"shot-head\"><h3 class=\"shot-name\">{}</h3><span class=\"shot-meta\">No longer captured</span></div>\n<div class=\"compare compare--single\">",
                styled_name(name),
            );
            render_frame(
                html,
                "Last baseline",
                &format!("baseline/{}.png", file),
                &escaped,
                assets.sizes.get(&format!("baseline/{}", name)).copied(),
                None,
            );
            html.push_str("</div>\n");
        }
    }

    html.push_str("</article>\n");
}

/// Renders the comparison of a changed shot.
///
/// The page carries only what each view needs (image paths, sizes and change
/// regions); the script builds the chosen view for shots near the viewport and
/// drops it again far away, so a report with thousands of screenshots stays
/// light. Without scripting, the side-by-side frames in `<noscript>` show.
fn render_changed(
    html: &mut String,
    shot: &ChangedShot,
    file: &str,
    escaped: &str,
    assets: &Assets,
) {
    let baseline_size = assets
        .sizes
        .get(&format!("baseline/{}", shot.name))
        .copied();
    let current_size = assets.sizes.get(&format!("current/{}", shot.name)).copied();
    let regions: Vec<String> = shot
        .regions
        .iter()
        .map(|r| format!("{},{},{},{}", r.x, r.y, r.width, r.height))
        .collect();

    let _ = write!(
        html,
        "<div class=\"compare compare--changed\" data-compare data-name=\"{escaped}\" data-src-baseline=\"baseline/{file}.png\" data-src-current=\"current/{file}.png\" data-src-diff=\"diff/{file}.png\" data-regions=\"{regions}\"",
        escaped = escaped,
        file = file,
        regions = regions.join(" "),
    );
    if shot.heatmap_path.is_some() {
        let _ = write!(html, " data-src-heatmap=\"heatmap/{}.png\"", file);
    }
    for (attribute, size) in [
        ("baseline-size", baseline_size),
        ("current-size", current_size),
    ] {
        if let Some((width, height)) = size {
            let _ = write!(html, " data-{}=\"{}x{}\"", attribute, width, height);
        }
    }
    html.push_str(">\n<noscript><div class=\"compare\">\n");
    let mismatch = matches!((baseline_size, current_size), (Some(b), Some(c)) if b != c);
    render_frame(
        html,
        "Baseline",
        &format!("baseline/{}.png", file),
        escaped,
        baseline_size,
        None,
    );
    render_frame(
        html,
        "Current",
        &format!("current/{}.png", file),
        escaped,
        current_size,
        mismatch.then_some("size changed"),
    );
    html.push_str("</div></noscript>\n</div>\n");
}

/// Renders one labeled screenshot that enlarges when clicked.
///
/// `size` reserves the image's space before it loads; `note` is appended to
/// the size caption (for example "size changed").
fn render_frame(
    html: &mut String,
    label: &str,
    src: &str,
    escaped_name: &str,
    size: Option<(u32, u32)>,
    note: Option<&str>,
) {
    let lower = label.to_lowercase();
    let (caption, dimensions) = match size {
        Some((width, height)) => (
            match note {
                Some(note) => format!("{} × {}, {}", width, height, note),
                None => format!("{} × {}", width, height),
            },
            format!(" width=\"{}\" height=\"{}\"", width, height),
        ),
        None => ("file missing".to_string(), String::new()),
    };
    let _ = writeln!(
        html,
        "<figure class=\"frame\"><figcaption><span>{label}</span><span class=\"frame-size{flag}\">{caption}</span></figcaption><button type=\"button\" class=\"frame-image\" data-zoom=\"{src}\" data-label=\"{name} {lower}\" aria-label=\"Enlarge {lower} of {name}\"><img src=\"{src}\" alt=\"{label} of {name}\"{dimensions} loading=\"lazy\" decoding=\"async\"></button></figure>",
        label = label,
        flag = if size.is_none() || note.is_some() { " is-flagged" } else { "" },
        caption = caption,
        lower = lower,
        src = src,
        name = escaped_name,
        dimensions = dimensions,
    );
}

fn render_unchanged(html: &mut String, result: &DiffResult) {
    if result.unchanged.is_empty() {
        return;
    }

    let _ = writeln!(
        html,
        "<details class=\"group unchanged\"{}>\n<summary>Unchanged<span class=\"count\">{}</span></summary>",
        if result.changed.is_empty() && result.failed.is_empty() { " open" } else { "" },
        result.unchanged.len(),
    );
    // Storybook ids look like "components-button--primary@mobile": group them by
    // component so a long list of passing stories stays scannable.
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for name in &result.unchanged {
        let (component, story) = name.split_once("--").unwrap_or(("", name.as_str()));
        match groups.iter_mut().find(|(key, _)| *key == component) {
            Some((_, stories)) => stories.push(story),
            None => groups.push((component, vec![story])),
        }
    }

    html.push_str("<div class=\"unchanged-groups\">\n");
    for (component, stories) in &groups {
        html.push_str("<section class=\"unchanged-group\" data-unchanged-group>\n");
        if !component.is_empty() {
            let _ = writeln!(
                html,
                "<h3>{}<span class=\"count\">{}</span></h3>",
                html_escape(component),
                stories.len()
            );
        }
        html.push_str("<ul>\n");
        for story in stories {
            let full = if component.is_empty() {
                story.to_string()
            } else {
                format!("{}--{}", component, story)
            };
            let _ = writeln!(
                html,
                "<li data-unchanged=\"{}\" title=\"{}\">{}</li>",
                html_escape(&full),
                html_escape(&full),
                html_escape(story)
            );
        }
        html.push_str("</ul>\n</section>\n");
    }
    html.push_str("</div>\n</details>\n");
}

fn render_footer(html: &mut String) {
    let _ = writeln!(
        html,
        "<footer class=\"footer\">{}<span>pixelguard {}</span><a href=\"https://github.com/emiliodominguez/pixelguard#readme\">Documentation</a></footer>",
        LOGO,
        env!("CARGO_PKG_VERSION"),
    );
}

fn render_zoom(html: &mut String) {
    html.push_str(
        r#"<dialog class="zoom" id="zoom" aria-label="Enlarged screenshot">
<div class="zoom-head"><span class="zoom-label" id="zoom-label"></span><button type="button" class="button" id="zoom-close">Close</button></div>
<div class="zoom-body" id="zoom-body"><img id="zoom-image" alt=""></div>
</dialog>
"#,
    );
}

/// Splits a Storybook-style name such as "components-button--primary@mobile"
/// into component, story and viewport. Parts a name does not have are empty, so
/// a plain name like "home" is all story.
fn name_parts(name: &str) -> (&str, &str, &str) {
    let (rest, viewport) = name.rsplit_once('@').unwrap_or((name, ""));
    let (component, story) = rest.split_once("--").unwrap_or(("", rest));
    (component, story, viewport)
}

/// The full screenshot name with the story emphasized and the component and
/// viewport quieter. Selecting and copying it still gives the exact name.
fn styled_name(name: &str) -> String {
    let (component, story, viewport) = name_parts(name);
    let mut styled = String::new();
    if !component.is_empty() {
        let _ = write!(
            styled,
            "<span class=\"name-quiet\">{}--</span><wbr>",
            html_escape(component)
        );
    }
    let _ = write!(
        styled,
        "<span class=\"name-story\">{}</span>",
        html_escape(story)
    );
    if !viewport.is_empty() {
        let _ = write!(
            styled,
            "<span class=\"name-quiet\">@{}</span>",
            html_escape(viewport)
        );
    }
    styled
}

/// Percent-encodes a screenshot name for use as one URL path segment.
///
/// Keeps RFC 3986 unreserved characters (and `@`) and encodes everything else,
/// so names with spaces or non-ASCII characters still resolve to their files.
pub(super) fn url_path_segment(name: &str) -> String {
    let mut encoded = String::with_capacity(name.len());
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'@') {
            encoded.push(byte as char);
        } else {
            let _ = write!(encoded, "%{:02X}", byte);
        }
    }
    encoded
}

/// Escapes HTML special characters.
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
