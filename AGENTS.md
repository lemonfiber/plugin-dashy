# AGENTS.md — plugin-dashy

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

Dashy as a first-party lemonfiber plugin, offered in Homepage's place as the dashboard:
`plugin.toml` and the adapter that speaks the `dashboard.show@1` contract beside Dashy,
by writing the page it reads, `conf.yml`, into its configuration directory. The adapter
is the Rust crate at the root, built on `lemonfiber-adapter-kit` from the core at the
commit `Cargo.toml` pins, into the image the `Dockerfile` builds. `release.yml`
publishes that image from every commit on `main` and from the train's tag. The
[bundle contract](https://github.com/lemonfiber/spec/blob/main/20-architecture/contracts/bundle.md)
says how the catalogue pins this repository and how lemonfiber carries it.

## The rules you cannot break

- **The plugin id is the service's id** (`ARCH-R266`). The container, the
  configuration directory, the settings and the journal keep their names. It is a
  substitute: no service declares `replaces`.
- **Dashy runs `lemonfiber-dashy`'s image** (`OPS-R93`). That image holds Dashy's
  `/cors-proxy` and `/status-check` to the addresses the page names, follows no
  redirect and refuses every ping; `dashy.unnamed-refused`, `dashy.named-check-answers`,
  `dashy.redirect-not-followed` and `dashy.pings-nothing` hold it there, against the
  addresses of Dashy's own that `config/dashy/conf.yml` names.
- **Every page sets `appConfig.preventWriteToDisk` and `disableConfiguration`**, the
  one this plugin ships and every one the adapter writes, and `dashy.save-refused`
  holds Dashy to refusing a save. Dashy's REST API stays off: `ENABLE_API` is never set.
- **Its proofs' recordings sit under `recordings/dashy/`**, and its starting
  files under `config/dashy/` (`ARCH-R271`).
- **Every service declares `reaches` and `asks_for`** (`ARCH-R270`).
- **No Python here.** The reader is spec's `first-party-plugin` workflow's.
- **No `unwrap`, `expect`, `panic` or indexing** outside tests. The lints in
  `Cargo.toml` are the core's.
- **Every file in Dashy's configuration directory is reached through
  `lemonfiber_adapter_kit::shared`**, so a link, a hardlink or a file that is not
  plain, which Dashy could plant there, is refused by name or replaced and
  never written through.
- **Every value is held to its grammar before anything is written**: an address
  is `http` or `https`, an icon a plain name or such an address, and text carries
  no control character, no markup (`<`, `>`, `"`, `'`, `` ` ``, `&`) and none of the
  variables Dashy reads from its environment (`DASHY_`, `VITE_APP_`, `VUE_APP_`).
- **Every address Dashy's server asks is a service on the stack's network.** What the
  adapter writes as a check or a status page's address is what `lemonfiber-dashy`'s
  server is let reach, so each is parsed and held to `http` or `https`, no account,
  no fragment, and a host that is one of the panel list's `hosts` (`ARCH-R434`), the
  names the core gives a check at, and one label of a service's name. Anything else,
  an IP address, a public name or a name the core did not give, is refused before
  anything is written, and so are a name a container's hosts file holds (`localhost`
  and the `ip6-` names), a container's id and a metadata or cluster name (`metadata`,
  `instance-data`, `kubernetes`) even where `hosts` names one. What is written is the parsed address, never the raw one. A status page is an origin, `scheme://host[:port]`, with
  no path or query, and its name is lowercase letters, digits and `-`, so the two
  addresses read from it name no other path.
- **A page changed by hand is never written over.** One that is neither the page
  last written nor the one this plugin ships is named in the refusal.
- **The core's crates move with the train.** The `rev` in `Cargo.toml` is the
  core commit `targets.toml` names, the same for every `lemonfiber-*` crate.
- **Moving Dashy's digest** means recording every response again against the
  new image in the same change, moving `RELEASE` and `DIGEST` in
  `src/adapter.rs` with it, and committing the `proofs.json` the checks write.
