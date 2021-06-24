#! /bin/bash

# This is the 'edit' extension to Rem
# It allows performing actions on removed files

[ -n "$REM_ENV" ] || exit 33

get_line() {
    # Extract i'th entry of ~/.trash/history
    local linenum="$1"
    tac "$LOGFILE" |
    grep '|' |
    sed --quiet "${linenum}p"
} 

print_info() {
    local id="$( file_aliased "$( get_line "$1" )" )"
    critical "cat \"$REGISTRY/$id/meta\""
    if [ -z "$SANDBOX" ]; then
        echo "=============================================="
        echo ""
    fi
}

clean_history() {
    local id
    refpoint "Clean history"
    cat "$LOGFILE" |
    while read -t 0.05 line; do
        id="$( file_aliased "$line" )"
        [ -e "$REGISTRY/$id" ] && echo "$line"
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
    critical "mv \"$REGISTRY/$aliased/file\" \"$actual\""
    # Check that it actually worked before deleting the backup
    critical "[ -e \"$actual\" ] || exit 20"
    critical "rm -rf \"$REGISTRY/$aliased\""
}

undo_del() {
    : > "$LOGFILE.tmp"
    refpoint "Rollback last deletion"
    tac "$LOGFILE" |
    {
        while read -t 0.05 line; do
            [ -z "$line" ] && break
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
    local line="$( get_line "$linenum" )"
    local aliased="$( file_aliased "$line" )"
    local actual="$( file_actual "$line" )"
    refpoint "Delete permanently '$actual'"
    critical "rm -rf \"$REGISTRY/$aliased\""
}


