// Florence Monna — CV template (Typst 0.14).
//
// Shared by the English (full) and French (2-page condensed) CVs. The Python
// generator (tools/gen_cv.py) loads content/*.md — the single source of truth,
// same schema the Dioxus build.rs reads — and emits a concrete .typ file that
// `#import`s this template and calls `render-cv(cv)`.
//
// `cv` is a dict shaped by the generator:
//   cv = (
//     lang: "en" | "fr",
//     strings: (…),          // per-language section headings & labels
//     name: str, title: str,
//     contact: (email, linkedin, phone, location),
//     experience: ((role: content, org: str, location: str, period: str, bullets: (content, …)), …),
//     education:   ((degree: content, venue: str, location: str, period: str, bullets: (content, …)), …),
//     skills: ((kind: "head"|"para"|"list", body: content | (content, …)), …),
//     publications: ((kind: …, body: …), …),
//   )
// `bullets` are content arrays; prose is pre-grouped into head/para/list blocks.
// The generator has already converted markdown **bold** → Typst *bold* and
// escaped Typst-special characters (# $ @ \ < > ` etc.) in all text fields.
//
// Ordering (experience & education) is newest-first by `period.end`, set by the
// generator from the content's real dates — not an `order:` field. The template
// itself is order-agnostic: it renders whatever list it is given.

#let render-cv(cv) = {
  let s = cv.strings
  let is-fr = cv.lang == "fr"

  // --- page geometry: dense but readable. FR targets ~2 pages, so slightly
  // tighter margins/font than the full EN CV.
  let margin = if is-fr { (top: 1.3cm, bottom: 1.1cm, left: 1.4cm, right: 1.4cm) }
               else { (top: 1.6cm, bottom: 1.4cm, left: 1.7cm, right: 1.7cm) }

  set page(
    paper: "a4",
    margin: margin,
    numbering: none,
  )
  set text(font: "New Computer Modern", size: if is-fr { 9.5pt } else { 10pt }, lang: cv.lang)
  set par(leading: 0.62em, justify: true)
  show link: set text(fill: black)   // keep the CV monochrome / printable

  // --- header: name + title on one line, contact line below.
  block(width: 100%)[
    #text(size: if is-fr { 19pt } else { 20pt }, weight: "bold", tracking: 0.5pt)[#cv.name]
    #h(0.6em) #text(size: if is-fr { 12pt } else { 13pt }, fill: luma(90))[#cv.title]
    #v(0.3em)
    #block(width: 100%)[
      #set text(size: if is-fr { 8.5pt } else { 9pt }, fill: luma(70))
      #cv.contact.email
      #h(1.2em) • #h(1.2em)
      #link("https://linkedin.com/in/" + cv.contact.linkedin)[linkedin.com/in/#cv.contact.linkedin]
      #h(1.2em) • #h(1.2em)
      #cv.contact.location
    ]
  ]
  v(0.4em)

  // --- section heading: small-caps title with a thin rule beneath.
  let section(title) = {
    v(1em)
    block(width: 100%, spacing: 0pt)[
      #text(size: if is-fr { 11pt } else { 12pt }, weight: "bold", tracking: 0.3pt)[#upper(title)]
      #v(-0.7em)
      #line(length: 100%, stroke: 0.6pt + luma(170))
    ]
    v(-0.1em)
  }

  // --- entry: role (bold) on the left, period right-aligned; org — location below.
  let entry(heading, org, location, period, bullets) = {
    block(width: 100%, spacing: 0pt)[
      #v(0.7em)
      #grid(
        columns: (1fr, auto),
        column-gutter: 1em,
        text(weight: "bold")[#heading],
        text(size: if is-fr { 8.5pt } else { 9pt }, fill: luma(90))[#period],
      )
      #v(-0.7em)
      #text(size: if is-fr { 9pt } else { 9.5pt }, fill: luma(70))[#org — #location]
    ]
    // Explicit gap before the bullets: the header `block` carries `spacing: 0pt`,
    // and a bare following `block` collapses the leading parskip, so the first
    // bullet's top climbs back over the org line's baseline (verified — they
    // overlapped ~5pt in the rendered PDF). A forced `v(0.4em)` keeps a clean
    // line of separation without the runaway spacing `above: 0.4em` produced.
    if bullets.len() > 0 {
      v(0.4em)
      block[
        #set text(size: if is-fr { 9pt } else { 9.5pt })
        #set par(leading: 0.5em)
        #list(marker: [•], indent: 0.6em, body-indent: 0.4em, ..bullets)
      ]
    }
    v(0.3em)
  }

  // --- prose block (skills, publications): the generator pre-groups the
  // markdown lines into blocks — (kind: "head", body: content) for headings,
  // (kind: "para", body: content) for bare lines, (kind: "list", body: (content, …))
  // for runs of bullets — so this stays a pure `for` with no mutation (Typst's
  // `let` is immutable across loop bodies).
  let prose(blocks) = {
    set text(size: if is-fr { 9pt } else { 9.5pt })
    set par(leading: 0.55em)
    for b in blocks {
      if b.kind == "head" {
        block(spacing: 0.6em)[#b.body]
      } else if b.kind == "list" {
        list(marker: [•], indent: 0.6em, body-indent: 0.4em, ..b.body)
      } else {
        block(spacing: 0.4em)[#b.body]
      }
    }
  }

  // --- sections in canonical CV order (matches the LaTeX original).
  section(s.experience_title)
  for e in cv.experience {
    entry(e.role, e.org, e.location, e.period, e.bullets)
  }

  section(s.education_title)
  for e in cv.education {
    entry(e.degree, e.venue, e.location, e.period, e.bullets)
  }

  if is-fr { pagebreak() } else {}
  section(s.skills_title)
  prose(cv.skills)

  // Publications: full EN CV page-breaks (mirrors LaTeX \newpage); the FR CV
  // keeps them inline.
  if not is-fr { pagebreak() } else { v(0.4em) }
  section(s.publications_title)
  prose(cv.publications)
}
