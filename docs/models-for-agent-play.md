# Models for Agent Play

Notes on which (local) LLMs to use to drive factions through the agent API, and
why. This is about the `scripts/agent_ollama.py` path — a model picks a move each
turn over Ollama. For the "train a net instead of prompting an LLM" route, see
`docs/reinforcement-learning.md`; for the API itself, `docs/agent-api-design.md`.

## What the model is actually being asked to do

This is the single most important thing to internalise, because it changes every
recommendation: **the model is not generating moves — it is picking one from a
pre-validated list.** Each turn the game emits `AvailableMoves` (the legal moves),
the driver numbers them, and the model returns `{"index": N}` under constrained
decoding. So:

- The task is **multiple-choice**, not open-ended generation. Output is ~5 tokens.
- The game guarantees legality; the model can only ever choose a *legal* move, and
  bad/garbage output falls back to the heuristic. So a weak model degrades to
  "plays legally but mediocre," never "breaks the game."
- This means **small models are surprisingly fine**, and reasoning/size mostly
  buys you *better strategy*, not *correctness*.

Implication: pick the smallest model that plays acceptably, then only scale up if
you specifically want smarter play and can afford the latency.

## The two things you're trading off

1. **Latency** — how long each move takes. Dominated by model size and whether the
   model is kept warm in memory. On CPU this is the pain point; on GPU it mostly
   disappears.
2. **Decision quality** — how good the chosen moves are. For a pick-from-list task
   this rises slowly with size and benefits more from a good *prompt* (clear
   board summary) than from raw parameter count.

## Recommendations

### CPU-only (no GPU)

Use a small model and keep it loaded. Rough feel for the pick-a-move task:

| Model | Size | CPU latency | Notes |
|---|---|---|---|
| `llama3.2:1b`, `gemma3:1b` | 1B | very fast | Fine — it's multiple choice. Good default to *iterate* with. |
| `qwen2.5:1.5b` | 1.5B | snappy | Solid balance of speed and sanity. |
| `qwen3:1.7b` | 1.7B | snappy | Good; **keep thinking off** (see below). |
| `qwen3:4b` | 4B | sluggish | Overkill on CPU for this task; this is what felt "soooo slow." |
| `qwen2.5:0.5b` | 0.5B | instant-ish | Usually fine, occasionally makes a dumb pick — heuristic fallback covers it. |

**Recommended CPU starting point:** `qwen3:1.7b` (or `qwen2.5:1.5b`), kept warm.

```bash
ollama pull qwen3:1.7b
OLLAMA_MODEL=qwen3:1.7b OLLAMA_KEEP_ALIVE=-1 python3 scripts/agent_ollama.py
```

### With a GPU

Latency stops being the constraint, so optimise for quality. Even a modest GPU
(6–8 GB VRAM) runs a 4B at tens of tokens/sec — instant for this task.

| Model | Size | Use when |
|---|---|---|
| `qwen3:4b` | 4B | Good default on GPU — was overkill on CPU, comfortable here. |
| `qwen3:8b`, `llama3.1:8b` | 8B | Want noticeably smarter strategy and have the VRAM. |
| `qwen2.5:7b-instruct` | 7B | Strong, well-behaved instruction follower. |
| 14B+ | — | Diminishing returns for "pick a move"; only if you're chasing the best play and have the hardware. |

**Recommended GPU starting point:** `qwen3:4b`, scale to an 8B if you want better
play. Point the driver at the GPU box with `OLLAMA_HOST` if it's a separate machine:

```bash
OLLAMA_HOST=http://gpu-box:11434 OLLAMA_MODEL=qwen3:8b python3 scripts/agent_ollama.py
```

## Latency knobs (independent of model choice)

- **Keep the model loaded.** First call pays the load cost; after that it stays
  warm ~5 min by default. For a long self-play game, pin it so it never unloads:
  `OLLAMA_KEEP_ALIVE=-1 ollama serve`, and the driver also sends `keep_alive: -1`.
- **Disable "thinking" on reasoning models.** qwen3 is a thinking model; with
  thinking on it writes a chain-of-thought paragraph *before every move*, which
  dominates latency. The driver already sends `"think": false`. This is the single
  biggest qwen3 speedup.
- **Constrained output is already on.** The driver passes a JSON schema in
  `format`, so the model emits `{"index": N}` and nothing else — no prose to wait
  on, no regex scraping.
- **Low temperature.** The driver uses `temperature: 0.2`; this is a decision task,
  not creative writing, so near-greedy is fine and a touch faster/steadier.

## Reasoning ("thinking") models — worth it?

Mostly **no for CPU, maybe for GPU**. The whole point of a thinking model is
multi-step deliberation, but here the game has already done the hard part
(enumerating legal moves), so the deliberation mostly re-derives obvious things at
high latency cost. If you do want a thinking model's strategy on a GPU, enable
thinking *selectively* — e.g. only in pivotal phases (Trade, civ-card buying),
not for routine Population Expansion / Movement — rather than paying it every turn.

## Prompt quality beats model size

For this task, a clearer board description usually helps more than a bigger model.
Cheap wins, all on the driver side (`scripts/agent_ollama.py`), no extra compute:

- Summarise *why* a move matters (e.g. "AcquireCard advances your A.S.T. epoch")
  rather than dumping raw move structs.
- Include the player's high-level situation (cities, treasury, AST position) so the
  model has context for the choice.
- Keep the legal-move list compact and consistently formatted.

If you find yourself reaching for a bigger model to fix bad play, try improving the
prompt first — it's free and often enough.

## When to stop using an LLM at all

LLM-driven play is great for **watching plausible games** and **prototyping**, and
it needs zero training. But it is comparatively slow and never really *learns*. If
the goal is a genuinely strong, fast AI, the LLM is a stepping stone: it can even
generate the imitation-learning data (logged `state → chosen move` pairs) for the
RL path. See `docs/reinforcement-learning.md` (R1 → R2): a small trained net that
scores the legal-move list will be far faster than any LLM and can be improved with
self-play.

## TL;DR

- It's a **multiple-choice** task with a legality guarantee and a heuristic
  fallback — **small models are fine**; scale up only for smarter strategy.
- **CPU:** `qwen3:1.7b` / `qwen2.5:1.5b`, kept warm, thinking off.
- **GPU:** `qwen3:4b`, step up to an 8B for stronger play.
- Biggest speed wins: **keep the model loaded** and **turn thinking off**.
- Biggest quality win that isn't "bigger model": **a better prompt**.
- For a fast, strong, learning AI, graduate from the LLM to a trained net
  (`docs/reinforcement-learning.md`).
