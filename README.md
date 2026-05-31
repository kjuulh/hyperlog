# hyperlog

![demo](./assets/demo.gif)

> **Server (`hyperlog-serve`).** Besides the TUI, this repo ships a gRPC + HTTP
> server (`crates/hyperlog-server`, bin `hyperlog-serve`) backing the
> [hyperlog-app](https://git.kjuulh.io/kjuulh/hyperlog-app) client (users/auth,
> a scalable bounded tree view, move/reorder, archive/restore, and per-item
> due-date/links metadata).
>
> **Container image.** CI publishes a multi-arch (amd64/arm64) image to
> `git.kjuulh.io/kjuulh/hyperlog-server` (`latest` / `main` / `main-<sha>`) on
> every push to `main`. Run it with Postgres:
>
> ```sh
> docker run --rm -p 4000:4000 \
>   -e DATABASE_URL=postgres://user:pass@host:5432/hyperlog \
>   -e HYPERLOG_JWT_SECRET=change-me \
>   git.kjuulh.io/kjuulh/hyperlog-server:latest
> ```
>
> Migrations run on startup. Env: `EXTERNAL_GRPC_HOST` (`:4000`), `EXTERNAL_HOST`
> (`:3000`), `INTERNAL_HOST` (`:3001`), `DATABASE_URL`, `HYPERLOG_JWT_SECRET`.
> CI: `.woodpecker/` (`ci.yaml` build+push per-arch via the buildx plugin →
> `manifest.yaml` multi-arch fuse). Secret: `registry_token` (package:write on
> `git.kjuulh.io/kjuulh`); server config `WOODPECKER_PLUGINS_PRIVILEGED=woodpeckerci/plugin-docker-buildx`.

## TUI roadmap

- [x] Display todos as todos
- [x] Create sections
- [x] Edit todos
- [ ] Move items
- [x] Display summaries and limit todos
  - [-] Implement scroll
    > Maybe not required anyways
- [x] Create onboarding experience
- [-] Let users choose a root
  > Skipped for now

- [ ] Remove footguns (session with @edb)
  - [ ] Should not be able to create todos on todos (don't even enter the dialog, print a message instead)
  - [ ] Highlight bold so that non-true-color terms still function
  - [ ] Create a help menu
  - [ ] At some point create a small demo

- [ ] Add create item command
