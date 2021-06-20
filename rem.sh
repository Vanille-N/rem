#! /bin/bash

# TODO:
# restore to specific location
# date-based selection

TRASH="$HOME/.trash"
LOGFILE="$TRASH/history"
LOCK="$TRASH/lock"
HELP_LOC="$HOME/bin/Rem/rem.fmt"
SANDBOX=
OVERWRITE=
RANDOM=$( date +%N | sed 's,^0*,,' )
IDENT="$$"
CRITICAL=1

HOME_SUB="s,$HOME,~,"

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
fmt() {
    echo -e "$1$__"
}
efmt() {
    fmt "$1" &>/dev/stderr
}

lock_critical_section() {
    if [ -e "$LOCK" ]; then
        efmt "${Bold}${Red}Another Rem process is running"
        efmt "Please terminate it before running Rem again"
        exit 111
    else
        touch "$LOCK"
    fi
}
unlock_critical_section() {
    rm "$LOCK"
}

UNIMPLEMENTED() {
    efmt "${Bold}${Red}Unimplemented action '$1'"
    exit 200
}

randname() {
    echo "$RANDOM$RANDOM$RANDOM" | base64
}

newname() {
    # Randomly generate until name is unused
    local name
    name=''
    until [ -n "$name" ] && ! [ -e "$TRASH/$name" ]; do
        name=`randname`
    done
    echo "$name"
}

file_actual() {
    # Extract true file name from 'alias|name'
    echo "$1" | cut -d'|' -f2
}
file_aliased() {
    # Extract alias from 'alias|name'
    echo "$1" | cut -d'|' -f1
}

check_install() {
    # Create ~/.trash if missing
    mkdir -p "$TRASH"
    : >> "$LOGFILE"
}

critical() {
    # When in sandbox mode, print commands
    # otherwise execute them for real
    if [ -n "$SANDBOX" ]; then
        echo -en "    ${Grey}"
        echo -n "${1//$HOME/\~}"
        echo -e "${__}"
    else
        eval "$1"
    fi
}

refpoint() {
    # Progress marker when in sandbox mode
    if [ -n "$SANDBOX" ]; then
        fmt "${Ital}>>> $1"
    fi
}

make_info() {
    local name="$1"
    local source="$2"
    local infofile="$TRASH/$name.info"
    refpoint "Create $infofile"
    critical "basename \"$source\" > \"$infofile\""
    critical "date \"+%Y-%m-%d %H:%M:%S\" >> \"$infofile\""
    critical "echo \"\" >> \"$infofile\""
    critical "exa -lh --color=always \"$source\" >> \"$infofile\""
    critical "echo \"\" >> \"$infofile\""
    critical "file \"$source\" | sed 's,:,\\n,' >> \"$infofile\""
}

del_register() {
    # Remove and add to registry
    local source="`pwd`/$1"
    if [ -e "$source" ]; then
        local name=`newname`
        refpoint "Register $source as $name"
        critical "echo \"$name|$source\" >> \"$LOGFILE\""
        make_info "$name" "$source"
        critical "mv \"$source\" \"$TRASH/$name\""
    else
        echo "$source not found             (skipping)"
    fi
}

get_line() {
    # Extract i'th entry of ~/.trash/history
    local linenum="$1"
    tac "$LOGFILE" |
    grep '|' |
    sed --quiet "${linenum}p"
} 

print_info() {
    local id="$( file_aliased "$( get_line "$1" )" )"
    critical "cat \"$TRASH/$id.info\""
    if [ -z "$SANDBOX" ]; then
        echo "=============================================="
        echo ""
    fi
}

list_pat() {
    local pat="$1"
    pat=".*$pat.*" # note that empty pattern matches anything
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        local actual="$( file_actual "$line" )"
        local aliased="$( file_aliased "$line" )"
        if [ -e "$TRASH/$aliased" ]; then
            if eval "[[ \"$actual\" =~ $pat ]]"; then
                echo "$index $actual"
            fi
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
        local aliased="$( file_aliased "$line" )"
        echo "$index $actual"
    done |
    sed "s,$HOME,~," |
    fzf --multi \
        --preview='\
            rem --info \
                --idx "$( echo {} | cut -d" " -f1 )" \
            | sed '"$HOME_SUB" \
        --preview-window=up
}

list_idx() {
    local start="$( echo "$1" | cut -d'-' -f1 )"
    local end="$( echo "$1" | cut -d'-' -f2 )"
    start="${start:-1}"
    end="${end:-0}"
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        if (( $index < $start )); then continue; fi
        local actual="$( file_actual "$line" )"
        local aliased="$( file_aliased "$line" )"
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
    local dt_old="$( echo "$1" | cut -d'-' -f2 | interprete_timeframe )"
    local dt_new="$( echo "$1" | cut -d'-' -f1 | interprete_timeframe )"
    local old=$(( current - dt_old ))
    local new=$(( current - dt_new ))
    local index=0
    tac "$LOGFILE" |
    grep '|' |
    while read -t 0.05 line; do
        let '++index'
        local actual="$( file_actual "$line" )"
        local aliased="$( file_aliased "$line" )"
        deleted="$( sed -n '2p' "$TRASH/$aliased.info" )"
        tdel="$( date -d "$deleted" '+%s' )"
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

clean_history() {
    local id
    refpoint "Clean history"
    cat "$LOGFILE" |
    while read -t 0.05 line; do
        id="$( file_aliased "$line" )"
        [ -e "$TRASH/$id" ] && echo "$line"
    done |
    awk 'BEGIN { RS="\n{2,}" } { print "\n" $0 }' > "$LOGFILE.tmp"
    critical "mv \"$LOGFILE.tmp\" \"$LOGFILE\""
}

restore_file() {
    local actual="$( file_actual "$1" )"
    local aliased="$( file_aliased "$1" )"
    refpoint "Restore $actual"
    critical "mkdir -p \"$( dirname "$actual" )\""
    if [ -z "$OVERWRITE" ] && [ -e "$actual" ]; then
        local id=0
        while [ -e "$actual.$id" ]; do
            let 'id++'
        done
        echo "File '$actual' already exists, using '$actual.$id' instead"
        actual+=".$id"
    fi
    echo "$actual"
    critical "mv \"$TRASH/$aliased\" \"$actual\""
    critical "rm \"$TRASH/$aliased.info\""
}
    

undo_del() {
    : > "$LOGFILE.tmp"
    refpoint "Rollback last deletion"
    tac "$LOGFILE" |
    {
        while read -t 0.05 line; do
            [[ -z "$1" ]] && break
            restore_file "$line"
        done
        [ -n "$SANDBOX" ] && return
        while read -t 0.05 line; do
            echo "$line" >> "$LOGFILE.tmp"
        done
    }
    critical "tac \"$LOGFILE.tmp\" > \"$LOGFILE\""
    critical "rm \"$LOGFILE.tmp\""
}

restore_index() {
    local line="$( get_line "$1" )"
    restore_file "$line"
}

permanent_remove() {
    local linenum="$( echo "$1" | cut -d" " -f1 )"
    local aliased="$( file_aliased "$( get_line "$linenum" )" )"
    refpoint "Delete permanently '$aliased'"
    critical "rm -r \"$TRASH/$aliased.info\""
    critical "rm -rf \"$TRASH/$aliased\""
}

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

CMD=
SELECT_PAT=()
SELECT_IDX=()
SELECT_TIME=()
SELECT_FZF=
HAS_SELECTION=
FILES=()

check_dup() {
    if [ -n "$CMD" ]; then
        efmt "${Bold}${Red}Duplicate command ${Yellow}'$1'${Red} provided: ${Ital}${Yellow}'$CMD'${Red} already registered"
        exit 10
    fi
}

check_file() {
    if (( "${#FILES[@]}" != 0 )); then
        efmt "${Bold}${Red}Cannot execute command ${Yellow}'$1'${Red}: ${Ital}there are files registered for deletion"
        efmt "Deletion or auxiliary commands must be exclusive"
        exit 11
    fi
}

check_aux() {
    if [ -n "$CMD" ] && [[ "$CMD" != "help" ]]; then
        efmt "${Bold}${Red}Cannot register ${Green}'$1'${Red} for deletion: ${Ital}there is a command waiting"
        efmt "Deletion or auxiliary commands must be exclusive"
    fi
}

parse_args() {
    while [ -n "$1" ]; do
        local arg="$1"
        shift
        case "$arg" in
            (-h|--help) check_dup help; CMD=help; CRITICAL=;;
            (-u|--undo) check_dup undo; check_file undo; CMD=undo;;
            (-r|--rest) check_dup rest; check_file rest; CMD=rest;;
            (-i|--info) check_dup info; check_file info; CMD=info; CRITICAL=;;
            (-d|--del) check_dup del; check_file del; CMD=del;;
            (-F|--fzf) check_file fzf; SELECT_FZF=1; HAS_SELECTION=1;;
            (-P|--pat) check_file pat
                SELECT_PAT+=( "$1" ); shift
                until [[ "$1" =~ ^(-.*|)$ ]]; do
                    SELECT_PAT+=( "$1" ); shift
                done
                HAS_SELECTION=1
                ;;
            (-I|--idx) check_file idx
                SELECT_IDX+=( "$1" ); shift
                while [ -n "$1" ] && [[ "$1" =~ ^([0-9]*-?[0-9]*)$ ]]; do
                    SELECT_IDX+=( "$1" ); shift
                done
                HAS_SELECTION=1
                ;;
            (-T|--time) check_file time
                SELECT_TIME+=( "$1" ); shift
                until [[ "$1" =~ ^(-.*|)$ ]]; do
                    SELECT_TIME+=( "$1" ); shift
                done
                HAS_SELECTION=1
                ;;
            (-S|--sandbox) SANDBOX=1; CRITICAL=;;
            (-O|--overwrite) OVERWRITE=1;;
            (--)
                check_aux "--"
                while [ -n "$1" ]; do
                    FILES+=( "$1" )
                    shift
                done
                ;;
            (-*)
                efmt "${Bold}${Red}Unknown argument ${Yellow}'$arg'"
                exit 13
                ;;
            (*) check_aux "$arg"; FILES+=( "$arg" );;
        esac
    done
    [ -z "$CMD" ] && CRITICAL=
}

check_install
parse_args "$@"
if [ -n "$SANDBOX" ]; then
    fmt "command        ${Yellow}${CMD:--}"
    fmt "fzf            ${Yellow}${SELECT_FZF:--}"
    fmt "pat            ${Yellow}${SELECT_PAT[@]:--}"
    fmt "idx            ${Yellow}${SELECT_IDX[@]:--}"
    fmt "time           ${Yellow}${SELECT_TIME[@]:--}"
    fmt "files          ${Yellow}${FILES[@]:--}"
    fmt ""
    fmt "${Purple}### This is the sandbox mode"
    fmt "${Purple}### None of the below commands are actually executed"
    fmt "${Purple}### Actual commands outside of sandbox mode may differ slightly"
    fmt ""
fi

if [[ "$CMD" == undo ]] || [[ "$CMD" == help ]]; then
    if [ -n "$HAS_SELECTION" ]; then
        efmt "${Bold}${Red}'$CMD' should have no selection"
        efmt "Remove any uses of ${Yellow}--pat${__}, ${Yellow}--idx${__}, ${Yellow}--fzf${__}"
        exit 12
    fi
fi

SELECT=()
select_files() {
    shopt -s lastpipe
    {
        for i in "${SELECT_IDX[@]}"; do
            list_idx "$i" | sed "$HOME_SUB"
        done
        for p in "${SELECT_PAT[@]}"; do
            list_pat "$p" | sed "$HOME_SUB"
        done
        for t in "${SELECT_TIME[@]}"; do
            list_time "$t" | sed "$HOME_SUB"
        done
        if [ -n "$SELECT_FZF" ]; then
            list_fzf
        fi
    } | sort -rn | uniq |
    while read line; do
        SELECT+=( "$line" )
    done
}

execute() {
    case "$CMD" in
        (info)
            for file in "${SELECT[@]}"; do
                print_info "$( echo "$file" | cut -d' ' -f1 )"
            done
            exit 0
            ;;
        (undo)
            undo_del
            ;;
        (del)
            (( "${#SELECT[@]}" == 0 )) && return
            if [ -z "$SANDBOX" ]; then
                fmt "This action will ${Bold}${Ital}_permanently_${__} delete"
                for file in "${SELECT[@]}"; do
                    echo "    $file"
                done
                fmt "${Bold}${Blue}Continue ? (y/N)"
                read -rsn1 kp
            else
                kp="y"
            fi
            if [[ "$kp" == "y" ]]; then
                for file in "${SELECT[@]}"; do
                    permanent_remove "$file"
                done
                clean_history
            else
                fmt "${Bold}${Red}Aborted"
            fi
            ;;
        (rest)
            for file in "${SELECT[@]}"; do
                restore_index "$( echo "$file" | cut -d' ' -f1 )"
            done
            clean_history
            ;;
        (help)
            print_help "${FILES[@]}"
            exit 0
            ;;
        ('')
            if [ -n "$HAS_SELECTION" ]; then
                for file in "${SELECT[@]}"; do
                    echo "$file"
                done
            else
                [ -z "$SANDBOX" ] && echo "" >> $LOGFILE
                for file in "${FILES[@]}"; do
                del_register "$file"
                done
            fi
            ;;
    esac
}
[ -n "$CRITICAL" ] && lock_critical_section
select_files
execute
[ -n "$CRITICAL" ] && unlock_critical_section
if [ -n "$SANDBOX" ]; then
    fmt ""
    fmt "${Purple}### Exit sandbox"
    fmt "${Purple}### Nothing was modified"
fi
exit 0

