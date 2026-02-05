# Trade UI Implementation Progress

## Overview
Implementing a comprehensive trade system with open/directed offers, settlement phase, and countdown timer.

## Completed Steps

### 1. Data Structures
- **`OpenTradeOffer`** struct in `trade_components.rs:317-443` - New simplified offer with:
  - `offering_guaranteed` / `offering_hidden_count` - cards being offered
  - `wanting_guaranteed` / `wanting_hidden_count` - cards being requested
  - `accepted_by` / `accepted_by_name` - acceptance tracking
  - `creator_settlement_cards` / `acceptor_settlement_cards` - for settlement phase
  - Validation methods (`is_valid`, `can_accept`, etc.)

- **`TradePhaseState`** resource in `trade_resources.rs:35-42`:
  - `phase: TradePhase` (Trading/Settling/Ended)
  - `countdown_seconds: f32`
  - `settling_trade: Option<Entity>`
  - `human_done: bool`
  - `create_offer_modal_open: bool`

- **UI Marker Components** in `trade_components.rs:446-469`:
  - `OpenOffersListContainer`, `CreateOfferModal`, `SettlementModal`
  - `TradePhaseUiRoot`, `CreateOfferButton`, `DoneTradingButton`

### 2. Trade UI Overlay (`trade_systems.rs:465-694`)
- Header with title ("🤝 Trade Phase") and countdown timer (MM:SS format)
- Player's cards display with colored badges (commodity + calamity cards)
- Open offers section with placeholder message
- "Create Offer" and "Done Trading" buttons with hover effects

### 3. Button Handlers (`trade_systems.rs:761-817`)
- **Done Trading**: Sets `human_done` flag, removes `CanTrade` from human player
- **Create Offer**: Opens the create offer modal by setting `create_offer_modal_open = true`

### 4. Create Offer Modal (`trade_systems.rs:823-1004`)
- Modal overlay (50% width, 70% height, centered)
- Header with title and close button
- Instructions text
- Player's cards displayed as clickable buttons
- Placeholder for offer summary
- Close button handler and despawn system

### 5. Plugin Integration (`trade_plugin.rs`)
- All new systems registered in Update schedule with `run_if(in_state(GameActivity::Trade))`
- Setup/cleanup systems on OnEnter/OnExit

## Completed Steps (continued)

### 6. Settlement Modal UI (`trade_systems.rs:1006-1576`)
- **`SettlementModal`** spawns when an accepted offer needs human settlement
- **`check_for_settlement_needed`** - detects when human is involved in a settling trade
- **`spawn_settlement_modal`** - creates modal with:
  - Trade summary showing required cards (guaranteed + hidden count)
  - Clickable card buttons with selection border highlighting
  - Selected cards display with count validation
  - Confirm button (enabled only when selection is valid)
  - Cancel button (withdraws the offer)
- **`SettlementCardButton`** component tracks card type and selection state
- **`SettlementSelection`** resource tracks selected cards during settlement
- **`handle_settlement_card_selection`** - toggles card selection, updates border color
- **`update_settlement_display`** - updates selected cards text and confirm button state
- **`handle_confirm_settlement`** - validates and applies settlement to offer
- **`handle_close_settlement_modal`** - cancels trade by withdrawing offer
- **`despawn_settlement_modal`** - cleans up modal and selection resource

### New Resources Added (`trade_resources.rs:42-43`)
- `settlement_modal_open: bool` - controls settlement modal visibility
- `settling_offer_entity: Option<Entity>` - tracks which offer is being settled

## Pending Steps

### 7. Wire Up AI Trade Behavior
- AI creates `OpenTradeOffer` entities
- AI accepts/rejects offers based on card needs
- AI settles trades by selecting cards

### 8. Full Offer Creation Logic
- Card selection UI (click cards to add to offer)
- Guaranteed vs hidden card selection
- "What you want" section with card type buttons
- Validation display (show if offer is valid)
- Publish offer button

## Key Files
- `src/civilization/concepts/trade/trade_systems.rs` - Main UI and systems
- `src/civilization/concepts/trade/trade_components.rs` - Data structures
- `src/civilization/concepts/trade/trade_resources.rs` - Resources
- `src/civilization/concepts/trade/trade_plugin.rs` - Plugin registration

## Trade Rules Reference (`assets/rules/trade_rules.txt`)
- Minimum 3 cards each side
- Exactly 2 cards must be truthfully stated (guaranteed) on each side
- Remaining cards can be hidden (unspecified or even calamities)
- 5 minute time limit recommended
- Trades are final once cards change hands

## Build Status
✅ Compiles successfully (only unrelated dead_code warnings)
