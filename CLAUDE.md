<identity>
X For You feed algorithm reference implementation: Rust services (home-mixer, thunder, candidate
pipeline) plus Python JAX models (phoenix).
</identity>

<environment>
You are operating in Codex CLI with full filesystem and network access.
The user sees local file changes and command outputs.
No CI or deployment automation is configured here.
</environment>

<stack>
- Rust (tokio, axum, reqwest, clap) for the simulator CLI + API proxy.
- Python 3.11 (JAX, Haiku, NumPy) managed by uv in `phoenix/`.
- Web: React + Vite + TypeScript in `webapp/`.
- Tests: pytest (Python).
</stack>

<structure>
- `home-mixer/`: Rust gRPC orchestrator; pipeline wiring in
  `home-mixer/candidate_pipeline/phoenix_candidate_pipeline.rs`; stages in `candidate_hydrators/`,
  `filters/`, `scorers/`, `sources/`, `selectors/`, `query_hydrators/`.
- `candidate-pipeline/`: Rust traits and pipeline execution helpers.
- `thunder/`: Rust in-memory post store + Kafka ingestion + gRPC service.
- `phoenix/`: Python JAX retrieval/ranking models; entrypoints `run_ranker.py`,
  `run_retrieval.py`; tests `test_*.py`.
- `simulator/`: Rust CLI + local API proxy for Grok scoring.
- `webapp/`: React/Vite UI for the simulator.
- `README.md`: architecture overview.
</structure>

<conventions>
<patterns>
  <do>
    Keep Rust modules in snake_case and exported via `mod.rs`.
    Implement pipeline stages by conforming to `Filter/Hydrator/Source/Scorer/Selector` traits and
    wiring them in `home-mixer/candidate_pipeline/phoenix_candidate_pipeline.rs`.
    Use type hints and dataclasses/NamedTuple in Python; keep array shapes stable.
  </do>
  <dont>
    Don't add new dependencies or edit `uv.lock` unless requested.
    Don't change stage ordering without updating pipeline wiring and tests/docs.
    Don't assume Rust code builds; missing internal crates are expected.
  </dont>
</patterns>
</conventions>

<commands>
| Task | Command | Notes |
| --- | --- | --- |
| Install (Phoenix) | `uv sync` | Run in `phoenix/` |
| Run ranker | `uv run run_ranker.py` | Run in `phoenix/` |
| Run retrieval | `uv run run_retrieval.py` | Run in `phoenix/` |
| Tests | `uv run pytest test_recsys_model.py test_recsys_retrieval_model.py` | Run in `phoenix/` |
| Web dev | `npm run dev` | Run in `webapp/` |
| Web build | `npm run build` | Run in `webapp/` |
| Simulator API | `cargo run -- serve --web-root ../webapp/dist` | Run in `simulator/` |
</commands>

<workflows>
<add_pipeline_stage>
1. Add module under `home-mixer/<stage_dir>/`.
2. Export it in `home-mixer/<stage_dir>/mod.rs`.
3. Wire it into `home-mixer/candidate_pipeline/phoenix_candidate_pipeline.rs` in stage order.
4. Update tests/docs if behavior changes.
</add_pipeline_stage>
<modify_phoenix_model>
1. Update `phoenix/recsys_model.py` or `phoenix/recsys_retrieval_model.py` plus helpers in
   `phoenix/grok.py`.
2. Adjust entrypoints (`phoenix/run_ranker.py` / `phoenix/run_retrieval.py`) if signatures change.
3. Update tests in `phoenix/test_*.py`.
4. Run the Phoenix tests.
</modify_phoenix_model>
</workflows>

<boundaries>
<do>
  Keep changes scoped to local code; prefer edits in `phoenix/` for runnable behavior.
</do>
<dont>
  Do not modify `.env*` files or add secrets.
  Do not attempt deployments, Kafka runs, or prod service calls.
  Do not edit `LICENSE`/`CODE_OF_CONDUCT` unless requested.
  Do not stub missing internal modules (`home-mixer/clients`, `home-mixer/params`,
  `home-mixer/util`, `thunder/strato_client`) unless asked.
</dont>
</boundaries>

<troubleshooting>
- Rust compile errors about missing `xai_*` crates or `clients/` modules mean the repo is
  incomplete; treat Rust code as reference-only.
- `ModuleNotFoundError: grok` when running scripts: run from `phoenix/` or set
  `PYTHONPATH=phoenix`.
- If JAX/Haiku imports fail, run `uv sync` in `phoenix/` and ensure Python 3.11.
</troubleshooting>

<skills>
None (no local skills defined).
</skills>
