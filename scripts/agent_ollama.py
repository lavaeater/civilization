#!/usr/bin/env python3
"""Drive a full Advanced Civilization game with a local Ollama model as the brain.

Three processes:

    ollama serve                       # model server on :11434 (often already up)
    AGENT_FACTIONS=all cargo run       # the game, all players agent-controlled
    python3 scripts/agent_ollama.py    # this driver

This is agent_autoplay.py with one thing swapped: instead of a hardcoded PRIORITY
table, each decision is handed to a local LLM (qwen3:4b by default). The game only
ever offers *legal* moves, so the model can only ever pick a legal one; if the model
errors or returns junk we fall back to the conservative heuristic, so a game never
stalls on a bad completion. See docs/agent-api-design.md.

Picking a move is basically multiple-choice, so a small model is plenty and much
faster on CPU. Swap the model with the OLLAMA_MODEL env var, e.g.:

    OLLAMA_MODEL=qwen3:1.7b python3 scripts/agent_ollama.py   # snappier on CPU

(also: OLLAMA_HOST to point at another machine's Ollama, e.g. a GPU box.)

Each faction plays with its own personality and randomness — see the PERSONAS
table below; edit it to taste. See docs/models-for-agent-play.md.
"""
import json
import os
import time
import urllib.error
import urllib.request

GAME = "http://127.0.0.1:7878"
OLLAMA = os.environ.get("OLLAMA_HOST", "http://127.0.0.1:11434")
MODEL = os.environ.get("OLLAMA_MODEL", "qwen3:4b")

# Fallback heuristic (used when the model is unreachable or unparseable).
# NOTE: keep Movement/ShipFerry ABOVE EndMovement, or the fallback ends the
# movement phase without ever moving — agents sit on their start areas and
# population never spreads (so it never grows). The fallback should still play a
# spreading game; the model just plays a smarter one.
PRIORITY = {
    "CityConstruction": 90, "EliminateCity": 80, "AcquireCard": 75,
    "DoneAcquiringCards": 70, "EndCityConstruction": 60,
    "Movement": 40, "ShipFerry": 35,
    "EndMovement": 20, "AttackArea": 5, "AttackCity": 5,
}

SYSTEM = (
    "You are a player in the board game Advanced Civilization. "
    "On your turn you are given the current phase and a numbered list of LEGAL moves. "
    "Choose exactly one by returning its index. "
    "Respond ONLY with the requested JSON."
)

# Per-faction personalities. Each faction's move decision is an independent call,
# so we can give each one its own strategy ("persona") and randomness
# ("temperature"). The persona is appended to the SYSTEM prompt and biases WHICH
# legal move kinds the model prefers (e.g. aggressive -> Movement/AttackArea,
# builder -> CityConstruction). Temperature adds behavioural variance: low =
# disciplined, high = erratic. Edit freely — keep persona text short and concrete
# so small models actually follow it. Factions not listed use DEFAULT_PERSONA.
#
# The 9 factions: Egypt, Crete, Africa, Asia, Assyria, Babylon, Illyria, Iberia, Thrace.
DEFAULT_PERSONA = (
    "Play a balanced game: expand population, build and keep cities, and end "
    "phases when nothing productive remains.",
    0.2,
)
PERSONAS = {
    "Egypt": (
        "You are an AGGRESSIVE EXPANSIONIST. Grab new territory early: favour "
        "Movement into empty areas and attacking rivals over sitting still.",
        0.5,
    ),
    "Babylon": (
        "You are a PEACEFUL CITY-BUILDER. Prioritise founding and keeping cities; "
        "avoid fights you don't need and protect your population.",
        0.2,
    ),
    "Crete": (
        "You are a CAUTIOUS NAVAL TURTLE. Consolidate, keep population safe, use "
        "ships and the sea, expand slowly, and avoid risky attacks.",
        0.15,
    ),
    "Asia": (
        "You are a MERCHANT. Rush through the action phases to reach trading; "
        "build cities (they generate trade) but avoid costly wars.",
        0.3,
    ),
    "Assyria": (
        "You are a MILITARY OPPORTUNIST. Balance expansion and building, but "
        "strike hard when you are clearly ahead of a neighbour.",
        0.45,
    ),
    "Africa": (
        "You are a WIDE EXPANSIONIST. Spread population across as many areas as "
        "possible; prefer Movement and expansion over concentrating force.",
        0.4,
    ),
    "Illyria": (
        "You are a CHAOTIC RAIDER. Be unpredictable and opportunistic: raid and "
        "attack when you can, and don't overthink consolidation.",
        0.75,
    ),
    "Iberia": (
        "You are a DEFENSIVE COASTAL BUILDER. Hold coastal areas, build cities "
        "there, and fight only to defend what you hold.",
        0.2,
    ),
    "Thrace": (
        "You are a STEADY GENERALIST. Adapt to the board: expand when safe, build "
        "cities when you can, and fight only when it clearly pays off.",
        0.3,
    ),
}


def persona_for(faction):
    """(persona text, temperature) for a faction, falling back to the default."""
    return PERSONAS.get(faction, DEFAULT_PERSONA)


def ollama_alive():
    """True if the Ollama server answers and has the configured model loaded."""
    try:
        with urllib.request.urlopen(OLLAMA + "/api/tags", timeout=3) as r:
            tags = json.load(r)
    except (urllib.error.URLError, OSError, json.JSONDecodeError):
        return False, []
    names = [m.get("name", "") for m in tags.get("models", [])]
    return True, names


def get(path):
    try:
        with urllib.request.urlopen(GAME + path, timeout=3) as r:
            return json.load(r)
    except (urllib.error.URLError, OSError, json.JSONDecodeError):
        return None


def post(path, body):
    req = urllib.request.Request(
        GAME + path, data=json.dumps(body).encode(), method="POST",
        headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=3) as r:
            return json.load(r)
    except urllib.error.HTTPError as e:
        return {"ok": False, "error": f"HTTP {e.code}"}
    except (urllib.error.URLError, OSError, json.JSONDecodeError):
        return None


def heuristic(moves):
    return max(moves, key=lambda m: PRIORITY.get(m.get("kind"), 1)) if moves else None


def ask_ollama(phase, faction, moves):
    """Return the chosen move index, or None to fall back to the heuristic."""
    valid = {m["index"] for m in moves}
    persona, temperature = persona_for(faction)
    listing = "\n".join(
        f'  index {m["index"]}: {m["kind"]} '
        f'{ {k: v for k, v in m.items() if k not in ("index", "kind")} }'
        for m in moves
    )
    prompt = (
        f"Phase: {phase}\nYou are playing faction: {faction}\n"
        f"Legal moves:\n{listing}\n\n"
        "Reply with JSON {\"index\": <one of the indices above>}."
    )
    body = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": f"{SYSTEM} {persona}"},
            {"role": "user", "content": prompt},
        ],
        "stream": False,
        "think": False,  # qwen3 is a thinking model; disable for snappier turns
        "keep_alive": -1,  # keep the model loaded in memory so it never reloads
        "options": {"temperature": temperature},
        # Constrained decoding: the model is forced to emit this shape.
        "format": {
            "type": "object",
            "properties": {"index": {"type": "integer"}},
            "required": ["index"],
        },
    }
    req = urllib.request.Request(
        OLLAMA + "/api/chat", data=json.dumps(body).encode(), method="POST",
        headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            resp = json.load(r)
        idx = json.loads(resp["message"]["content"]).get("index")
        return idx if idx in valid else None
    except (urllib.error.URLError, OSError, KeyError, ValueError, json.JSONDecodeError):
        return None


def pick(phase, faction, moves):
    if not moves:
        return None
    idx = ask_ollama(phase, faction, moves)
    if idx is None:                       # model down / bad output -> heuristic
        choice = heuristic(moves)
        return choice, "heuristic"
    return next(m for m in moves if m["index"] == idx), "ollama"


def play():
    last_phase = None
    print(f"brain: {MODEL} @ {OLLAMA}")
    alive, names = ollama_alive()
    if not alive:
        print(f"!! WARNING: Ollama unreachable at {OLLAMA} — every turn will use the\n"
              f"!! conservative HEURISTIC, not the model. Start it with `ollama serve`.")
    elif MODEL not in names:
        print(f"!! WARNING: model {MODEL!r} not found in Ollama (have: {names}).\n"
              f"!! Turns will fall back to the heuristic. `ollama pull {MODEL}`.")
    else:
        print(f"ok: model {MODEL!r} loaded and reachable")
    brain_n, fallback_n = 0, 0
    while True:
        state = get("/state")
        if not state:
            print("waiting for game on 127.0.0.1:7878 ...")
            time.sleep(2)
            continue

        phase = state.get("phase", "?")
        players = state.get("players", [])
        if phase != last_phase:
            print(f"=== phase: {phase} ({len(players)} agent players) ===")
            last_phase = phase
        if not players:
            time.sleep(1)
            continue

        acted = False
        if phase == "Trade":
            # Trading via the LLM is a bigger task (offer/accept/settle); for now
            # opt out so the phase clears. See docs/agent-api-design.md Trade T1-T4.
            for p in players:
                td = get(f"/trade?faction={p['faction']}")
                if td and td.get("can_trade"):
                    post("/trade/stop", {"faction": p["faction"]})
                    acted = True
        else:
            for p in players:
                if not p.get("your_turn"):
                    continue
                mv = get(f"/moves?faction={p['faction']}")
                result = pick(phase, p["faction"], mv.get("moves", []) if mv else [])
                if result:
                    choice, src = result
                    if src == "ollama":
                        brain_n += 1
                    else:
                        fallback_n += 1
                        # First silent degradation is the classic "agents froze"
                        # symptom — surface it loudly the moment it starts.
                        if fallback_n == 1 and brain_n == 0:
                            print(f"!! NOTE: first decision used the heuristic, not "
                                  f"{MODEL}. If this keeps happening, the model is "
                                  f"unreachable/erroring — check `ollama serve`.")
                    r = post("/move", {"faction": p["faction"], "index": choice["index"]})
                    ok = r.get("ok") if isinstance(r, dict) else r
                    print(f"[{phase}] {p['faction']} -> {choice.get('kind')} "
                          f"(via {src}, ok={ok}) [brain {brain_n}/heur {fallback_n}]")
                    acted = True

        time.sleep(0.1 if acted else 0.6)


if __name__ == "__main__":
    try:
        play()
    except KeyboardInterrupt:
        print("\nstopped")
