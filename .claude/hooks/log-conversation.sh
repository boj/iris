#!/usr/bin/env bash
# Stop hook: append the last human<->assistant exchange to HUMAN-CONVERSATIONS.md
# Reads session_id from stdin JSON, finds transcript, extracts last exchange.

CONV_FILE="/home/bojo/Development/ailang/HUMAN-CONVERSATIONS.md"
PROJECT_DIR="/home/bojo/.claude/projects/-home-bojo-Development-ailang"

# Read session_id from hook stdin
INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)

if [ -z "$SESSION_ID" ]; then
  exit 0
fi

TRANSCRIPT="${PROJECT_DIR}/${SESSION_ID}.jsonl"

if [ ! -f "$TRANSCRIPT" ]; then
  exit 0
fi

# Extract last human-typed message (content is a string, not array/tool-result)
# Use a temp file to avoid SIGPIPE from head -1 in pipeline
LAST_HUMAN=$(jq -r 'select(.type == "user" and (.message.content | type == "string")) | .message.content' "$TRANSCRIPT" 2>/dev/null | tail -1)

# Extract last assistant text blocks from the final assistant message
LAST_ASSISTANT=$(tac "$TRANSCRIPT" 2>/dev/null | jq -r 'select(.type == "assistant") | [.message.content[]? | select(.type == "text") | .text] | join("\n")' 2>/dev/null | sed '/^$/d' | head -1)

# Skip if nothing meaningful to log
if [ -z "$LAST_HUMAN" ] || [ "$LAST_HUMAN" = "null" ]; then
  exit 0
fi

# Skip continuation summaries
case "$LAST_HUMAN" in
  "This session is being continued"*) exit 0 ;;
esac

# Truncate very long assistant responses to first 500 chars
if [ ${#LAST_ASSISTANT} -gt 500 ]; then
  LAST_ASSISTANT="${LAST_ASSISTANT:0:500}..."
fi

# Append to conversation file
{
  echo ""
  echo "---"
  echo ""
  echo "**$(date '+%Y-%m-%d %H:%M')**"
  echo ""
  echo "**Human:** $LAST_HUMAN"
  echo ""
  if [ -n "$LAST_ASSISTANT" ] && [ "$LAST_ASSISTANT" != "null" ]; then
    echo "**Assistant:** $LAST_ASSISTANT"
    echo ""
  fi
} >> "$CONV_FILE"
