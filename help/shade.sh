#! /bin/bash

RColumn="\r\x1b[45C"
MColumn="\r\x1b[30C"
esc_code() {
    echo "\x1b[${1}m"
}
Green=`esc_code 92`
Red=`esc_code 91`
Blue=`esc_code 94`
Yellow=`esc_code 93`
Grey=`esc_code 97`
Purple=`esc_code 95`
Bold=`esc_code 1`
Ital=`esc_code 3`
__=`esc_code 0`

cat "$1" |
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
    -e 's,`([]()<>+|[A-Z _.:]+)`,'"${Bold}${Blue}\\1${__},g" \
    -e 's,`'"('[^']*')"'`,'"${Green}\\1${__},g" \
    -e 's,`(#[^`]+)`,'"${Grey}\\1${__},g" \
    -e 's,<>((<[^>]|[^<]>|.)+)<>,'"${Ital}${Blue}\\1${__},g" \
    -e 's,`~*([^`]+)`,'"${__}\\1${__},g" \
    -e 's, ---,'"$( seq 65 | awk '{ printf "-" }' ),g" \
    -e 's,!--,'"${Ital},g"

