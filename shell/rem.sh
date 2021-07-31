#! /bin/bash

# TODO:
# restore to specific location
# date-based selection
# sudo mode

ROOT="${REM_ROOT:-$HOME/.trash}"
LOGFILE="$ROOT/history"
LOCK="$ROOT/lock"
REGISTRY="$ROOT/registry"
LOCATION="$HOME/bin/Rem"
HELP_LOC="$LOCATION/help.fmt"
SANDBOX=
OVERWRITE=
RANDOM=$( date +%N | sed 's,^0*,,' )
CRITICAL=1


REM_ENV=1

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

load_ext() {
    local f="$LOCATION/shell/rem_$1.sh"
    if [ -e "$f" ]; then
        . "$f"
    else
        efmt "${Bold}${Red}Unable to load extension ${Blue}'$1'${Blue}"
        efmt "  File ${Bold}${Blue}'$f'${__} does not exist"
        exit 7
    fi
}

UNIMPLEMENTED() {
    efmt "${Bold}${Red}Unimplemented action '$1'"
    exit 200
}

if [[ "$HOME" == /root ]]; then
    efmt "${Bold}${Red}You should not run Rem as root"
    efmt "  Stick to rm and mv for such critical operations"
    exit 25
fi
if ! [ -e "$LOCATION" ]; then
    efmt "${Bold}${Red}Rem is not installed at '$LOCATION'"
    efmt "  This could lead to problems for dynamically loading modules"
    exit 26
fi
case "$REM_LS" in
    (exa|ls)
        if ! which "$REM_LS" &>/dev/null; then
            efmt "${Bold}${Red}REM_LS is set to '$REM_LS' yet the corresponding"
            efmt "${Bold}${Red}command is not installed"
            exit 30
        fi
        ;;
    ('')
        if which exa &>/dev/null; then
            export REM_LS=exa
        else # I don't think we need to check that ls in installed...
            export REM_LS=ls
        fi
        ;;
    (*) efmt "${Bold}${Red}Not an ls-type command: ${Green}'REM_LS=$REM_LS'"
        efmt "  Use either ${Purple}'ls'${__} or ${Purple}'exa'${__}"
        exit 31
        ;;
esac
case "$REM_FZF" in
    (fzf|sk)
        if ! which "$REM_FZF" &>/dev/null; then
            efmt "${Bold}${Red}REM_FZF is set to '$REM_FZF' yet the corresponding"
            efmt "${Bold}${Red}command is not installed"
            exit 30
        fi
        ;;
    ('')
        if which sk &>/dev/null; then
            export REM_FZF=sk
        elif which fzf &>/dev/null; then
            export REM_FZF=fzf
        fi;;
    (*) efmt "${Bold}${Red}Not an fzf-type command: ${Green}'REM_FZF=$REM_FZF'"
        eftm "  Use either ${Purple}'fzf'${__} or ${Purple}'sk'${__}"
        exit 31
        ;;
esac

randname() {
    echo "$RANDOM$RANDOM$RANDOM" | base64
}

newname() {
    # Randomly generate until name is unused
    local name
    name=''
    until [ -n "$name" ] && ! [ -e "$REGISTRY/$name" ]; do
        name=`randname`
    done
    echo "$name"
}

file_actual() {
    # Extract true file name from 'alias|name|timestamp'
    echo "$1" | cut -d'|' -f2
}
file_aliased() {
    # Extract alias from 'alias|name|timestamp'
    echo "$1" | cut -d'|' -f1
}
file_timestamp() {
    # extract timestamp from 'alias|name|timestamp'
    echo "$1" | cut -d'|' -f3
}

check_install() {
    # Create ~/.trash if missing
    mkdir -p "$ROOT"
    mkdir -p "$REGISTRY"
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
    local source="${2//\'/"'\"'\"'"}"
    local infofile="$REGISTRY/$name/meta"
    refpoint "Create '$infofile'"
    critical "basename '$source' > '$infofile'"
    critical "date '+%Y-%m-%d %H:%M:%S' >> '$infofile'"
    critical "echo '' >> '$infofile'"
    critical "${REM_LS} -Flah --color=always '$source' >> '$infofile'"
    critical "echo '' >> '$infofile'"
    critical "file '$source' | sed 's,:,\n,' >> '$infofile'"
}

del_register() {
    # Remove and add to registry
    local source=$( readlink -m "$1" )
    if [[ "$source" == "$HOME" ]]; then
        efmt "${Bold}${Red}Cannot move out home directory"
        exit 15
    fi
    if [ -e "$source" ]; then
        local name=`newname`
        refpoint "Register $source as $name"
        critical "mkdir '$REGISTRY/$name'"
        critical "echo '$name|${source//\'/"'\"'\"'"}|$( date '+%s' )' >> '$LOGFILE'"
        make_info "$name" "$source"
        critical "mv '${source//\'/"'\"'\"'"}' '$REGISTRY/$name/file'"
    else
        echo "$source not found             (skipping)"
    fi
}

help_fmt() {
    local f="$LOCATION/help/$1.ansi"
    if [ -e "$f" ]; then
        cat "$f"
    elif [ -e "$LOCATION/help/main.ansi" ]; then
        efmt "${Bold}${Red}No such help menu ${Green}'$1'"
        efmt "  Try one of"
        efmt "    examples, cmd, select,"
        efmt "    info, rest, undo, del,"
        efmt "    pat, fzf, idx"
        efmt "  or leave blank"
        exit 100
    else
        efmt "${Bold}${Red}No help generated"
        efmt "  Try going to $LOCATION/help and running 'make'"
        exit 200
    fi
}

print_help() {
    if [ -z "$1" ]; then
        help_fmt 'main'
        return
    fi
    while [ -n "$1" ]; do
        help_fmt "$1"
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
        echo "Can only execute one command at a time"
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

empty_selectors() {
    efmt "${Bold}${Red}Selector list is empty"
    efmt "  $1 should take at least one selector"
    exit 13
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
                [ -v 1 ] || empty_selectors pat  
                SELECT_PAT+=( "$1" ); shift
                until [[ "$1" =~ ^(-.*|)$ ]]; do
                    SELECT_PAT+=( "$1" ); shift
                done
                HAS_SELECTION=1
                ;;
            (-I|--idx) check_file idx
                [ -v 1 ] || empty_selectors idx
                SELECT_IDX+=( "$1" ); shift
                until [[ "$1" =~ ^(-.*|)$ ]]; do
                    SELECT_IDX+=( "$1" ); shift
                done
                HAS_SELECTION=1
                ;;
            (-T|--time) check_file time
                [ -v 1 ] || empty_selectors time
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

if [ -n "$SELECT_FZF" ] && [ -z "$REM_FZF" ]; then
    efmt "${Bold}${Red}REM_FZF is unset and neither fzf nor sk is installed"
    exit 25
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
    load_ext 'query'
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
            load_ext 'edit'
            for file in "${SELECT[@]}"; do
                print_info "$( echo "$file" | cut -d' ' -f1 )"
            done
            exit 0
            ;;
        (undo)
            load_ext 'edit'
            undo_del
            ;;
        (del)
            load_ext 'edit'
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
            load_ext 'edit'
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
[ -n "$HAS_SELECTION" ] && select_files
execute
[ -n "$CRITICAL" ] && unlock_critical_section
if [ -n "$SANDBOX" ]; then
    fmt ""
    fmt "${Purple}### Exit sandbox"
    fmt "${Purple}### Nothing was modified"
fi
exit 0

