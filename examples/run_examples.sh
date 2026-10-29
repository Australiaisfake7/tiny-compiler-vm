#!/usr/bin/env bash
#
# Compiles and runs every example program.
#
#   ./run_examples.sh [path/to/tiny-compiler-vm] [--check | --update]
#
#   (no flag)  print the output of every example
#   --check    compare each output with the file in expected/ and report PASS or FAIL
#   --update   rewrite the files in expected/ from the current output
#
# The compiler binary is taken from, in this order: the first argument, the TLVM
# environment variable, ../target/release/tiny-compiler-vm, ../target/debug/tiny-compiler-vm,
# and finally `tiny-compiler-vm` on the PATH.

set -u
cd "$(dirname "$0")"

BIN=""
MODE="run"
for arg in "$@"; do
    case "$arg" in
        --check)  MODE="check" ;;
        --update) MODE="update" ;;
        *)        BIN="$arg" ;;
    esac
done

if [ -z "$BIN" ]; then
    for candidate in "${TLVM:-}" ../target/release/tiny-compiler-vm ../target/debug/tiny-compiler-vm; do
        if [ -n "$candidate" ] && [ -x "$candidate" ]; then BIN="$candidate"; break; fi
    done
fi
if [ -z "$BIN" ] && command -v tiny-compiler-vm >/dev/null 2>&1; then
    BIN="tiny-compiler-vm"
fi
if [ -z "$BIN" ]; then
    echo "Could not find tiny-compiler-vm. Build it with 'cargo build --release' or pass its path." >&2
    exit 2
fi

mkdir -p expected
passed=0
failed=0

# Compiles one program, runs it (feeding NAME.in to standard input when that file exists)
# and prints everything it produced, without the tool's own "Compiled"/"Executed" notes.
run_one() {
    local source="$1"
    local base="${source%.tl}"
    local input="/dev/null"
    [ -f "$base.in" ] && input="$base.in"

    {
        "$BIN" compile "$source" 2>&1
        if [ -f "$base.tb" ]; then
            "$BIN" execute "$base.tb" < "$input" 2>&1
            rm -f "$base.tb"
        fi
    } | grep -v -e '^Compiled ' -e '^Executed '
}

for source in [0-9]*.tl errors/*.tl; do
    name="$(basename "${source%.tl}")"
    expected="expected/$name.out"

    case "$MODE" in
        run)
            echo "=================== $source"
            run_one "$source"
            echo
            ;;
        update)
            run_one "$source" > "$expected"
            echo "updated $expected"
            ;;
        check)
            if [ -f "$expected" ] && [ "$(run_one "$source")" == "$(cat "$expected")" ]; then
                echo "PASS  $source"
                passed=$((passed + 1))
            else
                echo "FAIL  $source"
                failed=$((failed + 1))
            fi
            ;;
    esac
done

if [ "$MODE" == "check" ]; then
    echo
    echo "$passed passed, $failed failed"
    [ "$failed" -eq 0 ]
fi
