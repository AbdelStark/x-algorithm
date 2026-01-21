#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
START_DIR="$(pwd -P)"
HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-8787}"
PHOENIX_PORT="${PHOENIX_PORT:-8000}"
WEB_ROOT="${WEB_ROOT:-$ROOT_DIR/webapp/dist}"
CURL_TIMEOUT="${CURL_TIMEOUT:-10}"
E2E_KEEP_TMP="${E2E_KEEP_TMP:-0}"
E2E_VERBOSE="${E2E_VERBOSE:-0}"
LOG_DIR="${E2E_LOG_DIR:-}"
RUN_ID="$(date '+%Y%m%d_%H%M%S')_${RANDOM}"

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
    --keep-tmp)
      E2E_KEEP_TMP=1
      shift
      ;;
    --verbose)
      E2E_VERBOSE=1
      shift
      ;;
    --help|-h)
      cat <<'HELP'
Usage: ./scripts/e2e.sh [--log-dir DIR] [--keep-tmp] [--verbose]
  --log-dir DIR  Save Phoenix and simulator logs under this directory.
  --keep-tmp     Preserve temp working directory after the run.
  --verbose      Show uv sync output and extra details.
HELP
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

TMP_DIR="$(mktemp -d)"
LOG_RUN_DIR=""
if [[ -n "$LOG_DIR" ]]; then
  if [[ "$LOG_DIR" != /* ]]; then
    LOG_DIR="$START_DIR/$LOG_DIR"
  fi
  mkdir -p "$LOG_DIR"
  LOG_RUN_DIR="$LOG_DIR/run-$RUN_ID"
  mkdir -p "$LOG_RUN_DIR"
  LOG_FILE="$LOG_RUN_DIR/simulator.log"
  PHOENIX_LOG="$LOG_RUN_DIR/phoenix.log"
  UV_SYNC_LOG="$LOG_RUN_DIR/uv_sync.log"
else
  LOG_FILE="$TMP_DIR/server.log"
  PHOENIX_LOG="$TMP_DIR/phoenix.log"
  UV_SYNC_LOG="$TMP_DIR/uv_sync.log"
fi
WEB_ROOT_CREATED=false
INDEX_CREATED=false

export SCORING_MODE=heuristic
export SIM_SNAPSHOT_PATH="$TMP_DIR/snapshots.json"
export USER_PROFILES_PATH="$TMP_DIR/user_profiles.json"
export SCORING_CONFIG_PATH="$TMP_DIR/scoring.toml"
export PHOENIX_ENDPOINT="http://$HOST:$PHOENIX_PORT"

CHECK_NAMES=()
CHECK_STATUS=()
CHECK_DURATION_MS=()
CHECK_DETAILS=()
FAILED_CHECKS=0
LAST_DETAIL=""
CONFIG_RESPONSE=""
UPDATE_RESPONSE=""
E2E_USER_ID="e2e_user"
AUTO_USER_ID="e2e_auto_user"

now_ms() {
  python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
}

log() {
  local level="$1"
  shift
  printf '%s [%s] %s\n' "$(date '+%H:%M:%S')" "$level" "$*"
}

sanitize_detail() {
  printf '%s' "$1" | tr '\n' ' ' | tr -s '[:space:]' ' ' | sed -e 's/^ //; s/ $//'
}

set_detail() {
  LAST_DETAIL="$(sanitize_detail "$1")"
}

run_check() {
  local name="$1"
  shift
  local start end duration rc detail
  start=$(now_ms)
  LAST_DETAIL=""
  set +e
  "$@"
  rc=$?
  set -e
  end=$(now_ms)
  duration=$((end - start))
  CHECK_NAMES+=("$name")
  CHECK_STATUS+=("$rc")
  CHECK_DURATION_MS+=("$duration")
  detail="$LAST_DETAIL"
  CHECK_DETAILS+=("$detail")
  if [[ "$rc" -eq 0 ]]; then
    log "PASS" "$name (${duration}ms)"
    if [[ -n "$detail" ]]; then
      log "INFO" "$detail"
    fi
  else
    log "FAIL" "$name (${duration}ms)"
    if [[ -n "$detail" ]]; then
      log "INFO" "$detail"
    fi
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
  fi
}

require_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    log "ERROR" "Missing required command: $name"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
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

fatal() {
  log "ERROR" "$1"
  FAILED_CHECKS=$((FAILED_CHECKS + 1))
  exit 1
}

check_config() {
  local response detail rc
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" "$CONFIG_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "config request failed"
    return "$rc"
  fi
  CONFIG_RESPONSE="$response"
  detail=$(CONFIG_RESPONSE="$CONFIG_RESPONSE" python3 - <<'PY' 2>&1
import json
import os

data = json.loads(os.environ["CONFIG_RESPONSE"])
required = ["scoring", "weights", "weighted", "diversity", "oon", "phoenix"]
missing = [key for key in required if key not in data]
assert not missing, f"missing config keys: {missing}"

weights = data["weights"]
required_weights = [
    "favorite", "reply", "repost", "photo_expand", "click", "profile_click",
    "vqv", "share", "share_dm", "share_link", "dwell", "quote", "quoted_click",
    "follow_author", "not_interested", "block", "mute", "report", "dwell_time",
]
missing_weights = [key for key in required_weights if key not in weights]
assert not missing_weights, f"missing weights: {missing_weights}"

print(
    f"weights={len(weights)} mode={data['scoring'].get('mode')}"
    f" oon_multiplier={data['oon'].get('multiplier')}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_weights_update() {
  local update_payload response detail rc
  if [[ -z "$CONFIG_RESPONSE" ]]; then
    set_detail "config response missing"
    return 1
  fi
  update_payload=$(CONFIG_RESPONSE="$CONFIG_RESPONSE" python3 - <<'PY' 2>&1
import json
import os

data = json.loads(os.environ["CONFIG_RESPONSE"])
weights = data["weights"]
weights["favorite"] = weights.get("favorite", 1.0) + 0.5
print(json.dumps({"weights": weights}))
PY
)
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "$update_payload"
    return "$rc"
  fi
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -X PUT \
    -H "Content-Type: application/json" \
    -d "$update_payload" \
    "$CONFIG_WEIGHTS_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "weights update failed"
    return "$rc"
  fi
  UPDATE_RESPONSE="$response"
  detail=$(CONFIG_RESPONSE="$CONFIG_RESPONSE" UPDATE_RESPONSE="$UPDATE_RESPONSE" python3 - <<'PY' 2>&1
import json
import os

before = json.loads(os.environ["CONFIG_RESPONSE"])["weights"]["favorite"]
after = json.loads(os.environ["UPDATE_RESPONSE"])["weights"]["favorite"]
assert after > before, f"favorite not updated: {before} -> {after}"
print(f"favorite_before={before} favorite_after={after}")
PY
)
  rc=$?
  set_detail "$detail"
  CONFIG_RESPONSE="$UPDATE_RESPONSE"
  return "$rc"
}

check_user_profile() {
  local user_payload user_response history_response detail rc
  user_payload=$(cat <<JSON
{
  "user_id": "$E2E_USER_ID",
  "followers": 50000,
  "following": 420,
  "account_age_days": 2000,
  "verified": true,
  "engagement_history": [
    {
      "post_id": "hist_post_1",
      "author_id": "author_1",
      "timestamp": 1710000000,
      "actions": {
        "liked": true,
        "replied": false,
        "reposted": false,
        "quoted": false,
        "clicked": true,
        "shared": false,
        "followed_author": false,
        "blocked": false,
        "muted": false,
        "reported": false
      }
    },
    {
      "post_id": "hist_post_2",
      "author_id": "author_2",
      "timestamp": 1710003600,
      "actions": {
        "liked": true,
        "replied": true,
        "reposted": false,
        "quoted": false,
        "clicked": false,
        "shared": true,
        "followed_author": true,
        "blocked": false,
        "muted": false,
        "reported": false
      }
    }
  ]
}
JSON
)
  user_response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$user_payload" \
    "$USERS_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "user upsert failed"
    return "$rc"
  fi
  history_response=$(curl -fsS --max-time "$CURL_TIMEOUT" "$USER_HISTORY_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "user history fetch failed"
    return "$rc"
  fi
  detail=$(EXPECTED_USER_ID="$E2E_USER_ID" USER_RESPONSE="$user_response" HISTORY_RESPONSE="$history_response" python3 - <<'PY' 2>&1
import json
import os

expected = os.environ["EXPECTED_USER_ID"]
user = json.loads(os.environ["USER_RESPONSE"])
history = json.loads(os.environ["HISTORY_RESPONSE"])
assert user.get("user_id") == expected
assert len(history) == 2, f"history length {len(history)}"
assert history[0]["post_id"].startswith("hist_")
print(f"user_id={expected} history_len={len(history)}")
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_baseline_heuristic() {
  local payload response detail rc
  payload=$(cat <<'JSON'
{
  "text": "E2E baseline: Testing production-like scoring signals.",
  "media": "none",
  "has_link": false,
  "followers": 1200,
  "following": 350,
  "account_age_days": 900,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.4,
  "verified": false,
  "hour_of_day": 10,
  "novelty": 0.8,
  "timeliness": 0.8,
  "topic_saturation": 0.2,
  "audience_fit": 0.85,
  "controversy": 0.05,
  "sentiment": 0.6,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "baseline_heuristic"
}
JSON
)
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "baseline simulate failed"
    return "$rc"
  fi
  detail=$(SIM_RESPONSE="$response" python3 - <<'PY' 2>&1
import json
import os

data = json.loads(os.environ["SIM_RESPONSE"])
assert data["scoring_mode"] == "heuristic"
assert 0 <= data["score"] <= 100
expected = data["weighted_score"] * data["diversity_multiplier"] * data["oon_multiplier"]
assert abs(data["final_score"] - expected) < 1e-6
actions = data["actions"]
for key in [
    "like", "reply", "repost", "quote", "click", "profile_click", "video_view",
    "photo_expand", "share", "share_dm", "share_link", "dwell", "follow_author",
    "quoted_click", "not_interested", "block", "mute", "report", "dwell_time",
]:
    assert key in actions
print(
    f"score={data['score']:.2f} weighted={data['weighted_score']:.3f}"
    f" final={data['final_score']:.3f} tier={data['tier']}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_link_signal() {
  local payload_no payload_yes response_no response_yes detail rc
  payload_no=$(cat <<'JSON'
{
  "text": "Link test baseline",
  "media": "none",
  "has_link": false,
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
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "link_no"
}
JSON
)
  payload_yes=$(cat <<'JSON'
{
  "text": "Link test baseline",
  "media": "none",
  "has_link": true,
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
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "link_yes"
}
JSON
)
  response_no=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_no" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "link=no simulate failed"
    return "$rc"
  fi
  response_yes=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_yes" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "link=yes simulate failed"
    return "$rc"
  fi
  detail=$(NO_LINK_RESPONSE="$response_no" LINK_RESPONSE="$response_yes" python3 - <<'PY' 2>&1
import json
import os

no_link = json.loads(os.environ["NO_LINK_RESPONSE"])
yes_link = json.loads(os.environ["LINK_RESPONSE"])

click_no = no_link["actions"]["click"]
click_yes = yes_link["actions"]["click"]
share_link_no = no_link["actions"]["share_link"]
share_link_yes = yes_link["actions"]["share_link"]

assert click_yes > click_no, f"click not boosted: {click_no} -> {click_yes}"
assert share_link_yes > share_link_no, f"share_link not boosted: {share_link_no} -> {share_link_yes}"

print(
    f"click_no={click_no:.3f} click_yes={click_yes:.3f}"
    f" share_link_no={share_link_no:.3f} share_link_yes={share_link_yes:.3f}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_oon_penalty() {
  local payload_in payload_oon response_in response_oon detail rc
  payload_in=$(cat <<'JSON'
{
  "text": "OON penalty test",
  "media": "none",
  "has_link": false,
  "is_oon": false,
  "followers": 1200,
  "following": 350,
  "account_age_days": 900,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.4,
  "verified": false,
  "hour_of_day": 10,
  "novelty": 0.8,
  "timeliness": 0.8,
  "topic_saturation": 0.2,
  "audience_fit": 0.85,
  "controversy": 0.05,
  "sentiment": 0.6,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "oon_in"
}
JSON
)
  payload_oon=$(cat <<'JSON'
{
  "text": "OON penalty test",
  "media": "none",
  "has_link": false,
  "is_oon": true,
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
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "oon_out"
}
JSON
)
  response_in=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_in" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "oon in-network simulate failed"
    return "$rc"
  fi
  response_oon=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_oon" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "oon out-of-network simulate failed"
    return "$rc"
  fi
  detail=$(CONFIG_RESPONSE="$CONFIG_RESPONSE" IN_RESPONSE="$response_in" OON_RESPONSE="$response_oon" python3 - <<'PY' 2>&1
import json
import os

config = json.loads(os.environ["CONFIG_RESPONSE"])
expected_multiplier = config["oon"]["multiplier"]
in_net = json.loads(os.environ["IN_RESPONSE"])
oon = json.loads(os.environ["OON_RESPONSE"])

assert abs(oon["oon_multiplier"] - expected_multiplier) < 1e-6
assert oon["final_score"] < in_net["final_score"]

print(
    f"final_in={in_net['final_score']:.3f} final_oon={oon['final_score']:.3f}"
    f" oon_multiplier={oon['oon_multiplier']:.2f}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_video_vqv_threshold() {
  local payload_short payload_long response_short response_long detail rc
  payload_short=$(cat <<'JSON'
{
  "text": "Video threshold test",
  "media": "video",
  "video_duration_seconds": 2,
  "has_link": false,
  "followers": 2400,
  "following": 400,
  "account_age_days": 1200,
  "avg_engagement_rate": 0.03,
  "posts_per_day": 1.1,
  "verified": false,
  "hour_of_day": 12,
  "novelty": 0.5,
  "timeliness": 0.4,
  "topic_saturation": 0.2,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "video_short"
}
JSON
)
  payload_long=$(cat <<'JSON'
{
  "text": "Video threshold test",
  "media": "video",
  "video_duration_seconds": 12,
  "has_link": false,
  "followers": 2400,
  "following": 400,
  "account_age_days": 1200,
  "avg_engagement_rate": 0.03,
  "posts_per_day": 1.1,
  "verified": false,
  "hour_of_day": 12,
  "novelty": 0.5,
  "timeliness": 0.4,
  "topic_saturation": 0.2,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "video_long"
}
JSON
)
  response_short=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_short" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "video short simulate failed"
    return "$rc"
  fi
  response_long=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_long" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "video long simulate failed"
    return "$rc"
  fi
  detail=$(SHORT_RESPONSE="$response_short" LONG_RESPONSE="$response_long" python3 - <<'PY' 2>&1
import json
import os

short = json.loads(os.environ["SHORT_RESPONSE"])
long = json.loads(os.environ["LONG_RESPONSE"])

assert long["weighted_score"] > short["weighted_score"]

print(
    f"weighted_short={short['weighted_score']:.3f}"
    f" weighted_long={long['weighted_score']:.3f}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_weighted_score_formula() {
  local payload response detail rc
  if [[ -z "$CONFIG_RESPONSE" ]]; then
    set_detail "config response missing"
    return 1
  fi
  payload=$(cat <<'JSON'
{
  "text": "Weighted score formula check",
  "media": "video",
  "video_duration_seconds": 12,
  "has_link": false,
  "followers": 3600,
  "following": 410,
  "account_age_days": 1400,
  "avg_engagement_rate": 0.03,
  "posts_per_day": 1.1,
  "verified": false,
  "hour_of_day": 15,
  "novelty": 0.6,
  "timeliness": 0.6,
  "topic_saturation": 0.2,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.3,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "weighted_formula"
}
JSON
)
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "weighted score simulate failed"
    return "$rc"
  fi
  detail=$(CONFIG_RESPONSE="$CONFIG_RESPONSE" SIM_RESPONSE="$response" python3 - <<'PY' 2>&1
import json
import os

config = json.loads(os.environ["CONFIG_RESPONSE"])
data = json.loads(os.environ["SIM_RESPONSE"])
weights = config["weights"]
weighted_cfg = config["weighted"]
actions = data["actions"]

total = 0.0
total += actions["like"] * weights["favorite"]
total += actions["reply"] * weights["reply"]
total += actions["repost"] * weights["repost"]
total += actions["photo_expand"] * weights["photo_expand"]
total += actions["click"] * weights["click"]
total += actions["profile_click"] * weights["profile_click"]
total += actions["share"] * weights["share"]
total += actions["share_dm"] * weights["share_dm"]
total += actions["share_link"] * weights["share_link"]
total += actions["dwell"] * weights["dwell"]
total += actions["quote"] * weights["quote"]
total += actions["quoted_click"] * weights["quoted_click"]
total += actions["follow_author"] * weights["follow_author"]
total += actions["not_interested"] * weights["not_interested"]
total += actions["block"] * weights["block"]
total += actions["mute"] * weights["mute"]
total += actions["report"] * weights["report"]
total += actions["dwell_time"] * weights["dwell_time"]

video_duration = 12.0
if video_duration >= weighted_cfg["vqv_duration_threshold"]:
    total += actions["video_view"] * weights["vqv"]

if total < 0.0:
    total += weighted_cfg["score_offset"]

expected = total
observed = data["weighted_score"]
assert abs(expected - observed) < 1e-6, f"expected {expected}, got {observed}"
print(f"weighted_expected={expected:.4f} weighted_observed={observed:.4f}")
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_impressions_scale() {
  local payload_small payload_large response_small response_large detail rc
  payload_small=$(cat <<'JSON'
{
  "text": "Impressions scale test",
  "media": "none",
  "has_link": false,
  "followers": 1000,
  "following": 300,
  "account_age_days": 900,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.2,
  "verified": false,
  "hour_of_day": 9,
  "novelty": 0.5,
  "timeliness": 0.4,
  "topic_saturation": 0.3,
  "audience_fit": 0.6,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "impressions_small"
}
JSON
)
  payload_large=$(cat <<'JSON'
{
  "text": "Impressions scale test",
  "media": "none",
  "has_link": false,
  "followers": 100000,
  "following": 300,
  "account_age_days": 900,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.2,
  "verified": false,
  "hour_of_day": 9,
  "novelty": 0.5,
  "timeliness": 0.4,
  "topic_saturation": 0.3,
  "audience_fit": 0.6,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "impressions_large"
}
JSON
)
  response_small=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_small" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "impressions small simulate failed"
    return "$rc"
  fi
  response_large=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_large" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "impressions large simulate failed"
    return "$rc"
  fi
  detail=$(SMALL_RESPONSE="$response_small" LARGE_RESPONSE="$response_large" python3 - <<'PY' 2>&1
import json
import os

small = json.loads(os.environ["SMALL_RESPONSE"])
large = json.loads(os.environ["LARGE_RESPONSE"])

assert large["impressions_in"] > small["impressions_in"]
assert large["impressions_total"] > small["impressions_total"]
print(
    f"in_small={small['impressions_in']:.1f} in_large={large['impressions_in']:.1f}"
    f" total_small={small['impressions_total']:.1f} total_large={large['impressions_total']:.1f}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_negative_feedback() {
  local payload_base payload_neg response_base response_neg detail rc
  payload_base=$(cat <<'JSON'
{
  "text": "Breaking: new workflow guide. What do you think?",
  "media": "none",
  "has_link": false,
  "followers": 1800,
  "following": 320,
  "account_age_days": 1000,
  "avg_engagement_rate": 0.025,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 13,
  "novelty": 0.6,
  "timeliness": 0.6,
  "topic_saturation": 0.3,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.3,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "negative_base"
}
JSON
)
  payload_neg=$(cat <<'JSON'
{
  "text": "CLICK NOW!!! THIS IS A SCAM!!! #SPAM #SPAM #SPAM http://bad.link @user1 @user2",
  "media": "none",
  "has_link": true,
  "followers": 1800,
  "following": 320,
  "account_age_days": 1000,
  "avg_engagement_rate": 0.025,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 13,
  "novelty": 0.1,
  "timeliness": 0.1,
  "topic_saturation": 0.95,
  "audience_fit": 0.2,
  "controversy": 0.9,
  "sentiment": -0.8,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "negative_case"
}
JSON
)
  response_base=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_base" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "negative base simulate failed"
    return "$rc"
  fi
  response_neg=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_neg" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "negative case simulate failed"
    return "$rc"
  fi
  detail=$(BASE_RESPONSE="$response_base" NEG_RESPONSE="$response_neg" python3 - <<'PY' 2>&1
import json
import os

base = json.loads(os.environ["BASE_RESPONSE"])
neg = json.loads(os.environ["NEG_RESPONSE"])

assert neg["signals"]["negative_risk"] > base["signals"]["negative_risk"]
assert neg["actions"]["not_interested"] > base["actions"]["not_interested"]
assert neg["actions"]["block"] > base["actions"]["block"]
assert neg["actions"]["report"] > base["actions"]["report"]
assert neg["score"] < base["score"]

print(
    f"neg_risk_base={base['signals']['negative_risk']:.3f}"
    f" neg_risk_neg={neg['signals']['negative_risk']:.3f}"
    f" score_base={base['score']:.2f} score_neg={neg['score']:.2f}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_dwell_time_growth() {
  local payload_short payload_long response_short response_long detail rc
  payload_short=$(cat <<'JSON'
{
  "text": "Short text.",
  "media": "none",
  "has_link": false,
  "followers": 1500,
  "following": 280,
  "account_age_days": 800,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 8,
  "novelty": 0.4,
  "timeliness": 0.4,
  "topic_saturation": 0.3,
  "audience_fit": 0.6,
  "controversy": 0.1,
  "sentiment": 0.1,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "dwell_short"
}
JSON
)
  payload_long=$(cat <<'JSON'
{
  "text": "Longer text with more context. This explains the idea in detail, shares background, and adds nuance so readers spend more time reading the post from start to finish.",
  "media": "none",
  "has_link": false,
  "followers": 1500,
  "following": 280,
  "account_age_days": 800,
  "avg_engagement_rate": 0.02,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 8,
  "novelty": 0.4,
  "timeliness": 0.4,
  "topic_saturation": 0.3,
  "audience_fit": 0.6,
  "controversy": 0.1,
  "sentiment": 0.1,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "request_id": "dwell_long"
}
JSON
)
  response_short=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_short" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "dwell short simulate failed"
    return "$rc"
  fi
  response_long=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_long" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "dwell long simulate failed"
    return "$rc"
  fi
  detail=$(SHORT_RESPONSE="$response_short" LONG_RESPONSE="$response_long" python3 - <<'PY' 2>&1
import json
import os

short = json.loads(os.environ["SHORT_RESPONSE"])
long = json.loads(os.environ["LONG_RESPONSE"])

assert long["actions"]["dwell_time"] > short["actions"]["dwell_time"]
print(
    f"dwell_short={short['actions']['dwell_time']:.2f}"
    f" dwell_long={long['actions']['dwell_time']:.2f}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_auto_history_creation() {
  local payload response detail rc
  payload=$(cat <<JSON
{
  "text": "Auto history creation test",
  "media": "none",
  "has_link": false,
  "followers": 2100,
  "following": 360,
  "account_age_days": 1100,
  "avg_engagement_rate": 0.03,
  "posts_per_day": 1.1,
  "verified": false,
  "hour_of_day": 11,
  "novelty": 0.5,
  "timeliness": 0.5,
  "topic_saturation": 0.2,
  "audience_fit": 0.7,
  "controversy": 0.1,
  "sentiment": 0.2,
  "use_ai": false,
  "scoring_mode": "phoenix",
  "user_id": "$AUTO_USER_ID",
  "request_id": "auto_history"
}
JSON
)
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "auto history simulate failed"
    return "$rc"
  fi
  detail=$(USER_PROFILES_PATH="$USER_PROFILES_PATH" USER_ID="$AUTO_USER_ID" SIM_RESPONSE="$response" python3 - <<'PY' 2>&1
import json
import os

path = os.environ["USER_PROFILES_PATH"]
user_id = os.environ["USER_ID"]
data = json.loads(os.environ["SIM_RESPONSE"])
assert data["scoring_mode"] == "phoenix"
assert not any("Phoenix unavailable" in warn for warn in data.get("warnings", []))

with open(path, "r", encoding="utf-8") as handle:
    profiles = json.load(handle)
profile = profiles.get(user_id)
assert profile is not None, "auto profile missing"
assert len(profile.get("engagement_history", [])) > 0

print(f"auto_user={user_id} history_len={len(profile['engagement_history'])}")
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_phoenix_rank() {
  local payload response detail rc
  payload=$(cat <<'JSON'
{
  "user_id": "rank_user",
  "history_posts": [
    {
      "post_id": "hist_1",
      "author_id": "hist_author_1",
      "text_hash": 101,
      "author_hash": 201,
      "product_surface": 0
    },
    {
      "post_id": "hist_2",
      "author_id": "hist_author_2",
      "text_hash": 102,
      "author_hash": 202,
      "product_surface": 0
    }
  ],
  "history_actions": [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  "candidates": [
    {
      "post_id": "candidate_short",
      "author_id": "cand_author",
      "text_hash": 501,
      "author_hash": 601,
      "product_surface": 0,
      "video_duration_seconds": 2
    },
    {
      "post_id": "candidate_long",
      "author_id": "cand_author",
      "text_hash": 501,
      "author_hash": 601,
      "product_surface": 0,
      "video_duration_seconds": 12
    }
  ]
}
JSON
)
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$PHOENIX_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "phoenix /rank failed"
    return "$rc"
  fi
  detail=$(RANK_RESPONSE="$response" python3 - <<'PY' 2>&1
import json
import os

data = json.loads(os.environ["RANK_RESPONSE"])
scores = data.get("scores", [])
assert len(scores) == 2

required = [
    "like", "reply", "repost", "photo_expand", "click", "profile_click",
    "video_view", "share", "share_dm", "share_link", "dwell", "quote",
    "quoted_click", "follow_author", "not_interested", "block", "mute",
    "report", "dwell_time",
]

weights = {}
ranks = []
for entry in scores:
    actions = entry["phoenix_scores"]
    missing = [key for key in required if key not in actions]
    assert not missing, f"missing actions: {missing}"
    weights[entry["post_id"]] = entry["weighted_score"]
    ranks.append(entry["rank"])

assert weights["candidate_long"] > weights["candidate_short"]
assert sorted(ranks) == [1, 2]

print(
    f"short_score={weights['candidate_short']:.3f}"
    f" long_score={weights['candidate_long']:.3f}"
    f" ranks={ranks}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_phoenix_simulation() {
  local payload response detail rc
  payload=$(cat <<JSON
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
  "user_id": "$E2E_USER_ID",
  "request_id": "phoenix_sim"
}
JSON
)
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "phoenix simulate failed"
    return "$rc"
  fi
  detail=$(PHOENIX_RESPONSE="$response" python3 - <<'PY' 2>&1
import json
import os

data = json.loads(os.environ["PHOENIX_RESPONSE"])
assert data["scoring_mode"] == "phoenix"
phoenix = data.get("phoenix_actions") or {}
assert phoenix, "missing phoenix_actions"

for key in [
    "like", "reply", "repost", "quote", "click", "profile_click", "video_view",
    "photo_expand", "share", "share_dm", "share_link", "dwell", "follow_author",
    "quoted_click", "not_interested", "block", "mute", "report", "dwell_time",
]:
    assert 0.0 <= phoenix[key] <= 1.0, f"{key} out of range"

warnings = data.get("warnings", [])
assert not any("Phoenix unavailable" in warn for warn in warnings)

print(
    f"final={data['final_score']:.3f} phoenix_like={phoenix['like']:.3f}"
    f" warnings={len(warnings)}"
)
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_hybrid_blend() {
  local payload_heuristic payload_phoenix payload_hybrid response_heuristic response_phoenix response_hybrid detail rc
  payload_heuristic=$(cat <<JSON
{
  "text": "Hybrid blend test tweet",
  "media": "none",
  "has_link": false,
  "followers": 1800,
  "following": 320,
  "account_age_days": 1000,
  "avg_engagement_rate": 0.025,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 14,
  "novelty": 0.5,
  "timeliness": 0.5,
  "topic_saturation": 0.3,
  "audience_fit": 0.65,
  "controversy": 0.15,
  "sentiment": 0.1,
  "use_ai": false,
  "scoring_mode": "heuristic",
  "user_id": "$E2E_USER_ID",
  "request_id": "blend_heuristic"
}
JSON
)
  payload_phoenix=$(cat <<JSON
{
  "text": "Hybrid blend test tweet",
  "media": "none",
  "has_link": false,
  "followers": 1800,
  "following": 320,
  "account_age_days": 1000,
  "avg_engagement_rate": 0.025,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 14,
  "novelty": 0.5,
  "timeliness": 0.5,
  "topic_saturation": 0.3,
  "audience_fit": 0.65,
  "controversy": 0.15,
  "sentiment": 0.1,
  "use_ai": false,
  "scoring_mode": "phoenix",
  "user_id": "$E2E_USER_ID",
  "request_id": "blend_phoenix"
}
JSON
)
  payload_hybrid=$(cat <<JSON
{
  "text": "Hybrid blend test tweet",
  "media": "none",
  "has_link": false,
  "followers": 1800,
  "following": 320,
  "account_age_days": 1000,
  "avg_engagement_rate": 0.025,
  "posts_per_day": 1.0,
  "verified": false,
  "hour_of_day": 14,
  "novelty": 0.5,
  "timeliness": 0.5,
  "topic_saturation": 0.3,
  "audience_fit": 0.65,
  "controversy": 0.15,
  "sentiment": 0.1,
  "use_ai": false,
  "scoring_mode": "hybrid",
  "phoenix_weight": 0.6,
  "user_id": "$E2E_USER_ID",
  "request_id": "blend_hybrid"
}
JSON
)
  response_heuristic=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_heuristic" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "blend heuristic simulate failed"
    return "$rc"
  fi
  response_phoenix=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_phoenix" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "blend phoenix simulate failed"
    return "$rc"
  fi
  response_hybrid=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload_hybrid" \
    "$SIMULATE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "blend hybrid simulate failed"
    return "$rc"
  fi
  detail=$(HEURISTIC_RESPONSE="$response_heuristic" PHOENIX_RESPONSE="$response_phoenix" HYBRID_RESPONSE="$response_hybrid" python3 - <<'PY' 2>&1
import json
import os

heuristic = json.loads(os.environ["HEURISTIC_RESPONSE"])["actions"]
phoenix = json.loads(os.environ["PHOENIX_RESPONSE"]).get("phoenix_actions") or {}
hybrid = json.loads(os.environ["HYBRID_RESPONSE"])["actions"]
assert phoenix, "missing phoenix actions"

fields = [
    "like", "reply", "repost", "quote", "click", "profile_click", "video_view",
    "photo_expand", "share", "share_dm", "share_link", "dwell", "follow_author",
    "quoted_click", "not_interested", "block", "mute", "report", "dwell_time",
]

for key in fields:
    low = min(heuristic[key], phoenix[key])
    high = max(heuristic[key], phoenix[key])
    value = hybrid[key]
    assert low - 1e-6 <= value <= high + 1e-6, f"{key} out of range"

print(f"fields_checked={len(fields)}")
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

check_compare_rankings() {
  local payload response detail rc
  payload=$(cat <<JSON
{
  "scoring_mode": "phoenix",
  "user_id": "$E2E_USER_ID",
  "candidates": [
    {
      "request_id": "compare_a",
      "text": "Compare draft A",
      "media": "none",
      "has_link": false,
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
      "controversy": 0.1,
      "sentiment": 0.2,
      "use_ai": false
    },
    {
      "request_id": "compare_b",
      "text": "Compare draft B",
      "media": "none",
      "has_link": true,
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
      "controversy": 0.1,
      "sentiment": 0.2,
      "use_ai": false
    },
    {
      "request_id": "compare_c",
      "text": "Compare draft C",
      "media": "none",
      "has_link": false,
      "is_oon": true,
      "followers": 1200,
      "following": 350,
      "account_age_days": 900,
      "avg_engagement_rate": 0.02,
      "posts_per_day": 1.4,
      "verified": false,
      "hour_of_day": 10,
      "novelty": 0.2,
      "timeliness": 0.3,
      "topic_saturation": 0.6,
      "audience_fit": 0.4,
      "controversy": 0.3,
      "sentiment": -0.2,
      "use_ai": false
    }
  ]
}
JSON
)
  response=$(curl -fsS --max-time "$CURL_TIMEOUT" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$COMPARE_URL")
  rc=$?
  if [[ "$rc" -ne 0 ]]; then
    set_detail "compare simulate failed"
    return "$rc"
  fi
  detail=$(COMPARE_RESPONSE="$response" python3 - <<'PY' 2>&1
import json
import os

data = json.loads(os.environ["COMPARE_RESPONSE"])
results = data.get("results", [])
assert len(results) == 3

ranks = [entry["rank"] for entry in results]
assert sorted(ranks) == [1, 2, 3]

scores = [entry["response"]["final_score"] for entry in results]
assert scores == sorted(scores, reverse=True)

modes = [entry["response"]["scoring_mode"] for entry in results]
assert all(mode == "phoenix" for mode in modes)

request_ids = {entry["response"]["request_id"] for entry in results}
assert request_ids == {"compare_a", "compare_b", "compare_c"}

print(f"ranks={ranks} scores={[round(score, 3) for score in scores]}")
PY
)
  rc=$?
  set_detail "$detail"
  return "$rc"
}

print_report() {
  local total passed duration_ms duration_sec
  total=${#CHECK_NAMES[@]}
  passed=$((total - FAILED_CHECKS))
  duration_ms=$(( $(now_ms) - START_MS ))
  duration_sec=$(python3 - <<PY
print(round($duration_ms / 1000.0, 2))
PY
)
  echo ""
  echo "E2E report"
  echo "=========="
  echo "Duration: ${duration_sec}s"
  echo "Checks: ${total} | Passed: ${passed} | Failed: ${FAILED_CHECKS}"
  if [[ -n "$LOG_RUN_DIR" ]]; then
    echo "Logs: $LOG_RUN_DIR (phoenix.log simulator.log)"
    if [[ "$E2E_VERBOSE" == "1" ]]; then
      echo "Logs: uv_sync.log in $LOG_RUN_DIR"
    fi
  else
    echo "Logs: phoenix=$PHOENIX_LOG simulator=$LOG_FILE"
    if [[ "$E2E_VERBOSE" == "1" ]]; then
      echo "Logs: uv_sync=$UV_SYNC_LOG"
    fi
  fi
  if [[ "$E2E_KEEP_TMP" == "1" || "$FAILED_CHECKS" -gt 0 ]]; then
    echo "Temp: $TMP_DIR"
  fi
  echo ""
  for idx in "${!CHECK_NAMES[@]}"; do
    local status="PASS"
    if [[ "${CHECK_STATUS[$idx]}" -ne 0 ]]; then
      status="FAIL"
    fi
    if [[ -n "${CHECK_DETAILS[$idx]}" ]]; then
      printf ' - [%s] %s (%sms) %s\n' "$status" "${CHECK_NAMES[$idx]}" "${CHECK_DURATION_MS[$idx]}" "${CHECK_DETAILS[$idx]}"
    else
      printf ' - [%s] %s (%sms)\n' "$status" "${CHECK_NAMES[$idx]}" "${CHECK_DURATION_MS[$idx]}"
    fi
  done
}

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
  if [[ "$E2E_KEEP_TMP" == "1" || "$FAILED_CHECKS" -gt 0 ]]; then
    log "INFO" "Keeping temp dir: $TMP_DIR"
  else
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT

require_cmd curl
require_cmd python3
require_cmd uv
require_cmd cargo

START_MS=$(now_ms)

log "INFO" "E2E starting"
log "INFO" "Host=$HOST Port=$PORT PhoenixPort=$PHOENIX_PORT"
log "INFO" "Temp dir=$TMP_DIR"
if [[ -n "$LOG_RUN_DIR" ]]; then
  log "INFO" "Log output dir=$LOG_RUN_DIR"
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

log "STEP" "Phoenix: install dependencies"
pushd "$ROOT_DIR/phoenix" >/dev/null
if [[ "$E2E_VERBOSE" == "1" ]]; then
  uv sync --project .
else
  uv sync --project . >"$UV_SYNC_LOG" 2>&1
fi
log "INFO" "Phoenix log: $PHOENIX_LOG"
PHOENIX_PID=$(start_process "$PHOENIX_LOG" uv run uvicorn service.server:app --reload --port "$PHOENIX_PORT")
popd >/dev/null

if ! wait_for_url "Phoenix docs" "http://$HOST:$PHOENIX_PORT/docs" 180; then
  tail -n 200 "$PHOENIX_LOG" >&2 || true
  fatal "Phoenix service failed to start."
fi

log "STEP" "Simulator: start server"
pushd "$ROOT_DIR/simulator" >/dev/null
log "INFO" "Simulator log: $LOG_FILE"
SERVER_PID=$(start_process "$LOG_FILE" cargo run -- serve --host "$HOST" --port "$PORT")
popd >/dev/null

health_url="http://$HOST:$PORT/api/health"
if ! wait_for_url "Simulator health" "$health_url" 180; then
  tail -n 200 "$LOG_FILE" >&2 || true
  fatal "Simulator failed to start."
fi

SIM_BASE_URL="http://$HOST:$PORT"
PHOENIX_URL="http://$HOST:$PHOENIX_PORT/rank"
CONFIG_URL="$SIM_BASE_URL/api/config"
CONFIG_WEIGHTS_URL="$SIM_BASE_URL/api/config/weights"
SIMULATE_URL="$SIM_BASE_URL/api/simulate"
COMPARE_URL="$SIM_BASE_URL/api/simulate/compare"
USERS_URL="$SIM_BASE_URL/api/users"
USER_HISTORY_URL="$SIM_BASE_URL/api/users/$E2E_USER_ID/history"

run_check "Load scoring config" check_config
run_check "Update scoring weights" check_weights_update
run_check "Upsert user profile history" check_user_profile
run_check "Heuristic baseline scoring" check_baseline_heuristic
run_check "Link boost signals" check_link_signal
run_check "OON penalty applied" check_oon_penalty
run_check "Video VQV threshold" check_video_vqv_threshold
run_check "Weighted score formula" check_weighted_score_formula
run_check "Impressions scale with followers" check_impressions_scale
run_check "Negative feedback penalties" check_negative_feedback
run_check "Dwell time growth" check_dwell_time_growth
run_check "Auto history creation" check_auto_history_creation
run_check "Phoenix /rank scoring" check_phoenix_rank
run_check "Phoenix simulation scoring" check_phoenix_simulation
run_check "Hybrid blending bounds" check_hybrid_blend
run_check "Compare ranking ordering" check_compare_rankings

print_report

if [[ "$FAILED_CHECKS" -gt 0 ]]; then
  log "ERROR" "E2E checks failed."
  exit 1
fi

log "INFO" "E2E checks passed."
