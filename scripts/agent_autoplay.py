#!/usr/bin/env python3
"""Drive a full Advanced Civilization game through the embedded agent API.

Launch the game with agent-controlled players, e.g.:

    AGENT_FACTIONS=all cargo run

then run this script and watch it play every agent-controlled faction:

    python3 scripts/agent_autoplay.py

It is intentionally a conservative reference strategy (build cities, end phases,
skip trading) — enough to march a full self-play game forward so you can watch the
board and the A.S.T. progress. Strategy lives in PRIORITY / the Trade branch; tweak
freely. See docs/agent-api-design.md.
"""
import json
import time
import urllib.error
import urllib.request

BASE = "http://127.0.0.1:7878"

# Higher = preferred. Conservative: build cities, satisfy support, end phases,
# skip buying civ cards. Movement is ended rather than performed (calm, terminating).
PRIORITY = {
    "CityConstruction": 90,
    "EliminateCity": 80,
    "AcquireCard": 75,        # buy affordable civ cards (drives A.S.T. epochs)
    "DoneAcquiringCards": 70,  # ...then finish when nothing left to buy
    "EndCityConstruction": 60,
    "EndMovement": 50,
    "Movement": 10,
    "ShipFerry": 10,
    "AttackArea": 5,
    "AttackCity": 5,
}


def get(path):
    try:
        with urllib.request.urlopen(BASE + path, timeout=3) as r:
            return json.load(r)
    except (urllib.error.URLError, OSError, json.JSONDecodeError):
        return None


def post(path, body):
    data = json.dumps(body).encode()
    req = urllib.request.Request(
        BASE + path, data=data, method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=3) as r:
            return json.load(r)
    except urllib.error.HTTPError as e:
        return {"ok": False, "error": f"HTTP {e.code}"}
    except (urllib.error.URLError, OSError, json.JSONDecodeError):
        return None


def pick_move(moves):
    if not moves:
        return None
    return max(moves, key=lambda m: PRIORITY.get(m.get("kind"), 1))


def play():
    last_phase = None
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
            # Conservative: opt out so the trade phase clears. (Swap in /trade/offer
            # + /trade/accept + /trade/settle here for real trading.)
            for p in players:
                td = get(f"/trade?faction={p['faction']}")
                if td and td.get("can_trade"):
                    r = post("/trade/stop", {"faction": p["faction"]})
                    print(f"[Trade] {p['faction']} stops -> {r}")
                    acted = True
        else:
            for p in players:
                if not p.get("your_turn"):
                    continue
                mv = get(f"/moves?faction={p['faction']}")
                choice = pick_move(mv.get("moves", []) if mv else [])
                if choice:
                    r = post("/move", {"faction": p["faction"], "index": choice["index"]})
                    ok = r.get("ok") if isinstance(r, dict) else r
                    print(f"[{phase}] {p['faction']} -> {choice.get('kind')} ({ok})")
                    acted = True

        time.sleep(0.3 if acted else 0.6)


if __name__ == "__main__":
    try:
        play()
    except KeyboardInterrupt:
        print("\nstopped")
