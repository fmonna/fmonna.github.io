#!/usr/bin/env python3
# Florence Monna — CV generator (Typst).
#
# Loads the SAME Markdown content the Dioxus site reads (content/*.md — the single
# source of truth, parsed by PyYAML with the flat-dotted summary keys build.rs uses)
# and emits two concrete .typ files:
#
#   cv-en.typ  — full English CV (bullets from `en.long`).
#   cv-fr.typ  — French CV using the `fr.short` bodies (per content/README.md).
#                No experience is excluded; bullet content is whatever Florence
#                authors in `fr.short`, not a generator heuristic.
#
# Each emitted file is self-contained: a `#let cv = (...)` dict literal, then
# `#import "cv.template.typ": render-cv` and `#render-cv(cv)`. The dict's bullets
# and prose bodies are Typst *content blocks* so `**bold**` renders; plain fields
# (name, org, period, ...) are quoted strings.
#
# Bilingual model matches content.rs: EN is the source of truth; FR falls back to
# EN when empty; the FR CV still flags itself a draft via a note (the source FR
# text carries `fr_draft: true`).
#
# No commit — local-only. Run:  python3 tools/gen_cv.py  (from the repo root).

from __future__ import annotations

import sys
from pathlib import Path

import yaml

# --- configuration ----------------------------------------------------------

SITE_DIR = Path(__file__).resolve().parent.parent
CONTENT_DIR = SITE_DIR / "content"
TOOLS_DIR = SITE_DIR / "tools"
TEMPLATE = "cv.template.typ"
OUT_EN = TOOLS_DIR / "cv-en.typ"
OUT_FR = TOOLS_DIR / "cv-fr.typ"

# The FR CV uses the `fr.short` bodies as authored (no bullet cap, no entries
# dropped). `en.short`/`*.short` are the tight one-liners; the EN CV uses
# `en.long` for the full bullet lists. See content/README.md "Bilingual status".
#
# Ordering for BOTH CVs is newest-first, matching the LaTeX reference CV. It is
# derived from the `period.start` date — NOT from an `order:` field — so adding
# a new job never requires renumbering existing entries. (Sort key: start date
# descending; entries with the same start keep a stable filesystem order.)

# Per-language section headings / labels (mirror main.rs page titles + the LaTeX
# main.tex section order: experience → education → skills → newpage → pubs).
STRINGS = {
    "en": {
        "experience_title": "Research and Professional Experience",
        "education_title": "Education and Training",
        "skills_title": "Skills",
        "publications_title": "Publications (primary author in bold)",
    },
    "fr": {
        "experience_title": "Expérience professionnelle et de recherche",
        "education_title": "Formation",
        "skills_title": "Compétences",
        "publications_title": "Publications (premier auteur en gras)",
    },
}

EN_MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
             "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
FR_MONTHS = ["janv.", "févr.", "mars", "avr.", "mai", "juin",
             "juil.", "août", "sept.", "oct.", "nov.", "déc."]


# --- content loading --------------------------------------------------------

def load_yaml(path: Path) -> dict:
    """Parse the frontmatter (between leading `---` fences) of a content file."""
    txt = path.read_text(encoding="utf-8")
    parts = txt.split("---", 2)
    if len(parts) < 3:
        return {}
    return yaml.safe_load(parts[1]) or {}


def date_key(ym) -> tuple[int, int]:
    """Parse a `period` value ('YYYY-MM' or 'YYYY', or None) into a (year, month)
    tuple for sorting. None / empty / 'null' -> (9999, 99) so an open-ended
    ('present') entry sorts as newest. Unparseable -> (0, 0)."""
    if ym is None:
        return (9999, 99)
    s = str(ym).strip()
    if not s or s.lower() == "null":
        return (9999, 99)
    if len(s) >= 7 and s[4] == "-":
        try:
            return (int(s[:4]), int(s[5:7]))
        except ValueError:
            pass
    try:
        return (int(s[:4]), 0)
    except ValueError:
        return (0, 0)


def load_entries(subdir: str) -> list[dict]:
    """Load every *.md in content/<subdir>/, sorted newest-first by `period.end`
    (descending) — matches the LaTeX reference CV (EDF 2024 → oldest), the
    authoritative layout. End date is the key (not start, not `order`): it keeps
    overlapping ranges in Florence's authored order (e.g. Google-CNRS 2011–2014
    before Teaching 2012–2014, as in work_experience.tex), and an open-ended
    `end: null` ('present') sorts first so the current job is always on top.

    This is deliberately NOT the `order:` frontmatter field — adding a new job
    only needs a new file with its real dates; no renumbering of existing
    entries. (build.rs still sorts `Reverse(order)` for the site timeline; the
    PDF diverges here on purpose to match the LaTeX CV ordering.)"""
    d = CONTENT_DIR / subdir
    entries = []
    for p in sorted(d.glob("*.md")):
        entries.append(load_yaml(p))
    entries.sort(key=lambda e: date_key((e.get("period") or {}).get("end")),
                 reverse=True)
    return entries


def load_ordered_entries(subdir: str) -> list[dict]:
    """Load every *.md in content/<subdir>/, sorted ascending by the `order:`
    frontmatter field. For collections without a `period:` (skills, publications),
    `order:` carries the narrative sequence (Languages, Research, …) so it survives
    regardless of filename. Mirrors build.rs load_ordered_entries."""
    d = CONTENT_DIR / subdir
    entries = [load_yaml(p) for p in sorted(d.glob("*.md"))]
    entries.sort(key=lambda e: e.get("order") if e.get("order") is not None else 0)
    return entries


# --- text formatting (mirrors build.rs + main.rs) ---------------------------

def fmt_period(start, end, lang: str) -> str:
    """YYYY-MM or YYYY -> 'Sep 2024' / 'sep 2024'; empty end -> present/présent.
    Matches main.rs format_period exactly."""
    def fmt_one(ym):
        s = str(ym)
        if len(s) >= 7 and s[4] == "-":
            y, m = s[:4], s[5:]
            months = FR_MONTHS if lang == "fr" else EN_MONTHS
            try:
                name = months[int(m) - 1]
            except (ValueError, IndexError):
                return s
            return f"{name} {y}"
        return s

    s = fmt_one(start)
    if end in (None, "", "null"):
        e = "présent" if lang == "fr" else "present"
    else:
        e = fmt_one(end)
    return f"{s} — {e}"


def lstr(field: dict | None, lang: str) -> str:
    """Pick the lang form of an {en, fr} field, falling back to EN when FR empty
    (content.rs LStr::pick). `field` may be None or a plain string."""
    if field is None:
        return ""
    if isinstance(field, str):
        return field
    en = (field.get("en") or "").strip()
    fr = (field.get("fr") or "").strip()
    if lang == "en":
        return en
    return fr if fr else en


def bullets_from_long(text: str) -> list[str]:
    """The `*.long` block scalars are Markdown bullet lists (`- foo`). Extract
    the bullet text (stripping the leading `- `), preserving inline `**bold**`."""
    out = []
    for line in text.splitlines():
        t = line.strip()
        if t.startswith("- "):
            out.append(t[2:].strip())
        elif t and not t.startswith("#"):
            # a stray non-bullet line in a long body — keep as a bullet so it
            # isn't silently dropped.
            out.append(t)
    return out


def education_body(front: dict, lang: str) -> list[str]:
    """Synthesize bullets for an education entry from thesis/honors/specialization
    (+ courses), mirroring build.rs education_body(). Labels are bilingual;
    per-field FR falls back to EN when empty."""
    labels = {
        "thesis": ("Thesis", "Thèse"),
        "honors": ("Honors", "Mention"),
        "specialization": ("Specialization", "Spécialisation"),
    }
    out = []
    for key, (lab_en, lab_fr) in labels.items():
        fld = front.get(key)
        if not fld:
            continue
        en = (fld.get("en") or "").strip()
        if not en:
            continue
        fr = (fld.get("fr") or "").strip()
        lab = lab_fr if lang == "fr" else lab_en
        val = fr if (lang == "fr" and fr) else en
        out.append(f"**{lab}**: {val}")
    # courses (block list of {en, fr}) — present in the schema, unused by the
    # current content, but handled for completeness.
    courses = front.get("courses") or []
    for c in courses:
        en = (c.get("en") or "").strip()
        if not en:
            continue
        fr = (c.get("fr") or "").strip()
        val = fr if (lang == "fr" and fr) else en
        out.append(val)
    return out


# --- Typst emission ---------------------------------------------------------

# Inside a content block `[...]` these characters are markup-active and must be
# backslash-escaped (verified empirically: # @ $ ` \ all need it; * is handled
# separately as the bold delimiter). `<` `>` `&` `%` are literal in content.
_CONTENT_ESCAPES = [
    ("\\", "\\\\"),
    ("`", "\\`"),
    ("#", "\\#"),
    ("@", "\\@"),
    ("$", "\\$"),
]


def esc_content(text: str) -> str:
    """Escape markup-active chars for a Typst content block, and convert markdown
    `**bold**` to Typst `*bold*` (single-star). `*` NOT part of a `**` pair is
    escaped so a lone star doesn't toggle bold."""
    # Escape backslash and the markup-active set first (before touching stars).
    for a, b in _CONTENT_ESCAPES:
        text = text.replace(a, b)
    # Convert **bold** -> *bold*. Walk paired `**` markers.
    out = []
    i = 0
    n = len(text)
    while i < n:
        if text[i:i + 2] == "**":
            out.append("*")
            i += 2
        elif text[i] == "*":
            # a lone star — escape it so it can't start bold
            out.append("\\*")
            i += 1
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


def content_block(text: str) -> str:
    """Wrap escaped text as a Typst content-block literal `[...]`."""
    return "[" + esc_content(text) + "]"


def str_literal(text: str) -> str:
    """Emit a Typst quoted string literal, escaping `\\`, `"`, and newlines.
    Used for plain scalars (name, org, period) that the template interpolates —
    interpolation is fully literal, so no markup escaping is needed here."""
    out = ['"']
    for c in text:
        if c == "\\":
            out.append("\\\\")
        elif c == '"':
            out.append('\\"')
        elif c == "\n":
            out.append("\\n")
        elif c == "\r":
            out.append("\\r")
        elif c == "\t":
            out.append("\\t")
        else:
            out.append(c)
    out.append('"')
    return "".join(out)


def emit_array(items: list[str], indent: str) -> str:
    """Emit a Typst array of content blocks, one per line."""
    if not items:
        return "()"
    inner = ",\n".join(f"{indent}  {it}" for it in items)
    return f"(\n{inner},\n{indent})"


def group_prose(md: str, lang: str) -> list[tuple[str, object]]:
    """Parse a section's Markdown body (a skills/publications `summary.*.long`)
    into pre-grouped blocks the template's `prose()` renders with a pure `for`
    (no mutation): runs of `- ` bullets collapse into one ('list', [blocks])
    entry; `#`/`##` headings become ('head', block); other lines become
    ('para', block). Mirrors build.rs md_to_html's block structure (heading
    levels collapse to one head kind here since the template sizes all heads
    identically)."""
    blocks: list[tuple[str, object]] = []
    pending: list[str] = []

    def flush_list():
        if pending:
            blocks.append(("list", [content_block(b) for b in pending]))
            pending.clear()

    for line in md.splitlines():
        t = line.strip()
        if t.startswith("#"):
            flush_list()
            heading = t.lstrip("#").strip()
            blocks.append(("head", content_block(f"**{heading}**")))
        elif t.startswith("- ") or t.startswith("* "):
            pending.append(t[2:].strip())
        elif t == "":
            flush_list()
        else:
            flush_list()
            blocks.append(("para", content_block(t)))
    flush_list()
    return blocks


def emit_prose_blocks(blocks: list[tuple[str, object]], indent: str) -> str:
    """Emit the grouped prose blocks as a Typst array of (kind:, body:) dicts."""
    if not blocks:
        return "()"
    lines = []
    for kind, body in blocks:
        if kind == "list":
            body_lit = emit_array(body, indent + "    ")
        else:
            body_lit = body  # already a content-block literal
        lines.append(f"{indent}  (kind: {str_literal(kind)}, body: {body_lit}),")
    return f"(\n{chr(10).join(lines)}\n{indent})"


def section_blocks(entry: dict, lang: str) -> list[tuple[str, object]]:
    """One skills/publications section (one file) -> a head block carrying the
    section heading, followed by the body's prose blocks (bullet list or
    paragraph). This is what group_prose used to produce from a `## heading`
    line plus body in the old single-file form; now the heading lives in
    frontmatter and the body in `summary.<lang>.long` (FR falls back to EN)."""
    blocks: list[tuple[str, object]] = []
    heading = lstr(entry.get("heading"), lang)
    if heading:
        blocks.append(("head", content_block(f"**{heading}**")))
    summary = entry.get("summary") or {}
    en = (summary.get("en.long") or "").strip()
    fr = (summary.get("fr.long") or "").strip()
    body = fr if (lang == "fr" and fr) else en
    blocks.extend(group_prose(body, lang))
    return blocks


# --- CV assembly ------------------------------------------------------------

def build_experience(entries: list[dict], lang: str):
    """Build experience entries. Bullet source is the lang's `*.short` field,
    falling back to `*.long`, then to `en.long` (content.rs LStr::pick style:
    EN is the source of truth). Per content/README.md, the FR CV uses `fr.short`
    (the tight bodies Florence authors) and the EN CV uses `en.long` (full
    bullets) — but both go through the same fallback chain so a missing field
    never silently empties an entry."""
    out = []
    for e in entries:
        role = lstr(e.get("role"), lang)
        org = (e.get("company") or e.get("venue") or "").strip()
        location = lstr(e.get("location"), lang)
        period = fmt_period(e.get("period", {}).get("start"),
                            e.get("period", {}).get("end"), lang)
        summ = e.get("summary") or {}
        # Bullet source: prefer the lang's .short, then the lang's .long, then
        # en.long. (fr.short -> fr.long -> en.long / en.short -> en.long.)
        text = ""
        for key in (f"{lang}.short", f"{lang}.long", "en.long"):
            v = (summ.get(key) or "").strip()
            if v:
                text = v
                break
        bullets = bullets_from_long(text) if text else []
        if not bullets:
            continue
        bullet_lits = [content_block(b) for b in bullets]
        out.append({
            "role": content_block(role),
            "org": str_literal(org),
            "location": str_literal(location),
            "period": str_literal(period),
            "bullets": emit_array(bullet_lits, "        "),
        })
    return out


def build_education(entries: list[dict], lang: str):
    out = []
    for e in entries:
        degree = lstr(e.get("degree"), lang)
        venue = (e.get("venue") or e.get("company") or "").strip()
        location = lstr(e.get("location"), lang)
        period = fmt_period(e.get("period", {}).get("start"),
                            e.get("period", {}).get("end"), lang)
        bullets = education_body(e, lang)
        if not bullets:
            continue
        bullet_lits = [content_block(b) for b in bullets]
        out.append({
            "degree": content_block(degree),
            "venue": str_literal(venue),
            "location": str_literal(location),
            "period": str_literal(period),
            "bullets": emit_array(bullet_lits, "        "),
        })
    return out


def build_cv(lang: str, profile: dict, skills: list[dict],
             publications: list[dict], experience: list[dict],
             education: list[dict]) -> str:
    name = (profile.get("name") or "").strip()
    title = lstr(profile.get("title"), lang)
    contact = {
        "email": str_literal((profile.get("email") or "").strip()),
        "linkedin": str_literal((profile.get("linkedin") or "").strip()),
        "phone": str_literal((profile.get("phone") or "").strip()),
        "location": str_literal(lstr(profile.get("location"), lang)),
    }
    exp = build_experience(experience, lang)
    edu = build_education(education, lang)

    skills_blocks = [b for e in skills for b in section_blocks(e, lang)]
    pubs_blocks = [b for e in publications for b in section_blocks(e, lang)]

    s = STRINGS[lang]
    lines = []
    lines.append("// AUTO-GENERATED by tools/gen_cv.py from content/*.md.")
    lines.append("// Do not edit by hand — edit the Markdown and re-run the generator.")
    label = "FR (fr.short bodies)" if lang == "fr" else "EN (full, en.long)"
    lines.append(f"// CV: {label}.")
    lines.append("")
    lines.append(f'#import "{TEMPLATE}": render-cv')
    lines.append("")
    lines.append("#let cv = (")
    lines.append(f'  lang: {str_literal(lang)},')
    lines.append("  strings: (")
    for k, v in s.items():
        lines.append(f"    {k}: {str_literal(v)},")
    lines.append("  ),")
    lines.append(f"  name: {str_literal(name)},")
    lines.append(f"  title: {str_literal(title)},")
    lines.append("  contact: (")
    lines.append(f"    email: {contact['email']},")
    lines.append(f"    linkedin: {contact['linkedin']},")
    lines.append(f"    phone: {contact['phone']},")
    lines.append(f"    location: {contact['location']},")
    lines.append("  ),")
    lines.append("  experience: (")
    for e in exp:
        lines.append("    (")
        lines.append(f"      role: {e['role']},")
        lines.append(f"      org: {e['org']},")
        lines.append(f"      location: {e['location']},")
        lines.append(f"      period: {e['period']},")
        lines.append(f"      bullets: {e['bullets']},")
        lines.append("    ),")
    lines.append("  ),")
    lines.append("  education: (")
    for e in edu:
        lines.append("    (")
        lines.append(f"      degree: {e['degree']},")
        lines.append(f"      venue: {e['venue']},")
        lines.append(f"      location: {e['location']},")
        lines.append(f"      period: {e['period']},")
        lines.append(f"      bullets: {e['bullets']},")
        lines.append("    ),")
    lines.append("  ),")
    lines.append(f"  skills: {emit_prose_blocks(skills_blocks, '  ')},")
    lines.append(f"  publications: {emit_prose_blocks(pubs_blocks, '  ')},")
    lines.append(")")
    lines.append("")
    lines.append("#render-cv(cv)")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    if not (CONTENT_DIR / "profile.md").exists():
        print(f"error: content dir not found at {CONTENT_DIR}", file=sys.stderr)
        return 1

    profile = load_yaml(CONTENT_DIR / "profile.md")
    skills = load_ordered_entries("skills")
    publications = load_ordered_entries("publications")
    experience = load_entries("experience")
    education = load_entries("education")

    en = build_cv("en", profile=profile, skills=skills,
                  publications=publications, experience=experience,
                  education=education)
    fr = build_cv("fr", profile=profile, skills=skills,
                  publications=publications, experience=experience,
                  education=education)

    OUT_EN.write_text(en, encoding="utf-8")
    OUT_FR.write_text(fr, encoding="utf-8")
    print(f"wrote {OUT_EN.relative_to(SITE_DIR)}  ({en.count(chr(10))} lines)")
    print(f"wrote {OUT_FR.relative_to(SITE_DIR)}  ({fr.count(chr(10))} lines)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
