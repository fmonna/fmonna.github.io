#!/usr/bin/env bash
# Build the two CV PDFs (English full + French 2-page condensed) from the
# Markdown content under content/. Local-only — nothing is committed by this.
#
#   tools/gen_cv.py      reads content/*.md → emits tools/cv-{en,fr}.typ
#   tools/cv.template.typ  shared Typst template (render-cv)
#   typst compile        cv-{en,fr}.typ → cv-{en,fr}.pdf
#
# Usage:  ./tools/build_cvs.sh        (from the site/ dir)
set -euo pipefail

# Resolve the site dir from this script's location so it runs from anywhere.
SITE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_DIR="$SITE_DIR/tools"

# Find typst: $TYPST, then PATH, then the offline-install location used in dev.
TYPST="${TYPST:-$(command -v typst || true)}"
if [ -z "$TYPST" ]; then
  TYPST="$HOME/.local/bin/typst"
fi
if [ ! -x "$TYPST" ]; then
  echo "error: typst not found (set \$TYPST or install to ~/.local/bin/typst)" >&2
  exit 1
fi

echo "==> generating .typ sources from content/"
python3 "$TOOLS_DIR/gen_cv.py"

echo "==> compiling PDFs with $TYPST"
"$TYPST" compile "$TOOLS_DIR/cv-en.typ" "$TOOLS_DIR/cv-en.pdf"
"$TYPST" compile "$TOOLS_DIR/cv-fr.typ" "$TOOLS_DIR/cv-fr.pdf"

echo "==> done:"
for f in cv-en cv-fr; do
  pages="?"
  if command -v pdfinfo >/dev/null 2>&1; then
    pages=$(pdfinfo "$TOOLS_DIR/$f.pdf" 2>/dev/null | awk '/^Pages:/{print $2}')
  fi
  printf '   %s.pdf  (%s pages)\n' "$f" "$pages"
done
