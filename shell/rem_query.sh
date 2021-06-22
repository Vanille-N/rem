#! /bin/bash

# This is the 'query' extension to Rem
# It allows browsing the removed files

[ -n "$REM_ENV" ] || exit 33

list_pat() {
    local pat="$1"
    pat=".*$pat.*" # note that empty pattern matches anything
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        local actual="$( file_actual "$line" )"
        if eval "[[ \"$actual\" =~ $pat ]]"; then
            echo "$index $actual"
        fi
    done
}

list_fzf() {
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        local actual="$( file_actual "$line" )"
        echo "$index $actual"
    done |
    sed "s,$HOME,~," |
    ${REM_FZF} --multi \
        --preview='\
            rem --info \
                --idx "$( echo {} | cut -d" " -f1 )" \
            | sed '"$HOME_SUB" \
        --preview-window=up
}

list_idx() {
    local start="$( echo "$1" | cut -d':' -f1 )"
    local end="$( echo "$1" | cut -d':' -f2 )"
    start="${start:-1}"
    end="${end:-0}"
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        if (( $index < $start )); then continue; fi
        local actual="$( file_actual "$line" )"
        echo "$index $actual"
        if (( $index == $end )); then break; fi
    done
}

DUR_MINUTE=60
DUR_HOUR=$(( DUR_MINUTE * 60 ))
DUR_DAY=$(( DUR_HOUR * 24 ))
DUR_WEEK=$(( DUR_DAY * 7 ))
DUR_MONTH=$(( DUR_DAY * 30 ))
DUR_YEAR=$(( DUR_DAY * 365 ))
interprete_timeframe() {
    local curr=0
    local acc=0
    while read -n1 c; do
        if [[ $c =~ [[:digit:]] ]]; then
            let 'curr = curr * 10 + c'
        else
            case "$c" in
                (' '|"\t"|"\n"|'') continue;;
                (s) let 'acc += (curr ? curr : 1)';;
                (m) let 'acc += (curr ? curr : 1) * DUR_MINUTE';;
                (h) let 'acc += (curr ? curr : 1) * DUR_HOUR';;
                (d) let 'acc += (curr ? curr : 1) * DUR_DAY';;
                (W) let 'acc += (curr ? curr : 1) * DUR_WEEK';;
                (M) let 'acc += (curr ? curr : 1) * DUR_MONTH';;
                (Y) let 'acc += (curr ? curr : 1) * DUR_YEAR';;
                (*) efmt "${Bold}${Red}Not a valid duration: ${Green}'$c'"
                    efmt "  Use one of s,m,h,d,W,M,Y"
                    exit 110
                    ;;
            esac
            let 'curr = 0'

        fi
    done
    let 'acc += curr * DUR_DAY'
    echo "$acc"
}

list_time() {
    local current="$( date '+%s' )"
    local dt_old="$( echo "$1" | cut -d':' -f2 | interprete_timeframe )"
    [ -z "$dt_old" ] && exit 110
    local dt_new="$( echo "$1" | cut -d':' -f1 | interprete_timeframe )"
    [ -z "$dt_new" ] && exit 110
    local old=$(( current - dt_old ))
    local new=$(( current - dt_new ))
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        local actual="$( file_actual "$line" )"
        local tdel="$( file_timestamp "$line" )"
        if (( old <= new )); then
            # Normal interval check
            if (( old <= tdel )) && (( tdel <= new )); then
                echo "$index $actual"
            fi
        else
            # Inverted interval
            if (( old < tdel )) || (( tdel < new )); then
                echo "$index $actual"
            fi
        fi
    done
}

