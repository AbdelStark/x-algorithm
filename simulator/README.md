# Virality Simulator (CLI)

Heuristic simulator that approximates X For You ranking signals to estimate virality potential.
This is not an official model and does not use X data.

## Run

```sh
cargo run -- --text "Your tweet here" --followers 2500 --media image --hour 20
```

Or pipe text:

```sh
echo "Your tweet here" | cargo run -- --followers 2500 --media image --hour 20
```

## Grok-assisted mode (optional)

Set `XAI_API_KEY` in your environment (or `simulator/.env`) and pass `--ai` to blend Grok analysis into the model:

```sh
export XAI_API_KEY="..."
cargo run -- --text "Your tweet here" --ai
```

You can override the model with `--ai-model` or `XAI_MODEL`.
Use `XAI_API_BASE` if you need a custom endpoint.

## Serve the webapp

```sh
cd webapp
npm install
npm run build

cd ../simulator
cargo run -- serve --port 8787 --web-root ../webapp/dist
```

Then open `http://localhost:8787` in a browser.

## Common options

- `--followers`, `--following`, `--account-age-days`
- `--avg-engagement-rate`, `--posts-per-day`
- `--media` (`none|image|video|gif`), `--link` / `--no-link`
- `--novelty`, `--timeliness`, `--topic-saturation`, `--audience-fit`, `--controversy`
- `--details` for full action probabilities
