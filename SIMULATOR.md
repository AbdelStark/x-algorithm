# Virality Simulator

This simulator is a modeling sandbox for the X "For You" ranking system. It turns a tweet draft and account context into a virality estimate, with action probabilities, impressions, and suggestions. The goal is to be transparent and directionally useful, not to claim parity with production.

## What is real vs simulated

| Aspect | Source in repo | How the simulator uses it | Status |
| --- | --- | --- | --- |
| Pipeline stages (hydrate -> source -> score -> filter -> select) | `README.md`, `home-mixer/`, `candidate-pipeline/` | Mirrors the stage order as a conceptual flow | Conceptual only |
| Engagement action types (like, reply, repost, click, share, etc.) | `home-mixer/scorers/weighted_scorer.rs`, `phoenix/` | Uses the same action taxonomy in `ActionProbs` | Implemented |
| Weighted scoring over actions | `home-mixer/scorers/weighted_scorer.rs` | Uses a weighted sum in `simulator/src/lib.rs` | Partial (weights are heuristic) |
| OON vs in-network handling | `home-mixer/scorers/oon_scorer.rs` | Applies a simplified multiplier + impressions split | Approximation |
| Author diversity | `home-mixer/scorers/author_diversity_scorer.rs` | Uses a simplified penalty (not full per-feed sequencing) | Approximation |
| Phoenix prediction model | `phoenix/` | Not executed; we simulate probabilities | Simulated |
| Candidate retrieval (Thunder + Phoenix Retrieval) | `thunder/`, `phoenix/` | Not executed; we estimate aggregate reach | Simulated |
| Filters/hydrators | `home-mixer/filters/`, `home-mixer/candidate_hydrators/` | Not executed | Omitted |

Important: The open-source release does not include `home-mixer/params` or `home-mixer/clients` (see `home-mixer/lib.rs`). Those missing parameters and production clients are why the simulator cannot use real weights or a real prediction backend.

Also important: Grok analysis is not part of the For You algorithm. It is an optional, local assist that scores text-level signals (hook, clarity, shareability) and feeds those into the simulator.

## Architecture

```
CLI (cargo run -- simulate)        Web UI (Vite + React)
           |                                  |
           |                                  v
           |                      HTTP + SSE /api/simulate
           v                                  |
    simulator/src/lib.rs <---------------- simulator/src/server.rs
           |
   Heuristic + optional Grok
           |
    SimulationOutput (score, impressions, actions, suggestions)
```

Key modules:
- `simulator/src/lib.rs`: core heuristics, signal blending, action rates, weighted score.
- `simulator/src/llm.rs`: Grok scoring (optional) + SSE token streaming.
- `simulator/src/server.rs`: API + SSE + snapshot storage.
- `webapp/`: UI, SSE feed, snapshot compare, transcript panel.

## How it leverages the open-source algorithm

The simulator follows the public algorithm shape and vocabulary:
- **Two-stage idea**: in-network vs out-of-network split (`Thunder` + `Phoenix`), echoed in impressions estimates.
- **Multi-action scoring**: uses action probabilities and a weighted sum like `WeightedScorer`.
- **Diversity and penalties**: mirrors author diversity and negative actions (block/mute/report) as score dampeners.
- **Ranking intent**: output is a single ranked score plus action-level breakdown, similar to `Phoenix` outputs.

What it does not do:
- It does not run the Phoenix model (`phoenix/` is a JAX example, not wired into this simulator).
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

- No real user graph, history, or personalized embeddings.
- No real candidate retrieval or filtering (duplicates, blocks, muted keywords).
- Heuristic weights are not from production params.
- LLM scoring is subjective and model-dependent.
- Impressions are a synthetic estimate, not based on real inventory.

## Ideas for improvement

Short-term:
- Port the public weights structure from `WeightedScorer` and allow a tunable config file.
- Simulate candidate sets and apply `AuthorDiversityScorer` more faithfully.
- Calibrate action-rate distributions against a small labeled dataset.

Medium-term:
- Wire the Phoenix JAX model into a service and use it to score candidates.
- Add a synthetic user history generator to mimic the retrieval and ranking inputs.
- Implement a fuller filter/hydration pass (muted keywords, recency, repeats).

Long-term:
- Integrate a real data source (X API, opt-in analytics exports) for calibration.
- Add feedback loops and fit weights against observed engagements.
- Build benchmarking suites for known tweets and expected relative ranking.

## Running the simulator

CLI:
```
cargo run -- --text "this is a banger" --followers 1000 --following 50
```

Server + UI:
```
cargo run -- serve --port 8787
```
Then open `http://localhost:8787`.

Environment variables:
- `XAI_API_KEY`: enable Grok scoring.
- `XAI_MODEL`: override model name (default `grok-2-latest`).
- `XAI_API_BASE`: override API base URL.
- `SIM_SNAPSHOT_PATH`: snapshot storage file path (default `data/snapshots.json`).

## Streaming notes

The UI subscribes to `/api/simulate/stream` for SSE events. If the xAI API streams tokens, you will see live transcript updates. If the API only returns a final response, you will still see progress updates and the final raw response once complete.
