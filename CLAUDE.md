# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

This is a single-binary Cargo project (no workspace, no test suite yet).

- Build: `cargo build`
- Run: `cargo run`
- Type/borrow-check without building a binary (fastest iteration loop): `cargo check`
- Lint: `cargo clippy`
- Fast playtest (3 waves instead of the full 20): `cargo run -- --test`
- Force specific towers into the draft queue, one per round, for testing a tower/item combo without RNG: `cargo run -- --towers=laser,sprayer,soul_harvester` (names are case-insensitive, spaces or underscores both work, e.g. `soul_harvester` or `soul harvester`)

There's no way to drive the game headlessly — verifying gameplay behavior means running the binary and either playing manually or reading the code path closely, since Bevy pops up a real window.

## Architecture

Bevy 0.16 ECS tower-defense-roguelike. A run is: pick a tower from a 3-option draft, place it, survive a wave, spend money in an item shop, repeat. `src/main.rs` is the composition root — it inserts all starting resources, registers events, and wires every system into `Update` (plus a couple of `Startup`/`OnEnter` schedules); reading it top to bottom is the fastest way to see how everything fits together.

### Run lifecycle: `GameState`, not events

`game.rs` defines `GameState { Playing, Restarting }`. **Every** run-start system — resetting stats/HP/money, reactivating the draft and shop, respawning the path — lives in `OnEnter(GameState::Playing)`, and that schedule fires identically on the very first boot and on every restart (pressing `R` just bounces `Playing → Restarting → Playing`; `Restarting` exists solely to force Bevy to re-run `OnEnter`, since it won't re-fire `OnEnter` for a same-value transition). There used to be a `GameRestartEvent` fired from two separate places (`Startup` for boot, a keypress handler for restart) and it caused the same "shop is empty" class of bug three times before being replaced with this single-path design — don't reintroduce a second entry point into run-start logic.

Within `Update`, four phases run in a fixed chain via the `GamePhase` system set (`ResetTemporaries → TemporaryStatEffects → TemporaryTowerEffects → Gameplay`), gated by `game_is_running` (not paused, not won). Anything that needs a fully-computed per-tower temporary stat before it fires belongs in `Gameplay`.

### Kinds: `&'static Definition` wrappers, and why they must be `static`

`TowerKind`, `ItemKind`, and `SpellKind` all follow the same shape: a `Copy` wrapper around `&'static XDefinition`, with `PartialEq`/`Hash` implemented via **pointer identity** (`std::ptr::eq`), not by comparing contents. This means every `TOWER_X`/`ITEM`/`KIND` constant **must** be declared `pub static`, never `pub const`. A `const` gets re-inlined at each use site and can be promoted into a *different* anonymous static per call site, silently breaking identity comparisons (e.g. `*kind == laser::KIND` failing even though it's "the same" tower) — this has been the root cause of multiple real bugs in this codebase (an item failing to unlock because its gating tower's `KIND` was `const`). If you add a new tower/item/spell, copy the `static` pattern from an existing one.

`TowerDefinition` and `ItemDefinition` are built via constructor + `with_*` builder methods rather than struct literals:
- `TowerDefinition::new_attacking(name, range, cooldown, damage_formula, base_color, base, barrel, projectile_speed, angular_speed)` for towers that aim and fire on their own; `TowerDefinition::new_utility(name, range, base_color, base, barrel)` for towers that don't (auras, generators, click-to-trigger). Attacking towers require projectile speed and angular speed up front by design, so a real attacking tower can't be defined without them.
- `ItemDefinition::new(name, effects, cost, icon_color)`, then `.with_description(..)`, `.with_tags(..)`, `.with_max_purchases(n)` as needed.

### Temporary stats: one reset/apply convention, no exceptions

`components.rs` defines a `TemporaryStat { flat, multiplier }` (with `.reset()` and `.apply(base)`) and macro-generates per-entity components from it: `TemporaryAttackSpeed`, `TemporaryDamageBonus`, `TemporaryEnemySpeed`, `TemporaryRange`, `TemporaryProjectiles`, `TemporarySpread`. All of them follow the same lifecycle: a generic `reset_temporary_*` system zeroes them (most in `GamePhase::ResetTemporaries`; `TemporaryEnemySpeed` is reset inline mid-`Gameplay`, right before `move_enemies`, so aura systems like Tree's slow can run in between), and whichever tower/item system cares about that stat fully recomputes it from its own persistent state in `GamePhase::TemporaryTowerEffects`. Never mutate one of these components outside that cycle (e.g. don't set-and-forget in an `Added<TowerKind>` attach system) — the whole point is that every consumer re-derives the value every frame so multiple towers/items can layer onto the same tower without stepping on each other. Don't confuse this with `resources::Stat` (`raw_value` + `permanent_multiplier` + `temporary_boost` + `temporary_multiplier`), which is the analogous pattern for *global* player stats (`FireDamage`, `AttackSpeed`, `Loot`, etc.), reset by `reset_stat_temporaries`.

### Towers don't know about items

The generic `fire_towers`/`fire_beam_towers`/`aim_towers` systems in `towers.rs` read `Option<&TemporaryXxx>` components and fall back to the tower's static base stat when absent. A tower module attaches the `TemporaryXxx`/marker components it might need unconditionally (e.g. every Laser gets a `TemporaryRange`); an item's own systems then read/write those same components or listen for an event the tower already fires (`ShootEvent`, `EnemyKilledEvent`, `ChargeConsumedEvent`, or a tower-specific event like `SoulHarvestEvent`). The tower file never imports or references a specific item. When adding a tower-specific item upgrade, look at `lens.rs`/`soul_conduit.rs`/`extended_reach.rs` for the shape: a small `Resource(bool)` or `Resource(u32)` tracking purchase state, reset in `OnEnter(GameState::Playing)`, set from `ItemPurchasedEvent`, consumed by a system in `GamePhase::TemporaryTowerEffects` (for stat bonuses) or a plain `Update` system (for one-shot/event-driven effects) — never a change to the tower's own file.

### Shop unlocking: `item_definitions::unlock`

Every item plugin calls `unlock(app, condition, KIND)` once, where `condition` is `UnlockCondition::Always` (available from the start), `UnlockCondition::Tower(kind)` (unlocked the first time that tower is drafted), or `UnlockCondition::Item(kind)` (unlocked once another item is bought). This one function registers both the "add to pool" system and the "remove from pool + clear unlocked-flag" system (the latter into `OnEnter(GameState::Playing)`, so a locked item doesn't leak across a restart) and tracks unlock state in the shared `UnlockedItems` resource. `UnlockCondition::Always` items must be added to the pool from *within* `OnEnter(GameState::Playing)` itself (not `Update`), because `Shop::activate` — which rolls the actual 3 offers from the pool — runs in that same schedule; an `Update`-scheduled unlock would be a frame too late for the very first roll.

Note the two-layer shop model: `Shop.item_pool` is the set of items *eligible* to be offered; `Shop.offers` is the 3 actual slots shown to the player. Only `Shop::activate` (run start) and `Shop::reroll` (paid) regenerate `offers` from `item_pool` — adding something to the pool doesn't retroactively backfill an already-rolled shop.

### Charges/Conduit network

`charges.rs` implements a lobbing "charge" that hops between towers tagged `Conduit` (`tags::CONDUIT`) within range of each other, up to a jump limit, until it lands on a tower with the `ChargeConsumer` component, which fires `ChargeConsumedEvent { tower }`. Both the tag (to be a valid hop target) and the marker component (to actually consume on arrival) are required — a tower with only `ChargeConsumer` and no `Conduit` tag will never receive a charge. A tower can be `Conduit`-tagged without listening to `ChargeConsumedEvent` at all (a harmless network relay/dead-end, e.g. Tree).

### Tags

`tags.rs` macro-generates a marker component + a `TagInfo` (display name/color) per tag (`Biotic`, `Mechanical`, `Infernal`, `Conduit`). A tower/item's `.tags` list is declarative; `draft.rs`'s `place_draft_tower` is what actually inserts the corresponding marker components onto the spawned entity via `tag.insert(&mut commands)`.

### Draft vs. Shop

These are two independent offer/pick systems that look similar but serve different economies: `TowerDraft` (`draft.rs`) offers 3 towers between waves and is free; `Shop` (`shop.rs`) offers 3 purchasable items and persists through a wave. Don't conflate their reset/activate logic when working on one.
