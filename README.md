# sylk

[![crates.io](https://img.shields.io/crates/v/sylk.svg)](https://crates.io/crates/sylk)
[![docs.rs](https://docs.rs/sylk/badge.svg)](https://docs.rs/sylk)
[![CI](https://github.com/efbx/sylk/actions/workflows/ci.yml/badge.svg)](https://github.com/efbx/sylk/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/efbx/sylk/blob/main/LICENSE)

Typed resources, server functions and reconnecting hooks for [Dioxus](https://dioxuslabs.com) apps, fullstack or standalone.

The [crate documentation](https://docs.rs/sylk) is the complete guide, covering feature crates, startup rules and standalone details.

## Features

- **Resources**: a type-keyed registry filled once at startup and injected into server functions.
- **Resource functions**: `#[sylk::resource]` functions run at startup in dependency order.
- **Packs**: `#[derive(Pack)]` groups several resources into one handle.
- **Registration**: startup collects every `#[sylk::register]` implementation of a trait into an ordered `Seq<dyn Trait>`.
- **Collected traits**: `#[sylk::collect]` declares a trait whose `Seq` exists even with no implementations; `#[sylk::collect(N)]` requires exactly `N`.
- **Standalone mode**: the same `#[sylk::server]` functions run in process, without a server.
- **Live hooks**: `use_streaming`, `use_server_events` and `use_websocket` reconnect automatically; their `_with` forms take `Reconnect` options for an idle timeout and jitter.

## Installation

Requires Rust 1.95 or newer.

Fullstack:

```toml
[dependencies]
dioxus = { version = "0.7.10", features = ["fullstack"] }
dioxus-fullstack = { version = "0.7.10", default-features = false }
sylk = "0.5"

[features]
web = ["dioxus/web"]
server = ["dioxus/server", "sylk/server"]
```

Standalone:

```toml
[dependencies]
dioxus = { version = "0.7.10", features = ["desktop"] }
sylk = "0.5"

[features]
default = ["standalone"]
standalone = ["sylk/standalone"]
```

Library crates that export resources, registrations or server functions follow the [feature crate rules](https://docs.rs/sylk/latest/sylk/#feature-crates).

## Example

```rust
use dioxus::prelude::*;
#[cfg(feature = "server")]
use {
    sylk::Res,
    std::sync::{Arc, atomic::{AtomicU64, Ordering}},
};

#[cfg(feature = "server")]
#[derive(Clone, Default)]
struct Clicks {
    count: Arc<AtomicU64>,
}

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    sylk::serve(
        async |resources| {
            resources.insert(Clicks::default());
            Ok(())
        },
        || dioxus::server::router(App),
    );
}

#[server(clicks: Res<Clicks>)]
async fn bump() -> Result<u64> {
    Ok(clicks.count.fetch_add(1, Ordering::Relaxed) + 1)
}

#[component]
fn App() -> Element {
    rsx! { button { onclick: move |_| async move { _ = bump().await; }, "+1" } }
}
```

Run with `dx serve --web --fullstack`. More examples are in [`sylk-examples/examples`](https://github.com/efbx/sylk/tree/main/sylk-examples/examples).

## Feature flags

| Flag | Enables |
|---|---|
| *(none)* | `Resources`, `Res<T>`, resource functions, packs, registration, `#[sylk::server]` and the client hooks; on wasm targets, sylk enables `dioxus-fullstack`'s `web` feature itself |
| `server` | `sylk::serve`, `sylk::client_rendered`, `Resources::current`, `Res<T>` as an axum extractor |
| `standalone` | `sylk::launch`, `sylk::install` on native targets, in-process server functions |

`server` and `standalone` must not be enabled in the same build.

## License

[MIT](https://github.com/efbx/sylk/blob/main/LICENSE)
