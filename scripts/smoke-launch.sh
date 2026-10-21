#!/usr/bin/env bash
# Starts the built app, checks it is still running after a few seconds, then stops it.
# Usage: scripts/smoke-launch.sh path/to/keepawake-app [seconds]
set -u

app=$1
seconds=${2:-10}
log=$(mktemp)

"$app" >"$log" 2>&1 &
pid=$!
sleep "$seconds"

if kill -0 "$pid" 2>/dev/null; then
  kill "$pid"
  wait "$pid" 2>/dev/null
  status=0
else
  wait "$pid"
  echo "KeepAwake exited early with code $?"
  status=1
fi

cat "$log"
if grep -q "panicked" "$log"; then
  echo "KeepAwake panicked"
  status=1
fi
exit $status
