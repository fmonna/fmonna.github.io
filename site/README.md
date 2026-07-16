# fmonna-site

## Stack

- **Dioxus 0.7** (`fullstack` + `router` features) — reactive Rust UI, compiled to WASM for the client.
- **SSG prerendering** — `dx bundle --web --ssg` generates static HTML into `public/` for first paint / SEO.
- **Typst** (planned, not yet wired) — generates the EN-full and FR-2-page PDFs from the same Markdown source.

## Toolchain (one-time, local)

```sh
rustup target add wasm32-unknown-unknown
# dx CLI — prebuilt binary from the v0.7.9 release:
curl -fsSL -o /tmp/dx.tar.gz \
  https://github.com/DioxusLabs/dioxus/releases/download/v0.7.9/dx-aarch64-unknown-linux-gnu.tar.gz
tar -xzf /tmp/dx.tar.gz -C ~/.cargo/bin dx
dx --version   # dioxus 0.7.9
```

## Develop

```sh
cd site
dx serve --web        # dev server with hot reload at http://127.0.0.1:8080
```

## Build the static site (what CI deploys)

```sh
dx bundle --web --ssg   # emits prerendered HTML + WASM into public/
```

`public/` is what GitHub Pages serves.
