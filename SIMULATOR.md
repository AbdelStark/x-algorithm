# Virality Simulator

This simulator is a modeling sandbox for the X "For You" ranking system. It turns a tweet draft and account context into a virality estimate, with action probabilities, impressions, and suggestions. The goal is to be transparent and directionally useful, not to claim parity with production.

## What is real vs simulated

| Aspect | Source in repo | How the simulator uses it | Status |
| --- | --- | --- | --- |
| Pipeline stages (hydrate -> source -> score -> filter -> select) | `README.md`, `home-mixer/`, `candidate-pipeline/` | Mirrors the stage order as a conceptual flow | Conceptual only |
| Engagement action types (like, reply, repost, click, share, etc.) | `home-mixer/scorers/weighted_scorer.rs`, `phoenix/` | Uses the same action taxonomy in `ActionProbs` | Implemented |
| Weighted scoring over actions | `home-mixer/scorers/weighted_scorer.rs` | Weighted scorer + config in `simulator/src/scoring/` | Implemented (configurable) |
| OON vs in-network handling | `home-mixer/scorers/oon_scorer.rs` | OON multiplier in scoring pipeline + impressions split | Approximation |
| Author diversity | `home-mixer/scorers/author_diversity_scorer.rs` | Exponential decay scorer in pipeline | Approximation |
| Phoenix prediction model | `phoenix/` | Optional Phoenix JAX service (`phoenix/service/`) | Implemented (optional) |
| Candidate retrieval (Thunder + Phoenix Retrieval) | `thunder/`, `phoenix/` | Not executed; we estimate aggregate reach | Simulated |
| Filters/hydrators | `home-mixer/filters/`, `home-mixer/candidate_hydrators/` | Not executed | Omitted |
| User history | `simulator/src/user/` | Stored profiles + synthetic history for Phoenix | Optional |
| Calibration | `simulator/src/calibration/` | Metrics + weight tuning via CLI | Implemented |

Important: The open-source release does not include `home-mixer/params` or `home-mixer/clients` (see `home-mixer/lib.rs`). Those missing parameters and production clients are why the simulator cannot use real weights or production embeddings.

Also important: Grok analysis is not part of the For You algorithm. It is an optional, local assist that scores text-level signals (hook, clarity, shareability) and feeds those into the simulator.

## Architecture

```
CLI (cargo run -- simulate)        Web UI (Vite + React)
           |                                  |
           |                                  v
           |                      HTTP + SSE /api/simulate
           v                                  |
    simulator/src/lib.rs <---------------- simulator/src/server.rs
           |                                  |
   Heuristic + optional Grok                   |
           |                                  |
   Optional Phoenix service  <----------------+
           |
    SimulationOutput (score, impressions, actions, suggestions)
```

Key modules:
- `simulator/src/lib.rs`: core heuristics, signal blending, scoring pipeline.
- `simulator/src/llm.rs`: Grok scoring (optional) + SSE token streaming.
- `simulator/src/server.rs`: API + SSE + snapshot storage.
- `simulator/src/scoring/`: weighted + diversity + OON scorers.
- `simulator/src/user/`: user profiles + synthetic history.
- `phoenix/service/`: FastAPI wrapper for Phoenix JAX model.
- `webapp/`: UI, SSE feed, snapshot compare, transcript panel.

## How it leverages the open-source algorithm

The simulator follows the public algorithm shape and vocabulary:
- **Two-stage idea**: in-network vs out-of-network split (`Thunder` + `Phoenix`), echoed in impressions estimates.
- **Multi-action scoring**: uses action probabilities and a weighted sum like `WeightedScorer`.
- **Diversity and penalties**: mirrors author diversity and negative actions (block/mute/report) as score dampeners.
- **Ranking intent**: output is a single ranked score plus action-level breakdown, similar to `Phoenix` outputs.

What it does not do:
- It can run the Phoenix model via the optional service, but it uses synthetic embeddings and weights.
- It does not ingest or hydrate real candidates.
- It does not use missing production weights (params are excluded).

## Credibility: current reality

No bullshit: this is a heuristic simulator with optional LLM assist. It is **not** a faithful reconstruction of the production ranking system because:
- The real weights and prediction services are excluded from the open-source release.
- There is no access to user graph, engagement history, or actual candidate sets.
- The Phoenix model is not wired into the simulator.

What it is good for:
- Comparing drafts (A/B text variants).
- Understanding relative effects of hook, clarity, media, and share intent.
- Visualizing how action weights compound.

What it is not good for:
- Predicting real-world reach.
- Comparing two different accounts accurately.
- Estimating actual impressions at scale without calibration data.

## Limitations

- No real user graph or production embeddings.
- No real candidate retrieval or filtering (duplicates, blocks, muted keywords).
- Heuristic weights are not from production params.
- LLM scoring is subjective and model-dependent.
- Impressions are a synthetic estimate, not based on real inventory.

## Ideas for improvement

Short-term:
- Add real data sources (X API, opt-in exports) for calibration.
- Expand filter/hydration approximations.

Medium-term:
- Replace synthetic embeddings with real embedding tables.
- Tune weights with larger calibration sets.

Long-term:
- Integrate a real candidate retrieval pipeline.
- Build benchmarking suites for known tweets and expected ranking deltas.

## Running the simulator

All-in-one (Phoenix + simulator + webapp):
```
./scripts/dev.sh
```
Flags:
- `--log-dir ./output/dev-logs` to save logs per run.
- `--skip-uv-sync` or `--skip-npm-install` to skip dependency installs.

Docker Compose:
```
docker compose up --build
```
Then open `http://localhost:5173` (webapp), `http://localhost:8787/api/health` (simulator), and `http://localhost:8000/docs` (Phoenix).

CLI:
```
cargo run -- --text "this is a banger" --followers 1000 --following 50
```

Server + UI:
```
cargo run -- serve --port 8787
```
Then open `http://localhost:8787`.

Phoenix service (optional):
```
cd phoenix
uv run uvicorn service.server:app --reload --port 8000
```

Webapp dev server (optional):
```
cd webapp
npm install
npm run dev -- --host 127.0.0.1 --port 5173
```

Environment variables:
- `XAI_API_KEY`: enable Grok scoring.
- `XAI_MODEL`: override model name (default `grok-2-latest`).
- `XAI_API_BASE`: override API base URL.
- `SIM_SNAPSHOT_PATH`: snapshot storage file path (default `data/snapshots.json`).
- `PHOENIX_ENDPOINT`: Phoenix service URL (default `http://localhost:8000`).
- `PHOENIX_TIMEOUT_MS`: Phoenix request timeout (ms).
- `SCORING_MODE`: `heuristic`, `phoenix`, or `hybrid`.
- `PHOENIX_WEIGHT`: blend weight for hybrid mode.
- `SCORING_CONFIG_PATH`: config file path (default `config/scoring.toml`).
- `USER_PROFILES_PATH`: user profile storage path.
- `CALIBRATION_DATA_PATH`: default calibration dataset path.

## Streaming notes

The UI subscribes to `/api/simulate/stream` for SSE events. If the xAI API streams tokens, you will see live transcript updates. If the API only returns a final response, you will still see progress updates and the final raw response once complete.
