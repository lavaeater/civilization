# bevy_enhanced_input Integration Plan

## Goal

Make the game playable from the keyboard as much as possible. Two categories:

1. **HUD shortcuts** — F1-F4 toggle the four bottom HUD cards (Player Info, Trade Cards,
   Game State, Game Activity).
2. **Phase actions** — Confirm/Cancel any currently highlighted action; navigate between
   options; increment/decrement values.  These are wired up phase-by-phase as each phase
   grows keyboard support.

---

## Step 1 – Define all `InputAction` types

In `civilization_input_plugin.rs`, derive `InputAction` for every logical action.
Keep HUD actions in one block, shared navigation actions in another.

```rust
// HUD toggles
#[derive(InputAction)] #[action_output(bool)] pub struct TogglePlayerInfoCard;
#[derive(InputAction)] #[action_output(bool)] pub struct ToggleTradeCardsCard;
#[derive(InputAction)] #[action_output(bool)] pub struct ToggleGameStateCard;
#[derive(InputAction)] #[action_output(bool)] pub struct ToggleActivityCard;

// Universal navigation (used by multiple phases)
#[derive(InputAction)] #[action_output(bool)] pub struct Confirm;
#[derive(InputAction)] #[action_output(bool)] pub struct Cancel;
#[derive(InputAction)] #[action_output(bool)] pub struct NavigateNext;
#[derive(InputAction)] #[action_output(bool)] pub struct NavigatePrev;
#[derive(InputAction)] #[action_output(bool)] pub struct IncrementValue;
#[derive(InputAction)] #[action_output(bool)] pub struct DecrementValue;
#[derive(InputAction)] #[action_output(bool)] pub struct ToggleSelected;
```

---

## Step 2 – Define `HudContext` with bindings

`HudContext` is a `#[derive(Component)]` that acts as the input context.  It is always
active while the game is in the `Playing` state (spawned at `StartGame`, never despawned).

```rust
#[derive(Component)]
pub struct HudContext;
```

Register it in `Plugin::build`:
```rust
app.add_input_context::<HudContext>();
```

Spawn the entity (Step 3) with:
```rust
commands.spawn((
    HudContext,
    actions!(HudContext[
        (Action::<TogglePlayerInfoCard>::new(), bindings![KeyCode::F1]),
        (Action::<ToggleTradeCardsCard>::new(),  bindings![KeyCode::F2]),
        (Action::<ToggleGameStateCard>::new(),   bindings![KeyCode::F3]),
        (Action::<ToggleActivityCard>::new(),    bindings![KeyCode::F4]),
    ]),
));
```

---

## Step 3 – Context lifecycle

Spawn `HudContext` once when entering `GameActivity::StartGame`.  Because it is never
despawned the shortcuts remain available for the entire play session.

```rust
fn spawn_hud_context(mut commands: Commands) {
    commands.spawn((
        HudContext,
        actions!(HudContext[
            (Action::<TogglePlayerInfoCard>::new(), bindings![KeyCode::F1]),
            (Action::<ToggleTradeCardsCard>::new(),  bindings![KeyCode::F2]),
            (Action::<ToggleGameStateCard>::new(),   bindings![KeyCode::F3]),
            (Action::<ToggleActivityCard>::new(),    bindings![KeyCode::F4]),
        ]),
    ));
}
```

Register in plugin:
```rust
app.add_systems(OnEnter(GameActivity::StartGame), spawn_hud_context);
```

---

## Step 4 – HUD toggle observers

Each F-key fires `On<Fire<ToggleXxxCard>>`.  The observer looks up the `CollapsibleContent`
marked with the matching display marker and flips `Collapsible.collapsed`.

Pattern (one per card):
```rust
fn toggle_player_info(
    _: On<Fire<TogglePlayerInfoCard>>,
    content: Query<&CollapsibleContent, With<PlayerInfoDisplay>>,
    mut collapsibles: Query<&mut Collapsible>,
) {
    if let Ok(c) = content.single()
        && let Ok(mut coll) = collapsibles.get_mut(c.parent)
    {
        coll.collapsed = !coll.collapsed;
    }
}
```

Register all four in the plugin:
```rust
app.add_observer(toggle_player_info)
   .add_observer(toggle_trade_cards)
   .add_observer(toggle_game_state)
   .add_observer(toggle_activity_card);
```

---

## Step 5 – Phase contexts (future work)

Each interactive phase gets its own context component and its own spawn/despawn system
tied to `OnEnter` / `OnExit` of the relevant `GameActivity`.

| Context              | Active during                       | Key actions                                     |
|----------------------|-------------------------------------|-------------------------------------------------|
| `TradeContext`       | `Trade`                             | Confirm offer, Cancel, Navigate cards           |
| `CalamityContext`    | `ResolveCalamities`                 | Toggle city selected, Confirm, Navigate         |
| `MovementContext`    | `Movement`                          | Select token, Next token, Confirm move, Cancel  |
| `CityBuildContext`   | `CityConstruction`                  | Confirm, Cancel                                 |
| `ShipBuildContext`   | `ShipConstruction`                  | +/− ships, Prev/Next slot, Prev/Next area, Confirm |
| `CivCardContext`     | `AcquireCivilizationCards`          | Navigate, Buy, Cancel                           |

These will be wired up incrementally.  Use `NavigateNext` / `NavigatePrev` /
`IncrementValue` / `DecrementValue` / `Confirm` / `Cancel` / `ToggleSelected` from
Step 1 so all phases share the same logical key names.

---

## Step 6 – Confirm / Cancel universally

Once all phase contexts exist, wire `Enter` → `Confirm` and `Escape` → `Cancel` in every
phase context so the player never has to reach for the mouse just to accept the default
choice or dismiss a panel.

---

## Implementation order

- [x] Step 1 — Action types defined
- [x] Step 2 — `HudContext` defined and registered
- [x] Step 3 — Lifecycle system (`spawn_hud_context`)
- [x] Step 4 — HUD toggle observers (F1-F4)
- [ ] Step 5 — Phase contexts (incremental, one at a time)
- [ ] Step 6 — Universal Confirm / Cancel
