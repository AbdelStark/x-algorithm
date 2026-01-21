#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-8787}"
PHOENIX_PORT="${PHOENIX_PORT:-8000}"
WEB_ROOT="${WEB_ROOT:-$ROOT_DIR/webapp/dist}"

TMP_DIR="$(mktemp -d)"
LOG_FILE="$TMP_DIR/server.log"
PHOENIX_LOG="$TMP_DIR/phoenix.log"
WEB_ROOT_CREATED=false
INDEX_CREATED=false

export SCORING_MODE=heuristic
export SIM_SNAPSHOT_PATH="$TMP_DIR/snapshots.json"
export USER_PROFILES_PATH="$TMP_DIR/user_profiles.json"
export SCORING_CONFIG_PATH="$TMP_DIR/scoring.toml"
export PHOENIX_ENDPOINT="http://$HOST:$PHOENIX_PORT"

cleanup() {
  if [[ -n "${PHOENIX_PID:-}" ]]; then
    stop_process "$PHOENIX_PID"
    wait "$PHOENIX_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${SERVER_PID:-}" ]]; then
    stop_process "$SERVER_PID"
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
  if [[ "$INDEX_CREATED" == "true" ]]; then
    rm -f "$WEB_ROOT/index.html"
  fi
  if [[ "$WEB_ROOT_CREATED" == "true" ]]; then
    rmdir "$WEB_ROOT" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

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

if ! command -v uv >/dev/null 2>&1; then
  echo "uv is required for Phoenix service. Install it with: pip install uv" >&2
  exit 1
fi

if [[ ! -d "$WEB_ROOT" ]]; then
  mkdir -p "$WEB_ROOT"
  WEB_ROOT_CREATED=true
fi

if [[ ! -f "$WEB_ROOT/index.html" ]]; then
  cat <<'HTML' >"$WEB_ROOT/index.html"
<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Virality Simulator</title></head>
<body><main>Virality Simulator</main></body></html>
HTML
  INDEX_CREATED=true
fi

cp "$ROOT_DIR/config/scoring.toml" "$SCORING_CONFIG_PATH"

pushd "$ROOT_DIR/phoenix" >/dev/null
uv sync --project . >/dev/null
PHOENIX_PID=$(start_process "$PHOENIX_LOG" uv run uvicorn service.server:app --reload --port "$PHOENIX_PORT")
popd >/dev/null

phoenix_ready=false
for _ in {1..180}; do
  if curl -fsS "http://$HOST:$PHOENIX_PORT/docs" >/dev/null 2>&1; then
    phoenix_ready=true
    break
  fi
  sleep 1
done

if [[ "$phoenix_ready" != "true" ]]; then
  echo "Phoenix service failed to start." >&2
  tail -n 200 "$PHOENIX_LOG" >&2 || true
  exit 1
fi

pushd "$ROOT_DIR/simulator" >/dev/null
SERVER_PID=$(start_process "$LOG_FILE" cargo run -- serve --host "$HOST" --port "$PORT")
popd >/dev/null

health_url="http://$HOST:$PORT/api/health"
ready=false
for _ in {1..180}; do
  if curl -fsS "$health_url" >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 1
done

if [[ "$ready" != "true" ]]; then
  echo "Server failed to start." >&2
  tail -n 200 "$LOG_FILE" >&2 || true
  exit 1
fi

config_response=$(curl -fsS "http://$HOST:$PORT/api/config")
CONFIG_RESPONSE="$config_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["CONFIG_RESPONSE"])
assert "scoring" in data
assert "weights" in data
assert "phoenix" in data
PY

weights_update=$(CONFIG_RESPONSE="$config_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["CONFIG_RESPONSE"])
weights = data["weights"]
weights["favorite"] = weights.get("favorite", 1.0) + 0.5
print(json.dumps({"weights": weights}))
PY
)

update_response=$(curl -fsS \
  -X PUT \
  -H "Content-Type: application/json" \
  -d "$weights_update" \
  "http://$HOST:$PORT/api/config/weights")

UPDATE_RESPONSE="$update_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["UPDATE_RESPONSE"])
assert data["weights"]["favorite"] >= 1.0
PY

simulate_payload=$(cat <<'JSON'
{
  "text": "E2E test tweet",
  "media": "none",
  "has_link": false,
  "followers": 1200,
  "following": 350,
  "account_age_days": 900,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.4,
  "verified": false,
  "hour_of_day": 10,
  "novelty": 0.5,
  "timeliness": 0.4,
  "topic_saturation": 0.3,
  "audience_fit": 0.6,
  "controversy": 0.2,
  "sentiment": 0.1,
  "use_ai": false,
  "scoring_mode": "heuristic"
}
JSON
)

simulate_response=$(curl -fsS \
  -H "Content-Type: application/json" \
  -d "$simulate_payload" \
  "http://$HOST:$PORT/api/simulate")

SIM_RESPONSE="$simulate_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["SIM_RESPONSE"])
for key in ["score", "weighted_score", "final_score", "actions", "signals", "scoring_mode"]:
    assert key in data, f"missing {key}"
assert data["scoring_mode"] == "heuristic"
PY

phoenix_payload=$(cat <<'JSON'
{
  "text": "Phoenix model test tweet",
  "media": "none",
  "has_link": false,
  "followers": 2200,
  "following": 400,
  "account_age_days": 1200,
  "avg_engagement_rate": 0.03,
  "posts_per_day": 1.2,
  "verified": false,
  "hour_of_day": 11,
  "novelty": 0.6,
  "timeliness": 0.6,
  "topic_saturation": 0.2,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "phoenix",
  "user_id": "e2e_user"
}
JSON
)

phoenix_response=$(curl -fsS \
  -H "Content-Type: application/json" \
  -d "$phoenix_payload" \
  "http://$HOST:$PORT/api/simulate")

PHOENIX_RESPONSE="$phoenix_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["PHOENIX_RESPONSE"])
assert data["scoring_mode"] == "phoenix"
phoenix = data.get("phoenix_actions") or {}
assert phoenix, "missing phoenix_actions"
assert 0 <= phoenix.get("like", -1) <= 1
PY

hybrid_payload=$(cat <<'JSON'
{
  "text": "Hybrid model test tweet",
  "media": "none",
  "has_link": false,
  "followers": 2200,
  "following": 400,
  "account_age_days": 1200,
  "avg_engagement_rate": 0.03,
  "posts_per_day": 1.2,
  "verified": false,
  "hour_of_day": 11,
  "novelty": 0.6,
  "timeliness": 0.6,
  "topic_saturation": 0.2,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "hybrid",
  "phoenix_weight": 0.5,
  "user_id": "e2e_user"
}
JSON
)

hybrid_response=$(curl -fsS \
  -H "Content-Type: application/json" \
  -d "$hybrid_payload" \
  "http://$HOST:$PORT/api/simulate")

HYBRID_RESPONSE="$hybrid_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["HYBRID_RESPONSE"])
assert data["scoring_mode"] == "hybrid"
phoenix = data.get("phoenix_actions") or {}
assert phoenix, "missing phoenix_actions"
PY

compare_payload=$(cat <<'JSON'
{
  "scoring_mode": "phoenix",
  "candidates": [
    {
      "text": "First draft",
      "media": "none",
      "followers": 1200,
      "following": 350,
      "account_age_days": 900,
      "avg_engagement_rate": 0.02,
      "posts_per_day": 1.4,
      "verified": false,
      "hour_of_day": 10,
      "novelty": 0.4,
      "timeliness": 0.4,
      "topic_saturation": 0.3,
      "audience_fit": 0.6,
      "controversy": 0.2,
      "sentiment": 0.1,
      "use_ai": false,
      "scoring_mode": "phoenix",
      "user_id": "e2e_user"
    },
    {
      "text": "Second draft",
      "media": "none",
      "followers": 1200,
      "following": 350,
      "account_age_days": 900,
      "avg_engagement_rate": 0.02,
      "posts_per_day": 1.4,
      "verified": false,
      "hour_of_day": 10,
      "novelty": 0.7,
      "timeliness": 0.6,
      "topic_saturation": 0.2,
      "audience_fit": 0.7,
      "controversy": 0.2,
      "sentiment": 0.1,
      "use_ai": false,
      "scoring_mode": "phoenix",
      "user_id": "e2e_user"
    }
  ]
}
JSON
)

compare_response=$(curl -fsS \
  -H "Content-Type: application/json" \
  -d "$compare_payload" \
  "http://$HOST:$PORT/api/simulate/compare")

COMPARE_RESPONSE="$compare_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["COMPARE_RESPONSE"])
results = data.get("results", [])
assert len(results) == 2
assert results[0]["rank"] == 1
assert results[0]["response"]["scoring_mode"] == "phoenix"
PY

user_payload=$(cat <<'JSON'
{
  "user_id": "e2e_user",
  "followers": 1200,
  "following": 350,
  "account_age_days": 900,
  "verified": false,
  "generate_synthetic_history": true
}
JSON
)

user_response=$(curl -fsS \
  -H "Content-Type: application/json" \
  -d "$user_payload" \
  "http://$HOST:$PORT/api/users")

USER_RESPONSE="$user_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["USER_RESPONSE"])
assert data.get("user_id") == "e2e_user"
PY

history_response=$(curl -fsS "http://$HOST:$PORT/api/users/e2e_user/history")
HISTORY_RESPONSE="$history_response" python3 - <<'PY'
import json
import os

data = json.loads(os.environ["HISTORY_RESPONSE"])
assert isinstance(data, list)
assert len(data) > 0
PY

echo "E2E checks passed."
