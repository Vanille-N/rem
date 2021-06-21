#! /bin/bash

awk -e "/<$1>/"' { show = 1; next }' \
    -e '/<end>/ { show = 0 }' \
    -e 'show { print }' \
    'help.fmt'

