#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-8799}"
WEB_ROOT="${WEB_ROOT:-$ROOT_DIR/webapp/dist}"

TMP_DIR="$(mktemp -d)"
LOG_FILE="$TMP_DIR/server.log"

export SCORING_MODE=heuristic
export SIM_SNAPSHOT_PATH="$TMP_DIR/snapshots.json"
export USER_PROFILES_PATH="$TMP_DIR/user_profiles.json"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cargo run --manifest-path "$ROOT_DIR/simulator/Cargo.toml" -- serve \
  --host "$HOST" \
  --port "$PORT" \
  --web-root "$WEB_ROOT" \
  >"$LOG_FILE" 2>&1 &
SERVER_PID=$!

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

compare_payload=$(cat <<'JSON'
{
  "scoring_mode": "heuristic",
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
      "scoring_mode": "heuristic"
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
      "scoring_mode": "heuristic"
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
