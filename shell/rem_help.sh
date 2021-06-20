#! /bin/bash

# Rem :: Help
# This file can be dynamically loaded by Rem to handle help printing
# It assumes being loaded from the main Rem script

[ -n "$REM_ENV" ] || exit 33

help_fmt() {
    awk -e "/<$1>/"' { show = 1; next }' \
        -e '/<end>/ { show = 0 }' \
        -e 'show { print }' \
        "$HELP_LOC" |
    sed -E \
        -e 's,$,'"${__}"',; s,^  ,,' \
        -e 's,!###,   '"${Ital}${Green}," \
        -e 's,!##,'"${Bold}${Red}," \
        -e 's, *!#,'"${Bold}${Green}," \
        -e 's,&&&,'"\r\x1b[45C," \
        -e 's,\?\?\?,'"\r\x1b[30C${Ital}${Green}?," \
        -e 's,`(-[a-zA-Z_-]+)`,'"${Yellow}\\1${__},g" \
        -e 's,`\$:([a-z]+)`,'"${Purple}\\1${__},g" \
        -e 's,`([]()<>+|~[A-Z .-]+)`,'"${Bold}${Blue}\\1${__},g" \
        -e 's,`'"('[^']*')"'`,'"${Green}\\1${__},g" \
        -e 's,`(#[^`]+)`,'"${Grey}\\1${__},g" \
        -e 's,<>((<[^>]|[^<]>|.)+)<>,'"${Ital}${Blue}\\1${__},g" \
        -e 's,`~*([^`]+)`,'"${__}\\1${__},g" \
        -e 's, ---,'"$( seq 65 | awk '{ printf "-" }' ),g" \
        -e 's,!--,'"${Ital},g"
}

print_help() {
    if [ -z "$1" ]; then
        help_fmt 'overview'
        return
    fi
    while [ -n "$1" ]; do
        if [[ "$1" =~ [a-z]+ ]] && grep "<$1>" "$HELP_LOC"; then
            help_fmt "$1"
        else
            efmt "${Bold}${Red}No such help menu ${Green}'$1'"
            efmt "  Try one of"
            efmt "    example, cmd, select,"
            efmt "    info, rest, undo, del,"
            efmt "    pat, fzf, idx"
            efmt "  or leave blank"
        fi
        shift
        if [ -n "$1" ]; then
            seq `tput cols` | awk 'BEGIN { print "" } { printf "=" } END { print "\n" }'
        fi
    done
}

