# Virality Simulator Web

React + Vite frontend that mirrors the simulator model and can call the local Rust proxy for Grok scoring.

## Install

```sh
cd webapp
npm install
```

## Dev

```sh
npm run dev
```

The dev server proxies `/api` to `http://127.0.0.1:8787`.

## Build

```sh
npm run build
```

Serve the built assets with:

```sh
cd simulator
cargo run -- serve --web-root ../webapp/dist
```
