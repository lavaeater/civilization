# Reinforcement Learning — Can we train a good AI?

Short answer: **yes, this game is unusually well set up for RL**, and most of the
hard plumbing (a programmatic environment with legal-move enumeration) already
exists via the agent API. But "just run RL and get a strong AI" hides real work —
this is a multi-phase, multiplayer, partial-information game, which is harder than
the chess/Go poster children. This doc lays out the formulation, what we already
have, the honest challenges, and a pragmatic roadmap that starts cheap.

See also: `docs/agent-api-design.md` (the env interface), `docs/ast-design.md`
(the win condition), `scripts/agent_ollama.py` (an LLM driver over the same API).

## 1. Why this game fits RL well

RL needs three things: a **state** to observe, a set of **legal actions** to
choose from, and a **reward** signal. This codebase already hands us all three:

- **Legal actions for free.** Every turn the game computes
  `AvailableMoves { moves: HashMap<usize, GameMove> }` for the active player
  (`src/civilization/game_moves/`). This is *exactly* the "action mask" RL agents
  need — the agent never has to learn which moves are legal, only which is good.
  Illegal-move handling, the bane of board-game RL, is solved by construction.
- **A programmatic env.** The embedded HTTP server (`src/agent_api/`, port 7878)
  exposes `GET /state`, `GET /moves`, `POST /move`, and the trade endpoints. A
  training loop can step the game from Python without touching Bevy internals.
- **A natural episodic reward.** The A.S.T. is a 17-space track (0–16); space 16
  is **FINISH = a win** (`AstPosition`, `AST_FINISH = 16`). Progress along the
  track is a dense, already-computed proxy for "am I doing well," and reaching
  FINISH (or being first to) is the terminal win/loss signal.
- **Self-play is built in.** `AGENT_FACTIONS=all` makes every player agent-driven;
  a single running game is a full N-player self-play arena. Self-play is the
  engine behind essentially every strong board-game RL agent.

## 2. The RL formulation

| Element | This game |
|---|---|
| **Agent** | One faction (or a shared policy controlling all of them in self-play). |
| **State / observation** | Current `GameActivity` (phase) + board: areas, populations, cities, each player's tokens/treasury/trade cards/civ cards/AST position. Today `/state` is thin; a richer obs should reuse the `save_game` serialization (`src/civilization/concepts/save_game/`), which already snapshots the whole game. |
| **Action** | An *index* into the current `AvailableMoves`. The policy outputs a score per legal move; pick (or sample) the best. This sidesteps the variable, structured action space — you never emit raw "move 3 tokens from area 12 to 47," you rank the moves the game already enumerated. |
| **Reward** | Per-step shaping: change in AST position (+ for advancing, − for slipping back per rule 33.4), maybe small bonuses for cities/cards. Terminal: large + for winning (first to FINISH), − for losing/elimination. |
| **Episode** | One full game, from `StartGame` to a winner reaching `AST_FINISH` (or a turn cap). |

The key design decision that makes this tractable: **action selection by scoring
the legal-move list**, not generating actions from scratch. The same trick the
`agent_ollama.py` "pick an index" loop uses — but instead of an LLM, a trained
neural net (or a simpler model) produces the scores.

## 3. The honest challenges

These are why it's "train an AI" not "press a button":

1. **Multi-phase actions.** A turn spans Population Expansion, Movement, City
   Construction, Trade, Calamities, Civ-card buying… Each phase has a different
   move vocabulary. Options: (a) one policy over all phases conditioned on the
   phase id, or (b) a small policy per phase. (a) is simpler to start.
2. **Multiplayer credit assignment.** With 3–8 players, the win is sparse and the
   cause of a win is spread over hundreds of moves and other players' choices.
   AST-progress shaping helps a lot here — it turns a sparse win into a dense
   "are you climbing the track" signal.
3. **Non-stationarity in self-play.** Everyone is learning at once, so the
   opponents keep changing. Standard mitigations: keep a *pool* of past policy
   snapshots and play against them, not just the current self.
4. **Hidden information (Trade).** Offers carry hidden card counts and can bluff
   (`OpenTradeOffer`). That's genuine imperfect information; for a first agent,
   the easiest move is to **fix trade to a scripted policy** (the current driver
   just opts out) and let RL learn the rest, then revisit trade later.
5. **Env speed.** RL is sample-hungry — think 10⁵–10⁷ games. Stepping over HTTP
   one move at a time, with rendering, is far too slow. You'll want a **headless,
   un-throttled** game (no window, `ai_move_delay_secs: 0.0`, no `GamePaused`) and
   ideally many games in parallel. This is the single biggest engineering lift.

## 4. A pragmatic roadmap (cheapest first)

Don't start with deep RL. Climb the ladder — each rung gives a usable AI and de-risks the next:

- **R0 — Strong heuristic baseline.** Improve the hand-written priority in
  `agent_autoplay.py` / the `StupidAi`. Free, immediate, and you *need* a baseline
  to measure any learned agent against.
- **R1 — Imitation / behaviour cloning.** Log `(state, chosen_move)` pairs from the
  heuristic (or from your own play) and train a small net to predict the move.
  Supervised, stable, no reward engineering — a good way to validate the obs
  encoding and the "score the legal moves" architecture before any RL.
- **R2 — Self-play RL (the real thing).** Wrap the agent API as a
  [Gymnasium](https://gymnasium.farama.org/) environment, then train with a
  **masked-action PPO** (e.g. `sb3-contrib`'s `MaskablePPO`, or a small custom
  loop). Reward = AST-progress shaping + terminal win/loss. Self-play via
  `AGENT_FACTIONS=all`, opponents sampled from a snapshot pool.
- **R3 — Search + learning (optional, strongest).** If you want AlphaZero-style
  strength, add MCTS over the move list guided by the learned policy/value net.
  This needs a *fast forward model* (clone game state, try a move, evaluate) —
  much easier in-process in Rust than over HTTP, so it implies exposing the step
  function natively rather than via the web server.

A realistic "reasonably good AI without a research project" target is **R1→R2**:
behaviour-clone the heuristic, then improve it with self-play PPO. That alone
typically beats the scripted baseline.

## 5. Suggested architecture (R2)

```
┌────────────────────┐   step: POST /move        ┌──────────────────────┐
│ Bevy game(s)       │◄──────────────────────────│ Python: Gym env       │
│ headless, no delay │   obs: GET /state+/moves  │  wraps the agent API  │
│ AGENT_FACTIONS=all │──────────────────────────►│  (one env per game)   │
└────────────────────┘                           └──────────┬───────────┘
                                                            │ obs/action/reward
                                                  ┌─────────▼───────────┐
                                                  │ MaskablePPO learner │
                                                  │ policy = score the  │
                                                  │ legal-move list     │
                                                  └─────────────────────┘
```

- **Observation encoding:** start simple — a fixed-length vector of per-area and
  per-player features (token counts, cities, treasury, AST position, phase
  one-hot). Reuse the `save_game` snapshot as the source of truth.
- **Action head:** for each legal move, build a small feature vector (move kind +
  its parameters, e.g. from/to area, token count) and score it; softmax over the
  legal set. This handles the variable action count cleanly.
- **Reward:** `Δ(AST position)` each turn, `+1` (or large) for winning, `−1` for
  losing/elimination. Optionally tiny shaping for first city / first trade card.

## 6. What's missing in the codebase for this

Mostly speed and observation richness — both are listed as follow-ups in
`agent-api-design.md`:

- **Headless + fast-forward mode.** A way to run with no rendering, no AI delay,
  no pause, stepping as fast as the CPU allows. (Bevy can run a minimal
  `App` without the render plugins.)
- **Richer `/state`.** Wire the `save_game` snapshot into the API so the agent
  sees full board state, not just its own areas.
- **A reset endpoint.** `POST /reset` to start a fresh game in-process, so an
  episode loop doesn't have to relaunch the binary.
- **(For R3 only) a native step/clone API** — RL search needs to try moves on a
  copied state; doing that over HTTP is impractical.

## 7. Bottom line

- **Feasible?** Yes — the legal-move API + AST reward + built-in self-play make
  this a genuinely good RL target, better set up than most hobby game projects.
- **Easy?** The R1 (imitation) and R2 (self-play PPO) rungs are very doable for a
  motivated solo dev; expect the real work to be in **env speed**, **observation
  encoding**, and **reward shaping**, not the RL algorithm itself (use an
  off-the-shelf MaskablePPO).
- **Will it beat the heuristic?** Almost certainly, once self-play gets going —
  that's the usual outcome. Whether it becomes *strong* depends on how much
  compute and tuning you pour in, and on handling Trade's hidden information.

Start at R0/R1. They're cheap, give you a baseline and a working pipeline, and
make R2 mostly a matter of swapping supervised loss for a self-play reward.
