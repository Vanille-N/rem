#! /bin/bash

RColumn="\r\x1b[45C"
MColumn="\r\x1b[45C"


help_extract() {
    awk -e "/<$1>/"' { show = 1; next }' \
        -e '/<end>/ { show = 0 }' \
        -e 'show { print }' \
        'help.fmt'
}

help_color() {
    cat "parts/$1.fmt" |
    sed -E \
        -e 's,$,'"${__}"',; s,^  ,,' \
        -e 's,!###,   '"${Ital}${Green}," \
        -e 's,!##,'"${Bold}${Red}," \
        -e 's,!\+\+,'"${Bold}${Green}," \
        -e 's, *!#,'"${Bold}${Green}," \
        -e 's,&&&,'"${RColumn}," \
        -e 's,\?\?\?,'"${MColumn}${Ital}${Green}?," \
        -e 's,`(-[a-zA-Z_-]+)`,'"${Yellow}\\1${__},g" \
        -e 's,`\$:([a-z]+)`,'"${Purple}\\1${__},g" \
        -e 's,`([]()<>+|~[A-Z _.-]+)`,'"${Bold}${Blue}\\1${__},g" \
        -e 's,`'"('[^']*')"'`,'"${Green}\\1${__},g" \
        -e 's,`(#[^`]+)`,'"${Grey}\\1${__},g" \
        -e 's,<>((<[^>]|[^<]>|.)+)<>,'"${Ital}${Blue}\\1${__},g" \
        -e 's,`~*([^`]+)`,'"${__}\\1${__},g" \
        -e 's, ---,'"$( seq 65 | awk '{ printf "-" }' ),g" \
        -e 's,!--,'"${Ital},g"
}



MENUS=( $( grep -E '<[a-z]+>' 'help.fmt' | grep -v end | tr -d '<>' ) )
for m in ${MENUS[@]}; do
    help_fmt "$m"
done
