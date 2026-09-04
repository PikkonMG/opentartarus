# Contributing

Thanks for helping. This page says how to build, what a change needs before it is merged, and how to add a game profile, which is the most common kind of change.

## Build and test

You need Rust 1.75 or newer. Build and run the two binaries as the README describes. Before you open a pull request, run:

```
cargo test --workspace
cargo clippy --workspace --all-targets
```

Every test must pass. Clippy must not report anything new. The workspace carries a small number of pre-existing warnings, so compare against `main` rather than aiming for zero. The CI workflow runs both on every push and pull request, and fails if the warning count rises above the baseline in `.github/workflows/ci.yml`.

If you have a Tartarus V2 and OpenRazer, the hardware test runs with `OPENTARTARUS_HW_TEST=1`. It is skipped otherwise.

## What a change needs

- A test for each new function. Tests should fail when the code is wrong, not just call the function.
- No magic numbers. Sizes, limits, colours and timings are named constants next to the code that uses them.
- No dead code, no placeholders, no half-finished paths.
- Handle the specific error. A catch-all that swallows everything hides bugs.
- Keep the layers apart. Profile validation lives in `opentartarus-core`. The daemon plays remaps and talks to the device. The window only draws and sends requests.
- User-facing strings in the window live in `crates/opentartarus-ui/src/theme/strings.rs`, not inline in views.
- English only, in code, comments, docs and tests.

## Commits

One change per commit. The subject line is `type(scope): subject`, in the imperative, 50 characters or fewer, no full stop. Types are `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore` and `perf`. Add a body when the why is not obvious, wrapped at 72 columns.

Examples from the history:

```
fix(ui): stop edge keys being clipped by the canvas
feat(profiles): create and delete custom profiles
```

Open pull requests against `main`.

## Adding a game profile

Profiles are JSON files in `crates/opentartarus-core/profiles/`. Copy one that is close to your game and edit it. Then:

1. Add the game to `GameId` in `crates/opentartarus-core/src/types.rs`.
2. Add the id to `SHIPPED_IDS` in `crates/opentartarus-core/src/pack.rs`, in the order it should appear in the list.
3. Add a `*_bindings_match_spec` test in the same file that pins every key, and add the id to `NEW_GAME_IDS` so the structural tests cover it.
4. Give it a lighting colour. The sidebar uses it as the profile's swatch.

Build the layout from the game's real default PC bindings, and say in the pull request where you checked them. Official manuals, the game's own wiki, or Liquipedia are fine. A layout that guesses will be sent back.

Layout rules, so every profile feels the same on the pad:

- If the game moves with WASD, movement goes on the thumb pad and Space goes on the thumb key.
- The rest row, keys 11 to 15, gets the actions the player hits most in a fight.
- Row 2, keys 6 to 10, gets the next most used, or the number keys when the game uses them for weapons or skills.
- Row 1 and row 4 get menus, map, chat and other things that are not urgent.
- Never bind a key that does nothing in the game.
- Sprint, crouch and walk usually live on Shift and Ctrl. Bind `leftshift` and `leftctrl` for those. They are real keys here.
- If the game needs a step the player must take inside the game before the layout works, put one sentence in `setup_note`. Keep it short. Most profiles should not need one.

Shipped profiles are copied into the user's config folder the first time they are applied. A user who has already applied your profile will not see later edits until they press "Revert to shipped", so get the layout right before it ships.

## Reporting a bug

Say which pad you have, which distro, and whether the session is X11 or Wayland. Attach the daemon log from `~/.local/state/opentartarus/opentartarus.log`. If the window shows a red banner, quote it word for word.
