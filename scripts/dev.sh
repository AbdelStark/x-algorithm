#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
START_DIR="$(pwd -P)"

HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-8787}"
PHOENIX_PORT="${PHOENIX_PORT:-8000}"
WEB_HOST="${WEB_HOST:-127.0.0.1}"
WEB_PORT="${WEB_PORT:-5173}"
LOG_DIR="${LOG_DIR:-}"
RUN_ID="$(date '+%Y%m%d_%H%M%S')_${RANDOM}"
CURL_TIMEOUT="${CURL_TIMEOUT:-10}"
SKIP_UV_SYNC="${SKIP_UV_SYNC:-0}"
SKIP_NPM_INSTALL="${SKIP_NPM_INSTALL:-0}"

TMP_DIR="$(mktemp -d)"
LOG_RUN_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --log-dir)
      if [[ -z "${2:-}" ]]; then
        echo "Missing value for --log-dir" >&2
        exit 1
      fi
      LOG_DIR="$2"
      shift 2
      ;;
    --skip-uv-sync)
      SKIP_UV_SYNC=1
      shift
      ;;
    --skip-npm-install)
      SKIP_NPM_INSTALL=1
      shift
      ;;
    --help|-h)
      cat <<'HELP'
Usage: ./scripts/dev.sh [--log-dir DIR] [--skip-uv-sync] [--skip-npm-install]
Environment overrides:
  HOST, PORT, PHOENIX_PORT, WEB_HOST, WEB_PORT, CURL_TIMEOUT, LOG_DIR
HELP
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

setup_logs() {
  if [[ -n "$LOG_DIR" ]]; then
    if [[ "$LOG_DIR" != /* ]]; then
      LOG_DIR="$START_DIR/$LOG_DIR"
    fi
    mkdir -p "$LOG_DIR"
    LOG_RUN_DIR="$LOG_DIR/run-$RUN_ID"
    mkdir -p "$LOG_RUN_DIR"
    SIM_LOG="$LOG_RUN_DIR/simulator.log"
    PHOENIX_LOG="$LOG_RUN_DIR/phoenix.log"
    WEB_LOG="$LOG_RUN_DIR/webapp.log"
    UV_SYNC_LOG="$LOG_RUN_DIR/uv_sync.log"
    NPM_LOG="$LOG_RUN_DIR/npm.log"
  else
    SIM_LOG="$TMP_DIR/simulator.log"
    PHOENIX_LOG="$TMP_DIR/phoenix.log"
    WEB_LOG="$TMP_DIR/webapp.log"
    UV_SYNC_LOG="$TMP_DIR/uv_sync.log"
    NPM_LOG="$TMP_DIR/npm.log"
  fi
}

setup_logs

log() {
  local level="$1"
  shift
  printf '%s [%s] %s\n' "$(date '+%H:%M:%S')" "$level" "$*"
}

require_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    log "ERROR" "Missing required command: $name"
    exit 1
  fi
}

start_process() {
  local log_file="$1"
  shift
  if command -v setsid >/dev/null 2>&1; then
    setsid "$@" >"$log_file" 2>&1 &
  else
    "$@" >"$log_file" 2>&1 &
  fi
  echo $!
}

stop_process() {
  local pid="$1"
  if command -v setsid >/dev/null 2>&1; then
    kill -- "-$pid" >/dev/null 2>&1 || true
  else
    pkill -P "$pid" >/dev/null 2>&1 || true
    kill "$pid" >/dev/null 2>&1 || true
  fi
}

wait_for_url() {
  local name="$1"
  local url="$2"
  local max_wait="$3"
  log "INFO" "Waiting for $name at $url"
  for ((i=0; i<max_wait; i++)); do
    if curl -fsS --max-time "$CURL_TIMEOUT" "$url" >/dev/null 2>&1; then
      log "INFO" "$name ready after ${i}s"
      return 0
    fi
    if (( i > 0 && i % 10 == 0 )); then
      log "INFO" "Still waiting for $name... (${i}s)"
    fi
    sleep 1
  done
  return 1
}

monitor_pids() {
  local pids=("$PHOENIX_PID" "$SIM_PID" "$WEB_PID")
  local names=("Phoenix" "Simulator" "Webapp")
  while true; do
    for idx in "${!pids[@]}"; do
      local pid="${pids[$idx]}"
      if [[ -z "$pid" ]]; then
        continue
      fi
      if ! kill -0 "$pid" >/dev/null 2>&1; then
        log "ERROR" "${names[$idx]} exited (pid=$pid)."
        case "$idx" in
          0) tail -n 200 "$PHOENIX_LOG" || true ;;
          1) tail -n 200 "$SIM_LOG" || true ;;
          2) tail -n 200 "$WEB_LOG" || true ;;
        esac
        return 1
      fi
    done
    sleep 2
  done
}

cleanup() {
  if [[ -n "${WEB_PID:-}" ]]; then
    stop_process "$WEB_PID"
    wait "$WEB_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${SIM_PID:-}" ]]; then
    stop_process "$SIM_PID"
    wait "$SIM_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${PHOENIX_PID:-}" ]]; then
    stop_process "$PHOENIX_PID"
    wait "$PHOENIX_PID" >/dev/null 2>&1 || true
  fi
  if [[ -z "$LOG_RUN_DIR" ]]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT

require_cmd uv
require_cmd cargo
require_cmd npm
require_cmd node
require_cmd curl

log "INFO" "Starting local services"
log "INFO" "Simulator: http://$HOST:$PORT"
log "INFO" "Phoenix: http://$HOST:$PHOENIX_PORT/docs"
log "INFO" "Webapp: http://$WEB_HOST:$WEB_PORT"
if [[ -n "$LOG_RUN_DIR" ]]; then
  log "INFO" "Log output dir=$LOG_RUN_DIR"
fi

export PHOENIX_ENDPOINT="http://$HOST:$PHOENIX_PORT"
export RUST_LOG="${RUST_LOG:-info}"
export PHOENIX_LOG_LEVEL="${PHOENIX_LOG_LEVEL:-info}"

pushd "$ROOT_DIR/phoenix" >/dev/null
if [[ "$SKIP_UV_SYNC" != "1" ]]; then
  log "INFO" "Phoenix: installing dependencies (uv sync)"
  uv sync --project . >"$UV_SYNC_LOG" 2>&1 || {
    log "ERROR" "Phoenix uv sync failed (see $UV_SYNC_LOG)"
    exit 1
  }
else
  log "INFO" "Phoenix: skipping uv sync"
fi
log "INFO" "Phoenix log: $PHOENIX_LOG"
PHOENIX_PID=$(start_process "$PHOENIX_LOG" uv run uvicorn service.server:app --reload --host "$HOST" --port "$PHOENIX_PORT")
popd >/dev/null

pushd "$ROOT_DIR/simulator" >/dev/null
log "INFO" "Simulator log: $SIM_LOG"
SIM_PID=$(start_process "$SIM_LOG" cargo run -- serve --host "$HOST" --port "$PORT")
popd >/dev/null

pushd "$ROOT_DIR/webapp" >/dev/null
if [[ "$SKIP_NPM_INSTALL" != "1" ]]; then
  if [[ ! -d node_modules ]]; then
    log "INFO" "Webapp: installing dependencies (npm install)"
    npm install >"$NPM_LOG" 2>&1 || {
      log "ERROR" "Webapp npm install failed (see $NPM_LOG)"
      exit 1
    }
  fi
else
  log "INFO" "Webapp: skipping npm install"
fi
log "INFO" "Webapp log: $WEB_LOG"
WEB_PID=$(start_process "$WEB_LOG" npm run dev -- --host "$WEB_HOST" --port "$WEB_PORT")
popd >/dev/null

if ! wait_for_url "Phoenix" "http://$HOST:$PHOENIX_PORT/docs" 120; then
  log "ERROR" "Phoenix failed to start. Tail:"
  tail -n 200 "$PHOENIX_LOG" || true
  exit 1
fi

if ! wait_for_url "Simulator" "http://$HOST:$PORT/api/health" 120; then
  log "ERROR" "Simulator failed to start. Tail:"
  tail -n 200 "$SIM_LOG" || true
  exit 1
fi

if ! wait_for_url "Webapp" "http://$WEB_HOST:$WEB_PORT" 120; then
  log "ERROR" "Webapp failed to start. Tail:"
  tail -n 200 "$WEB_LOG" || true
  exit 1
fi

log "INFO" "All services are running. Press Ctrl-C to stop."
monitor_pids
