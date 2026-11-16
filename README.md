# Dashy for lemonfiber

**One page linking everything, with live status.**
Dashy stands in for Homepage as the dashboard. The bundle offers it and does not ship it.
Without it: Remember a dozen URLs and ports.

## How lemonfiber carries it

This repository holds Dashy's manifest and the adapter that speaks lemonfiber's
`dashboard.show` contract beside Dashy: a Rust crate built on the core's adapter kit,
into the image `ghcr.io/lemonfiber/dashy-adapter`, published from every commit on
`main` and from each tag the release train cuts. The adapter writes the groups,
entries and widgets the core hands it into `conf.yml`, the page Dashy reads in its
configuration directory, which it shares.

Dashy's own server answers anybody who reaches the page, and its `/cors-proxy`,
`/status-check` and `/ping-check` reach any address. The image lemonfiber runs is
[`lemonfiber-dashy`](https://github.com/lemonfiber/lemonfiber-dashy)'s, which holds those
endpoints to the addresses the page names, follows no redirect and pings nothing.
`dashy.unnamed-refused` holds it to refusing a service the page does not name,
`dashy.named-check-answers` to still asking one the page names,
`dashy.redirect-not-followed` to following no redirect, `dashy.check-takes-no-header`
to sending no header a caller hands a check, `dashy.proxy-forwards-only-get` to
forwarding nothing but a GET, and `dashy.pings-nothing` to refusing every ping. The page this plugin ships names Dashy's own addresses for those
proofs, until the adapter's first write replaces it.
Every page the adapter writes, and the one this plugin ships, sets
`appConfig.preventWriteToDisk`, so Dashy refuses to save the page for anybody.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The plugin's manifest |
| `Cargo.toml`, `src/` | The adapter, built on `lemonfiber-adapter-kit` |
| `Dockerfile` | The adapter's image |
| `config/dashy/` | The page Dashy starts with, written into its configuration directory |
| `recordings/dashy/` | The responses its proofs are proved against |
| `conformance/` | One recording per case of `dashboard.show@1`, answered by the adapter in front of the pinned Dashy |
| `targets.toml` | The lemonfiber its checks run against, and the core the adapter is built on |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
proof against its recording, and every pinned digest. `rust.yml` holds the
adapter to its format, lints, tests, every line covered and its dependency
policy. This repository holds no Python of its own. `just ci` runs the gates
that need nothing but this tree.

## Licence

The data in this repository is under the licence in `LICENSE`.
Dashy itself is under the MIT licence and is not distributed here: this
repository names an image, it does not contain one.
