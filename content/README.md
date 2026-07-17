# Content — single source of truth

Florence's resume content lives here in Markdown. It feeds **two** outputs:

1. **The Dioxus site** (`src/`) — reads these files, renders EN/FR bilingual pages.
2. **The Typst PDFs** (`tools/`, see `tools/gen_cv.py` + `tools/cv.template.typ`) — generates two CVs:
   - EN full CV — uses `en.long` bodies.
   - FR 2-page CV — uses `fr.short` bodies (tighter).

## Schema

Every entry is a Markdown file with YAML frontmatter.

### Experience (`experience/*.md`)

```yaml
---
id: edf                      # stable slug, used as a key
order: 1                     # display order (highest = most recent)
role:
  en: High Performance Computing Architect
  fr: Architecte Haute Performance Computing
company: EDF R&D
location:
  en: Paris, France
  fr: Paris, France
period:
  start: 2024-09
  end: null                  # null = current
summary:
  en.short: ""               # 1-line, for the FR 2-page CV
  en.long: |                 # full bullets, for the EN full CV + site
    - bullet one
    - bullet two
  fr.short: ""               # TODO: translate
  fr.long: ""                # TODO: translate
---
```

### Education (`education/*.md`), Teaching (`teaching/*.md`)

Same shape: `role`/`degree`/`venue`/`location`/`period` + bilingual `summary`.
Education entries may instead carry `thesis` / `honors` / `specialization` /
`courses` — when there is no `summary`, build.rs synthesizes the body from those.

### Talks (`talks/*.md`), Portfolio (`portfolio/*.md`), Archive (`archive/*.md`)

One file per item. Same `role` / `venue` (scalar or EN/FR map) / `location` /
`period` / bilingual `summary` shape as experience. `period.end` is treated as a
point-in-time date (a talk or archived item's date). Each currently has a single
placeholder entry to seed the section:

- `talks/sample-talk.md` — replace with a real talk.
- `portfolio/sample-project.md` — replace with a real project.
- `archive/sample-entry.md` — replace with a real archived item.

### Blog (`blog/*.md`)

One file per post. Uses `title` (EN/FR map) instead of `role`/`degree` for the
heading; `period.start` is the publish date (`period.end` left `null` = ongoing).
The post body is `summary.en.long` / `fr.long`, rendered as Markdown. A
placeholder lives at `blog/welcome.md`.

## Ordering

All collection timelines (experience, education, teaching, talks, portfolio,
blog, archive) list **newest-first by `period.end`**, not by the `order:` field.
`period.end: null` / absent means "present" and sorts first. This matches the
CV/LaTeX order, so adding a new job or talk needs no renumbering — `order:` is
read into each entry but is no longer used for sorting.

### Publications (`publications.md`)

One file, one section per category (Journals / Conferences / Guidebooks / Books / MOOC),
primary author **bold** preserved as Markdown `**Monna F.**`.

## Source & reconciliation

The content was migrated from `Florence_Resume/*.tex` (LaTeX, the authoritative
source). The previous Jekyll Markdown (`_experience`, `_education`, `_teaching`,
`_pages/skills.md`) was cross-checked: it is mostly the academicpages template
scaffolding (publications/talks/portfolio/cv.json are unfilled placeholders;
experience `en_short` fields are a failed-migration artifact reading
"Worked on backend systems using Rust and PostgreSQL" — discarded; the Argonne
entry is corrupted; the EDF entry body is placeholder text).

Discrepancies resolved in favor of LaTeX unless flagged below in the entry's
frontmatter (`reconciliation:` note). Open questions for Florence are marked
`# TODO(florence):`.

## Bilingual status

LaTeX is English-only. The `en.long` bodies below are faithful migrations.
`fr.*` and `*.short` fields are scaffolded but left empty / TODO — they should be
written (by Florence) before the FR 2-page CV is generated. Do not auto-translate
her professional record and present it as authored.
