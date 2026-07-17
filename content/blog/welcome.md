---
# Dummy entry — replace with a real post. The blog lists entries newest-first by
# period.end (date_key end-desc, matching the experience/education timeline).
id: welcome
order: 1
title:
  en: Welcome to my website
  fr: Bienvenue sur mon site
period:
  start: 2026-07
  end: null
summary:
  en.short: A short summary shown on the blog index and in cards.
  en.long: |
    This is a placeholder blog post. Replace `content/blog/welcome.md` with a real
    entry — one file per post, newest file renders first by date.

    The `summary.en.long` body is the post content rendered as Markdown (headings,
    paragraphs, bullet lists, **bold**).
  fr.short: Un résumé court affiché sur l'index du blog et dans les cartes.
  fr.long: |
    Ceci est un article d'espace réservé. Remplacez `content/blog/welcome.md` par
    une vraie entrée — un fichier par article, le plus récent s'affiche en premier
    par date.
---

Body below the frontmatter is ignored — the rendered content comes from
`summary.en.long` / `fr.long`.
