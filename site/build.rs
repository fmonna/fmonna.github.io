// Build script: reads the Markdown content under `content/` and emits a typed
// `content_gen.rs` module the app links against. This keeps the site's content
// (Markdown, human-edited) decoupled from its rendering (Rust/Dioxus) — the site
// recompiles automatically when content changes, with no runtime file IO.
//
// Zero new dependencies by design: the crate's build must stay offline-friendly
// (the dev environment has no crates.io access and the lockfile has no
// serde_yaml/pulldown-cmark). The YAML frontmatter subset used by the content
// files is small and hand-parsed here; the Markdown subset actually used in the
// bodies (headings, paragraphs, bullet lists, **bold**, "smart quotes") is
// converted to HTML by a tiny inline converter.
//
// Output: $OUT_DIR/content_gen.rs  (included via `include!(concat!(env!("OUT_DIR"), ...))`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let content_dir = PathBuf::from(&manifest_dir).join("content");

    // Re-run if any content file changes (or appears/disappears).
    println!(
        "cargo:rerun-if-changed={}",
        content_dir.join("README.md").display()
    );
    walk_content(&content_dir, &mut |p| {
        println!("cargo:rerun-if-changed={}", p.display());
    });

    let profile = parse_file(&content_dir.join("profile.md"));
    let skills = parse_file(&content_dir.join("skills.md"));
    let publications = parse_file(&content_dir.join("publications.md"));

    // Each collection is a directory of one-entry-per-file Markdown. They all
    // load the same way; `load_entries` also sorts newest-first by period.end
    // (NOT the `order:` field) so the site timeline matches the CV/LaTeX order
    // and adding a new job doesn't require renumbering anything. See the note
    // in tools/gen_cv.py for the rationale.
    let experience = load_entries(&content_dir, "experience");
    let education = load_entries(&content_dir, "education");
    let teaching = load_entries(&content_dir, "teaching");
    let talks = load_entries(&content_dir, "talks");
    let blog = load_entries(&content_dir, "blog");
    let portfolio = load_entries(&content_dir, "portfolio");
    let archive = load_entries(&content_dir, "archive");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("content_gen.rs");
    println!(
        "cargo:warning=content_gen: {} experience, {} education, {} teaching, {} talks, {} blog, {} portfolio, {} archive",
        experience.len(),
        education.len(),
        teaching.len(),
        talks.len(),
        blog.len(),
        portfolio.len(),
        archive.len(),
    );
    fs::write(&out_path, render_module(&Module {
        profile,
        skills,
        publications,
        experience,
        education,
        teaching,
        talks,
        blog,
        portfolio,
        archive,
    }))
    .expect("write content_gen.rs");

/// Load and date-sort one collection directory (e.g. "experience"). Each
/// `.md` file parses to an `Entry`; the vec is sorted newest-first by
/// `period.end` (null/present first), tie-broken by `period.start` desc, then
/// by filesystem order for stability. The `order:` frontmatter field is read
/// into the Entry but is NOT used for sorting — it is retained only so the
/// generated `Entry { order, .. }` literal stays populated.
fn load_entries(content_dir: &Path, subdir: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = content_dir
        .join(subdir)
        .read_dir()
        .unwrap_or_else(|e| panic!("read content/{} dir: {}", subdir, e))
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .filter_map(|e| parse_entry(&e.path()))
        .collect();
    // Stable sort by (end desc, start desc). `end` null/present sorts newest.
    entries.sort_by(|a, b| {
        let ea = period_end_key(&a.front);
        let eb = period_end_key(&b.front);
        eb.cmp(&ea)
            .then_with(|| period_start_key(&a.front).cmp(&period_start_key(&b.front)))
    });
    entries
}

/// Sort key for a period.end value. Null/absent/empty/"null" (=> current)
/// sorts newest (max); a year-month "2024-09" becomes (2024, 9); a bare year
/// "2024" becomes (2024, 0). Anything unparseable sorts oldest (0, 0).
fn period_end_key(front: &Yaml) -> (i64, i64) {
    let period = front.get("period").unwrap_or(&Yaml::Null);
    period_sort_key(period.get("end").unwrap_or(&Yaml::Null), true)
}

fn period_start_key(front: &Yaml) -> (i64, i64) {
    let period = front.get("period").unwrap_or(&Yaml::Null);
    period_sort_key(period.get("start").unwrap_or(&Yaml::Null), false)
}

/// `(year, month)` sort key for a period endpoint. When `present_is_newest` is
/// true, a null/empty value (current job) maps to `(i64::MAX, 99)` so it sorts
/// first under descending order; otherwise (a missing start) it maps to
/// `(0, 0)` so it sorts last.
fn period_sort_key(v: &Yaml, present_is_newest: bool) -> (i64, i64) {
    let s = match v {
        Yaml::Str(s) => s.as_str(),
        Yaml::Int(i) => {
            // bare year (e.g. 2014) parsed as Int — treat as year-only
            let y = *i;
            return (y, 0);
        }
        Yaml::Null => "",
        _ => return (0, 0),
    };
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("null") || s == "~" {
        return if present_is_newest { (i64::MAX, 99) } else { (0, 0) };
    }
    // "2024-09" -> (2024, 9)
    let bytes = s.as_bytes();
    if s.len() >= 7 && bytes[4] == b'-' {
        let year = s[..4].parse::<i64>().unwrap_or(0);
        let month = s[5..7].parse::<i64>().unwrap_or(0);
        return (year, month);
    }
    // bare year as a string
    if let Ok(y) = s.parse::<i64>() {
        return (y, 0);
    }
    (0, 0)
}
}

// --- emitted data model ----------------------------------------------------

struct Module {
    profile: Yaml,
    skills: Yaml,
    publications: Yaml,
    experience: Vec<Entry>,
    education: Vec<Entry>,
    teaching: Vec<Entry>,
    talks: Vec<Entry>,
    blog: Vec<Entry>,
    portfolio: Vec<Entry>,
    archive: Vec<Entry>,
}

/// Extract the (en, fr, fr_draft) triple from a bilingual prose file
/// (skills.md / publications.md), where `en`/`fr` are `|` block scalars and
/// `fr_draft` is a bool. Returns empty strings / false when absent.
fn prose_yaml(y: &Yaml) -> (String, String, bool) {
    let en = y.get("en").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let fr = y.get("fr").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let draft = y.get("fr_draft").and_then(|v| v.as_bool()).unwrap_or(false);
    (en, fr, draft)
}

struct Entry {
    id: String,
    order: i64,
    front: Yaml,
}

fn render_module(m: &Module) -> String {
    let mut s = String::new();
    s.push_str("// AUTO-GENERATED by build.rs from site/content/*. Do not edit by hand.\n");
    s.push_str("// Edit the Markdown under content/ and rebuild.\n\n");
    s.push_str("use crate::content::{Entry, LStr};\n\n");

    s.push_str("pub static PROFILE: &[(&str, &str)] = &[\n");
    emit_profile(&mut s, &m.profile);
    s.push_str("];\n\n");

    s.push_str("pub static SKILLS: LStr = ");
    emit_prose(&mut s, &m.skills);
    s.push_str(";\n");
    s.push_str("pub static SKILLS_DRAFT: bool = ");
    s.push_str(if prose_yaml(&m.skills).2 { "true" } else { "false" });
    s.push_str(";\n\n");

    s.push_str("pub static PUBLICATIONS: LStr = ");
    emit_prose(&mut s, &m.publications);
    s.push_str(";\n");
    s.push_str("pub static PUBLICATIONS_DRAFT: bool = ");
    s.push_str(if prose_yaml(&m.publications).2 { "true" } else { "false" });
    s.push_str(";\n\n");

    s.push_str("pub static EXPERIENCE: &[Entry] = &[\n");
    for e in &m.experience {
        emit_entry(&mut s, e);
    }
    s.push_str("];\n\n");

    s.push_str("pub static EDUCATION: &[Entry] = &[\n");
    for e in &m.education {
        emit_entry(&mut s, e);
    }
    s.push_str("];\n\n");

    // The remaining collections all use the same Entry shape; `emit_entry` is
    // generic over role/degree + venue + period + summary, so teaching/talks/
    // blog/portfolio/archive all render through it. `venue` may be a scalar
    // (experience `company`) or an EN/FR map (teaching/talks `venue`); both are
    // handled inside emit_entry.
    emit_collection(&mut s, "TEACHING", &m.teaching);
    emit_collection(&mut s, "TALKS", &m.talks);
    emit_collection(&mut s, "BLOG", &m.blog);
    emit_collection(&mut s, "PORTFOLIO", &m.portfolio);
    emit_collection(&mut s, "ARCHIVE", &m.archive);
    s
}

fn emit_collection(s: &mut String, name: &str, entries: &[Entry]) {
    s.push_str("pub static ");
    s.push_str(name);
    s.push_str(": &[Entry] = &[\n");
    for e in entries {
        emit_entry(s, e);
    }
    s.push_str("];\n\n");
}

fn emit_profile(s: &mut String, p: &Yaml) {
    let get = |k: &str| p.get(k).and_then(|v| v.as_str()).unwrap_or("");
    let loc_en = p.get("location").and_then(|l| l.get("en")).and_then(|v| v.as_str()).unwrap_or("");
    let loc_fr = p.get("location").and_then(|l| l.get("fr")).and_then(|v| v.as_str()).unwrap_or(loc_en);
    let title_en = p.get("title").and_then(|t| t.get("en")).and_then(|v| v.as_str()).unwrap_or("");
    let title_fr = p.get("title").and_then(|t| t.get("fr")).and_then(|v| v.as_str()).unwrap_or(title_en);
    push_kv(s, "name", get("name"));
    push_kv(s, "email", get("email"));
    push_kv(s, "linkedin", get("linkedin"));
    push_kv(s, "phone", get("phone"));
    push_kv_loc(s, "title", title_en, title_fr);
    push_kv_loc(s, "location", loc_en, loc_fr);
}

fn push_kv(s: &mut String, k: &str, v: &str) {
    s.push_str("    (");
    push_rust_str(s, k);
    s.push_str(", ");
    push_rust_str(s, v);
    s.push_str("),\n");
}

fn push_kv_loc(s: &mut String, k: &str, en: &str, fr: &str) {
    // encode as key with a NUL separator so the renderer can split EN/FR pairs
    s.push_str("    (");
    push_rust_str(s, k);
    s.push_str(", ");
    push_rust_str(s, &format!("{}\u{0}{}", en, fr));
    s.push_str("),\n");
}

/// Emit a bilingual prose block (skills/publications) as `LStr { en, fr }`,
/// converting each Markdown body to HTML. FR falls back to EN when empty.
fn emit_prose(s: &mut String, y: &Yaml) {
    let (en, fr, _) = prose_yaml(y);
    let en_html = md_to_html(&en);
    let fr_html = if fr.is_empty() { String::new() } else { md_to_html(&fr) };
    s.push_str("LStr { en: ");
    push_rust_str(s, &en_html);
    s.push_str(", fr: ");
    push_rust_str(s, &fr_html);
    s.push_str(" }");
}

fn emit_entry(s: &mut String, e: &Entry) {
    let role_en = e.front.get("role").and_then(|r| r.get("en")).and_then(|v| v.as_str()).unwrap_or("");
    let role_fr = e.front.get("role").and_then(|r| r.get("fr")).and_then(|v| v.as_str()).filter(|x| !x.is_empty()).unwrap_or(role_en);

    let degree_en = e.front.get("degree").and_then(|r| r.get("en")).and_then(|v| v.as_str());
    let degree_fr = e.front.get("degree").and_then(|r| r.get("fr")).and_then(|v| v.as_str());
    // Blog entries carry `title` instead of `role`/`degree`; fall through to it
    // so a blog post's heading is the post title rather than empty.
    let title_en = e.front.get("title").and_then(|r| r.get("en")).and_then(|v| v.as_str());
    let title_fr = e.front.get("title").and_then(|r| r.get("fr")).and_then(|v| v.as_str());
    let head_en = degree_en.or(title_en).unwrap_or(role_en);
    let head_fr = degree_fr
        .filter(|x| !x.is_empty())
        .or_else(|| title_fr.filter(|x| !x.is_empty()))
        .unwrap_or(degree_en.or(title_en).unwrap_or(role_fr));

    // `org` (the Entry.org field) is a single string, not bilingual. Experience
    // uses a scalar `company`; teaching/talks/portfolio/archive use `venue`,
    // which may be a scalar OR an EN/FR map. Prefer `company`, then `venue`
    // scalar, then `venue.en`. The FR org falls back to the EN form (org names
    // like "Sorbonnes University — UPMC" are usually un-translated).
    let company = e.front.get("company").and_then(|v| v.as_str()).unwrap_or("");
    let venue = e.front.get("venue").and_then(|v| v.as_str())
        .unwrap_or_else(|| e.front.get("venue").and_then(|v| v.get("en")).and_then(|v| v.as_str()).unwrap_or(""));

    let loc_en = e.front.get("location").and_then(|l| l.get("en")).and_then(|v| v.as_str()).unwrap_or("");
    let loc_fr = e.front.get("location").and_then(|l| l.get("fr")).and_then(|v| v.as_str()).filter(|x| !x.is_empty()).unwrap_or(loc_en);

    let period = e.front.get("period").cloned().unwrap_or(Yaml::Null);
    let (start, end) = period_range(&period);

    let summary = e.front.get("summary").cloned().unwrap_or(Yaml::Null);
    // Body resolution with a `.short` fallback: prefer the long form, fall back
    // to the short form (used by teaching/talks/portfolio entries that only
    // carry a one-line summary). FR falls back to its own short, then to EN.
    let long_en = summary.get("en.long").and_then(|v| v.as_str()).filter(|x| !x.is_empty())
        .unwrap_or_else(|| summary.get("en.short").and_then(|v| v.as_str()).unwrap_or(""));
    let long_fr = summary.get("fr.long").and_then(|v| v.as_str()).filter(|x| !x.is_empty())
        .unwrap_or_else(|| summary.get("fr.short").and_then(|v| v.as_str()).filter(|x| !x.is_empty()).unwrap_or(""));
    let fr_draft = summary.get("fr_draft").and_then(|v| v.as_bool()).unwrap_or(false);

    // Education entries use a different schema: thesis / honors / specialization
    // (each an EN/FR LStr) plus an optional `courses` list (of LStr maps). When
    // there's no summary body at all, synthesize one in Markdown from those
    // fields so the education page isn't blank.
    let (long_en, long_fr) = if long_en.is_empty() {
        education_body(&e.front)
    } else {
        (long_en.to_string(), long_fr.to_string())
    };

    let long_en_html = md_to_html(&long_en);
    let long_fr_html = if long_fr.is_empty() { String::new() } else { md_to_html(&long_fr) };

    s.push_str("    Entry {\n");
    s.push_str("        id: "); push_rust_str(s, &e.id); s.push_str(",\n");
    s.push_str("        order: "); s.push_str(&e.order.to_string()); s.push_str(",\n");
    s.push_str("        heading: LStr { en: "); push_rust_str(s, head_en); s.push_str(", fr: "); push_rust_str(s, head_fr); s.push_str(" },\n");
    s.push_str("        org: "); push_rust_str(s, if company.is_empty() { venue } else { company }); s.push_str(",\n");
    s.push_str("        location: LStr { en: "); push_rust_str(s, loc_en); s.push_str(", fr: "); push_rust_str(s, loc_fr); s.push_str(" },\n");
    s.push_str("        start: "); push_rust_str(s, &start); s.push_str(",\n");
    s.push_str("        end: "); push_rust_str(s, &end); s.push_str(",\n");
    s.push_str("        body_en: "); push_rust_str(s, &long_en_html); s.push_str(",\n");
    s.push_str("        body_fr: "); push_rust_str(s, &long_fr_html); s.push_str(",\n");
    s.push_str("        fr_draft: "); s.push_str(if fr_draft { "true" } else { "false" }); s.push_str(",\n");
    s.push_str("    },\n");
}

/// Build an (en, fr) Markdown body for an education entry from its own fields
/// (`thesis`, `honors`, `specialization`, `courses`). Each LStr field contributes
/// a bullet; FR is taken per-field and left empty when absent (the renderer falls
/// back to EN). Returns empty strings when none of the fields are present.
fn education_body(front: &Yaml) -> (String, String) {
    let mut en = String::new();
    let mut fr = String::new();
    let mut fr_any = false;
    let mut push = |label_en: &str, label_fr: &str, val: &Yaml| {
        let v_en = val.get("en").and_then(|v| v.as_str()).unwrap_or("").trim();
        let v_fr = val.get("fr").and_then(|v| v.as_str()).unwrap_or("").trim();
        if v_en.is_empty() {
            return;
        }
        en.push_str("- **");
        en.push_str(label_en);
        en.push_str("**: ");
        en.push_str(v_en);
        en.push('\n');
        if v_fr.is_empty() {
            fr.push_str("- **");
            fr.push_str(label_fr);
            fr.push_str("**: ");
            fr.push_str(v_en);
            fr.push('\n');
        } else {
            fr_any = true;
            fr.push_str("- **");
            fr.push_str(label_fr);
            fr.push_str("**: ");
            fr.push_str(v_fr);
            fr.push('\n');
        }
    };
    push("Thesis", "Thèse", &front.get("thesis").cloned().unwrap_or(Yaml::Null));
    push("Honors", "Mention", &front.get("honors").cloned().unwrap_or(Yaml::Null));
    push("Specialization", "Spécialisation", &front.get("specialization").cloned().unwrap_or(Yaml::Null));
    // `courses` is a block list of LStr maps: `- en: "..." \n   fr: ""`
    if let Some(courses) = front.get("courses") {
        if let Yaml::List(items) = courses {
            for c in items {
                let c_en = c.get("en").and_then(|v| v.as_str()).unwrap_or("").trim();
                let c_fr = c.get("fr").and_then(|v| v.as_str()).unwrap_or("").trim();
                if c_en.is_empty() {
                    continue;
                }
                en.push_str("- ");
                en.push_str(c_en);
                en.push('\n');
                if c_fr.is_empty() {
                    fr.push_str("- ");
                    fr.push_str(c_en);
                    fr.push('\n');
                } else {
                    fr_any = true;
                    fr.push_str("- ");
                    fr.push_str(c_fr);
                    fr.push('\n');
                }
            }
        }
    }
    // If nothing was FR-translated, return an empty FR string so the renderer
    // falls back to EN wholesale rather than showing EN-with-FR-labels.
    if !fr_any {
        fr.clear();
    }
    (en, fr)
}

fn period_range(p: &Yaml) -> (String, String) {
    let start = p.get("start").and_then(|v| v.scalar_str()).unwrap_or_default();
    let end = p.get("end").and_then(|v| v.scalar_str());
    let end = match end {
        Some(s) if s.is_empty() => String::new(),
        Some(s) => s.to_string(),
        None => String::new(), // null / absent => current
    };
    (start, end)
}

// --- tiny YAML parser (subset used by the content files) -------------------
// Supports: null, scalars (plain/quoted), block scalars (|), nested maps,
// inline-flow lists are NOT used; block lists (- item) are supported for
// `courses`. Values may be `key: value` or `key:` then a nested block.
// This is deliberately minimal — the content schema is hand-controlled.

#[derive(Clone, Debug)]
enum Yaml {
    Null,
    Str(String),
    Bool(bool),
    Int(i64),
    Map(BTreeMap<String, Yaml>),
    List(Vec<Yaml>),
}

impl Yaml {
    fn get(&self, k: &str) -> Option<&Yaml> {
        if let Yaml::Map(m) = self { m.get(k) } else { None }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Yaml::Str(s) => Some(s),
            Yaml::Null => Some(""),
            _ => None,
        }
    }
    /// Like `as_str`, but also coerces an Int to its decimal string. Needed for
    /// `period.start: 2011` (a bare year parses as Int, not Str) so year-only
    /// periods aren't dropped.
    fn scalar_str(&self) -> Option<String> {
        match self {
            Yaml::Str(s) => Some(s.clone()),
            Yaml::Null => Some(String::new()),
            Yaml::Int(i) => Some(i.to_string()),
            _ => None,
        }
    }
    fn as_bool(&self) -> Option<bool> {
        if let Yaml::Bool(b) = self { Some(*b) } else { None }
    }
}

fn parse_file(path: &Path) -> Yaml {
    let txt = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let body = strip_frontmatter(&txt).0;
    parse_block(&mut body.lines().peekable(), 0)
}

/// Parse an experience/education entry: frontmatter Yaml + id/order from it.
fn parse_entry(path: &Path) -> Option<Entry> {
    let front = parse_file(path);
    let id = front.get("id").and_then(|v| v.as_str())?.to_string();
    let order = front.get("order").and_then(|v| {
        if let Yaml::Int(i) = v { Some(*i) } else { None }
    }).unwrap_or(0);
    Some(Entry { id, order, front })
}

/// Split leading `---\n...\n---\n` frontmatter from the body. Returns (fm, body).
fn strip_frontmatter(txt: &str) -> (String, String) {
    let mut lines = txt.lines();
    let first = lines.next();
    if first.map(|l| l.trim() == "---").unwrap_or(false) {
        let mut fm = String::new();
        for l in lines.by_ref() {
            if l.trim() == "---" {
                break;
            }
            fm.push_str(l);
            fm.push('\n');
        }
        let body: String = lines.collect::<Vec<_>>().join("\n");
        (fm, body)
    } else {
        (String::new(), txt.to_string())
    }
}

fn parse_block<'a, I: Iterator<Item = &'a str>>(lines: &mut std::iter::Peekable<I>, min_indent: usize) -> Yaml {
    let mut map: BTreeMap<String, Yaml> = BTreeMap::new();
    let mut list: Vec<Yaml> = Vec::new();
    let mut is_list = false;
    while let Some(&line) = lines.peek() {
        if line.trim().is_empty() {
            lines.next();
            continue;
        }
        let indent = count_indent(line);
        if indent < min_indent {
            break;
        }
        let trimmed = &line[indent..];
        if trimmed.starts_with("# ") || trimmed.starts_with("#") {
            lines.next();
            continue;
        }
        if trimmed.starts_with("- ") {
            is_list = true;
            lines.next();
            let item = &trimmed[2..];
            // inline scalar after "- "
            list.push(parse_scalar_or_nested(item, lines, indent + 2));
            continue;
        }
        if is_list {
            break;
        }
        // key: value
        if let Some(colon) = find_kv_colon(trimmed) {
            lines.next();
            let key = trimmed[..colon].trim().trim_matches('"').to_string();
            let rest = trimmed[colon + 1..].trim();
            let val = parse_value(rest, lines, indent);
            map.insert(key, val);
            continue;
        }
        break;
    }
    if is_list {
        Yaml::List(list)
    } else if map.is_empty() {
        Yaml::Null
    } else {
        Yaml::Map(map)
    }
}

fn parse_scalar_or_nested<'a, I: Iterator<Item = &'a str>>(item: &str, lines: &mut std::iter::Peekable<I>, min_indent: usize) -> Yaml {
    let item = item.trim();
    if item.is_empty() {
        return parse_block(lines, min_indent);
    }
    // "- key: value" form (list of maps)
    if let Some(colon) = find_kv_colon(item) {
        let key = item[..colon].trim().trim_matches('"').to_string();
        let rest = item[colon + 1..].trim();
        let mut map: BTreeMap<String, Yaml> = BTreeMap::new();
        map.insert(key, parse_value(rest, lines, min_indent));
        // continue consuming further indented keys belonging to this map item
        while let Some(&line) = lines.peek() {
            if line.trim().is_empty() {
                lines.next();
                continue;
            }
            let indent = count_indent(line);
            if indent < min_indent {
                break;
            }
            let trimmed = &line[indent..];
            if trimmed.starts_with("- ") {
                break;
            }
            if let Some(colon) = find_kv_colon(trimmed) {
                lines.next();
                let key = trimmed[..colon].trim().trim_matches('"').to_string();
                let rest = trimmed[colon + 1..].trim();
                map.insert(key, parse_value(rest, lines, indent));
                continue;
            }
            break;
        }
        return Yaml::Map(map);
    }
    parse_scalar(item)
}

fn parse_value<'a, I: Iterator<Item = &'a str>>(rest: &str, lines: &mut std::iter::Peekable<I>, indent: usize) -> Yaml {
    // Strip a trailing inline `# comment`. In YAML a `#` is a comment only at the
    // start of a token or preceded by whitespace, and never inside quotes — so
    // `linkedin: florencemonna  # url` and `fr: ""  # TODO` parse as just
    // `florencemonna` / `""`. Block-scalar *content* lines are read raw below, so
    // a `#` inside a `|` body is preserved.
    let rest = strip_inline_comment(rest).trim();
    if rest == "|" || rest == "|-" || rest == "|+" {
        // block scalar: collect indented lines
        let mut buf = String::new();
        while let Some(&line) = lines.peek() {
            if line.trim().is_empty() {
                buf.push('\n');
                lines.next();
                continue;
            }
            let ind = count_indent(line);
            if ind <= indent {
                break;
            }
            lines.next();
            buf.push_str(&line[indent + 1..]);
            buf.push('\n');
        }
        return Yaml::Str(buf);
    }
    if rest.is_empty() {
        // nested map on following lines
        return parse_block(lines, indent + 1);
    }
    parse_scalar(rest)
}

/// Strip a trailing YAML inline comment from an unquoted scalar value. A `#`
/// counts as a comment start only when at the beginning of the (trimmed) value
/// or preceded by a space; `#` inside a single- or double-quoted string is kept.
fn strip_inline_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'\\' if in_double => {
                // escaped char inside double quotes — skip next
                i += 2;
                continue;
            }
            b'\'' if in_double => {}
            b'\'' if in_single => {
                // YAML single-quote escape is '' for a literal '
                if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    i += 2;
                    continue;
                }
                in_single = false;
            }
            b'\'' if !in_single && !in_double => in_single = true,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double => {
                if i == 0 || bytes[i - 1] == b' ' {
                    return &s[..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    s
}

fn parse_scalar(s: &str) -> Yaml {
    let s = s.trim();
    if s.is_empty() || s == "null" || s == "~" {
        return Yaml::Null;
    }
    if s == "true" {
        return Yaml::Bool(true);
    }
    if s == "false" {
        return Yaml::Bool(false);
    }
    if let Ok(i) = s.parse::<i64>() {
        return Yaml::Int(i);
    }
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        return Yaml::Str(s[1..s.len() - 1].to_string());
    }
    Yaml::Str(s.to_string())
}

fn find_kv_colon(s: &str) -> Option<usize> {
    // first ": " or trailing ":"
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b':' {
            if i + 1 == s.len() {
                return Some(i);
            }
            if bytes[i + 1] == b' ' {
                return Some(i);
            }
        }
    }
    None
}

fn count_indent(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

// --- Markdown -> HTML (minimal subset) -------------------------------------
// Handles: ATX headings (#..######), unordered list items (- ), paragraphs,
// inline **bold**, *italic*, and pass-through text. Smart-quotes left as-is.

fn md_to_html(md: &str) -> String {
    let mut out = String::new();
    let mut in_list = false;
    let mut para = String::new();

    let flush_para = |para: &mut String, out: &mut String, in_list: &mut bool| {
        close_list(out, in_list);
        if !para.is_empty() {
            out.push_str("<p>");
            out.push_str(&inline_md(para.trim()));
            out.push_str("</p>\n");
            para.clear();
        }
    };

    // Closes an open `<ul>` if we are about to emit a different block kind.
    fn close_list(out: &mut String, in_list: &mut bool) {
        if *in_list {
            out.push_str("</ul>\n");
            *in_list = false;
        }
    }

    for line in md.lines() {
        let t = line.trim_start();
        if t.starts_with("# ") {
            flush_para(&mut para, &mut out, &mut in_list);
            out.push_str("<h2>");
            out.push_str(&inline_md(&t[2..].trim()));
            out.push_str("</h2>\n");
        } else if t.starts_with("## ") {
            flush_para(&mut para, &mut out, &mut in_list);
            out.push_str("<h3>");
            out.push_str(&inline_md(&t[3..].trim()));
            out.push_str("</h3>\n");
        } else if t.starts_with("#") {
            // heading deeper than 2 -> h3
            flush_para(&mut para, &mut out, &mut in_list);
            let hs = t.find(' ').unwrap_or(t.len());
            out.push_str("<h3>");
            out.push_str(&inline_md(&t[hs..].trim()));
            out.push_str("</h3>\n");
        } else if t.starts_with("- ") || t.starts_with("* ") {
            if !para.is_empty() {
                flush_para(&mut para, &mut out, &mut in_list);
            }
            if !in_list {
                out.push_str("<ul>\n");
                in_list = true;
            }
            out.push_str("<li>");
            out.push_str(&inline_md(&t[2..].trim()));
            out.push_str("</li>\n");
        } else if t.is_empty() {
            flush_para(&mut para, &mut out, &mut in_list);
        } else {
            close_list(&mut out, &mut in_list);
            if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(t);
        }
    }
    flush_para(&mut para, &mut out, &mut in_list);
    out
}

fn inline_md(s: &str) -> String {
    // escape, then apply **bold** and *italic*
    let mut esc = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => esc.push_str("&amp;"),
            '<' => esc.push_str("&lt;"),
            '>' => esc.push_str("&gt;"),
            _ => esc.push(c),
        }
    }
    // **bold**
    let esc = replace_pairs(&esc, "**", "<strong>", "</strong>");
    // *italic*
    replace_pairs(&esc, "*", "<em>", "</em>")
}

fn replace_pairs(s: &str, delim: &str, open: &str, close: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    let mut open_tag = true;
    while let Some(idx) = rest.find(delim) {
        out.push_str(&rest[..idx]);
        out.push_str(if open_tag { open } else { close });
        open_tag = !open_tag;
        rest = &rest[idx + delim.len()..];
    }
    out.push_str(rest);
    out
}

// --- rust string literal emission ------------------------------------------

fn push_rust_str(s: &mut String, v: &str) {
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            '\u{0}' => s.push_str("\\0"),
            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
}

fn walk_content(dir: &Path, f: &mut impl FnMut(&Path)) {
    if let Ok(rd) = dir.read_dir() {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk_content(&p, f);
            } else if p.extension().is_some_and(|x| x == "md") {
                f(&p);
            }
        }
    }
}
