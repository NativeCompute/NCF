#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")/.."
files=$(git ls-files --cached --others --exclude-standard -- '*.rs' '*.tmpl' '*.js' '*.css' '*.html' | grep -v '^target/')
bad=0
for f in $files; do
  case "$f" in
    *.html) hits=$(grep -nE '<!--' "$f") ;;
    *.css) hits=$(grep -nE '/\*' "$f" | grep -vE '/\* [A-Za-z0-9 _:()./-]+ here(---)? \*/') ;;
    *) hits=$(grep -nE '(^|[^:"])//' "$f" | grep -vE '//[^"]*"' | grep -vE '^[0-9]+:\s*// [A-Za-z0-9 _:()./-]+ here(---)?\s*$') ;;
  esac
  if [ -n "$hits" ]; then
    echo "$f"
    echo "$hits" | sed 's/^/  /'
    bad=1
  fi
done
if [ "$bad" = 1 ]; then
  echo "Comments found. Use a lazy marker such as '// thing part here' or move the explanation to docs/."
  exit 1
fi
echo "No comments."
