#!/usr/bin/env bash
# scripts/prism-test.sh
# Manages Prism mock server lifecycle for integration tests.
# Usage: ./scripts/prism-test.sh

set -euo pipefail

PRISM_PORT=4010
PRISM_URL="http://127.0.0.1:${PRISM_PORT}"
SPEC_FILE="docs/openapi.json"
MAX_WAIT=15

# ── Start Prism ──────────────────────────────────────────────
echo "Starting Prism mock server on port ${PRISM_PORT}..."
npx @stoplight/prism-cli mock "${SPEC_FILE}" -p "${PRISM_PORT}" --errors -d &
PRISM_PID=$!

# Ensure Prism is killed on exit regardless of outcome
cleanup() {
  echo "Stopping Prism (PID ${PRISM_PID})..."
  kill "${PRISM_PID}" 2>/dev/null || true
  wait "${PRISM_PID}" 2>/dev/null || true
}
trap cleanup EXIT

# ── Health check ─────────────────────────────────────────────
echo "Waiting for Prism to become healthy (max ${MAX_WAIT}s)..."
elapsed=0
while ! curl -so /dev/null "${PRISM_URL}" 2>/dev/null; do
  if [ "${elapsed}" -ge "${MAX_WAIT}" ]; then
    echo "ERROR: Prism did not become healthy within ${MAX_WAIT}s"
    exit 1
  fi
  sleep 1
  elapsed=$((elapsed + 1))
done
echo "Prism is healthy after ${elapsed}s."

# ── Run tests ────────────────────────────────────────────────
export PRISM_URL
echo "Running prism integration tests..."
cargo test --all-features --test prism -- --nocapture
TEST_EXIT=$?

# cleanup runs via trap
exit "${TEST_EXIT}"
