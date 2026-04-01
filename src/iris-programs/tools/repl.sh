#!/usr/bin/env bash
# IRIS REPL — Interactive Read-Eval-Print Loop
# Uses iris-stage0 to compile and evaluate each expression.
#
# Supports:
#   - Single expressions: 2 + 3
#   - Let bindings accumulate: let x = 5 (then x + 1)
#   - Multi-line: end with \ to continue
#   - :quit or :q to exit
#   - :reset to clear bindings
#   - :load <file> to evaluate a file
#   - :type <expr> to show result type

STAGE0="${STAGE0:-bootstrap/iris-stage0}"
RUNNER="src/iris-programs/interpreter/iris_run.iris"
BINDINGS=""
COUNTER=0

echo "IRIS REPL v0.1"
echo "Type expressions to evaluate. :q to quit, :reset to clear."
echo ""

while true; do
    # Prompt
    printf "iris> "
    read -r line
    [ $? -ne 0 ] && break  # EOF

    # Handle commands
    case "$line" in
        :quit|:q)
            echo "Bye."
            exit 0
            ;;
        :reset)
            BINDINGS=""
            COUNTER=0
            echo "Bindings cleared."
            continue
            ;;
        :load\ *)
            file="${line#:load }"
            if [ -f "$file" ]; then
                result=$("$STAGE0" run "$RUNNER" "$file" 0 2>&1)
                echo "$result"
            else
                echo "File not found: $file"
            fi
            continue
            ;;
        "")
            continue
            ;;
    esac

    # Handle multi-line (trailing \)
    while [[ "$line" == *\\ ]]; do
        line="${line%\\}"
        printf "  ... "
        read -r cont
        line="$line$cont"
    done

    # Check if it's a let binding
    if [[ "$line" == let\ * ]]; then
        # Accumulate binding
        BINDINGS="$BINDINGS
$line"
        # Try to evaluate to check for errors
        src="${BINDINGS}
let __repl_main__ x = 0"
        tmpfile=$(mktemp /tmp/iris_repl_XXXXXX.iris)
        echo "$src" > "$tmpfile"
        result=$("$STAGE0" run "$RUNNER" "$tmpfile" 0 2>&1)
        rm -f "$tmpfile"
        if echo "$result" | grep -q "error"; then
            echo "Error: $result"
            # Remove the failed binding
            BINDINGS=$(echo "$BINDINGS" | head -n -1)
        else
            name=$(echo "$line" | sed 's/let \([a-zA-Z_][a-zA-Z0-9_]*\).*/\1/')
            echo "  $name defined"
        fi
    else
        # Expression: wrap in main and evaluate
        COUNTER=$((COUNTER + 1))
        src="${BINDINGS}
let __repl_main__ x = $line"
        tmpfile=$(mktemp /tmp/iris_repl_XXXXXX.iris)
        echo "$src" > "$tmpfile"
        result=$("$STAGE0" run "$RUNNER" "$tmpfile" 0 2>&1)
        rm -f "$tmpfile"
        if echo "$result" | grep -q "error"; then
            # Try direct run (simpler programs)
            echo "$src" > "$tmpfile.iris"
            result=$("$STAGE0" run "$tmpfile.iris" 0 2>&1)
            rm -f "$tmpfile.iris"
        fi
        echo "= $result"
    fi
done
