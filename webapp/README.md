# Virality Simulator (Web)

Static webapp version of the virality simulator. Open `webapp/index.html` in a browser and adjust inputs to see predicted score, impressions, and suggestions.

The model is heuristic and mirrors the Rust CLI formulas for consistency.

## Grok-assisted mode

Run the local proxy server so the webapp can call the xAI API without exposing your key (set `XAI_API_KEY` or put it in `simulator/.env`):

```sh
cd simulator
export XAI_API_KEY="..."
cargo run -- serve --port 8787
```

Then open `http://localhost:8787` and enable the Grok toggle.
