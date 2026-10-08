#!/usr/bin/env bash
set -eu

summary="book/SUMMARY.md"
test -s "$summary"
while IFS= read -r line; do
    case "$line" in
        *']('*')'*)
            path="${line#*](}"
            path="${path%)}"
            test -s "book/$path"
            ;;
    esac
done < "$summary"
