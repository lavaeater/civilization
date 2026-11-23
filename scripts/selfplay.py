#!/usr/bin/env python3
"""Headless all-AI self-play: run several games, report averages per faction.

One 9-player game varies too much to judge an AI change by, so this runs N
games (in parallel, each its own adv_civ_server process and directory) and
averages what the game logs say:

- `[STATE]` lines (once per player per round): cities, board tokens, A.S.T.
  space, rule-35 score.
- `[LOSS]` lines (same cadence): board tokens and cities that left the board
  since the previous round, and treasury spent, by the phase it happened in.
- `[CITY_CONSTRUCTION] <Faction> built a city` lines.

Usage:
    scripts/selfplay.py                       # 6 games x 30 rounds
    scripts/selfplay.py -g 12 -r 40 -j 6
    scripts/selfplay.py --save base.json      # keep the summary...
    scripts/selfplay.py --compare base.json   # ...and diff a later run with it
    scripts/selfplay.py --report-only target/selfplay/<run>   # re-read logs

Game directories (server.log, savegame.json) go to target/selfplay/<time>/.
Only this script's own server processes are ever stopped.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import statistics
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ANSI = re.compile(r"\x1b\[[0-9;]*m")
STATE = re.compile(
    r"\[STATE\] .* of (\w+)(?: \[(\w+)\])?: score=(\d+) .*?AST sp(-?\d+).*?"
    r"cities=(\d+).*?board_pop=(\d+).*?stock=(\d+) treasury=(\d+)"
)
ROUND = re.compile(r"\[STATE\] ─+ Round (\d+)")
LOSS = re.compile(
    r"\[LOSS\] .* of (\w+): tokens\[(.*?)\] cities\[(.*?)\] treasury\[(.*?)\](?: gained\[(.*?)\])?"
)
BUILT = re.compile(r"\[CITY_CONSTRUCTION\] (\w+) built a city")

# Phases grouped into the causes worth telling apart. Tokens spent on
# CityConstruction became a city, so they are reported but not as a loss.
CAUSES = {
    "RemoveSurplusPopulation": "surplus",
    "Conflict": "conflict",
    "ResolveCalamities": "calamity",
    "CheckCitySupportAfterRemoveSurplusPopulation": "support",
    "CheckCitySupportAfterResolveCalamities": "support",
    "CityConstruction": "to_city",
}
CAUSE_ORDER = ["surplus", "conflict", "calamity", "support", "other", "to_city"]


def parse_counts(field: str) -> dict[str, int]:
    out: dict[str, int] = defaultdict(int)
    for part in filter(None, field.split(",")):
        phase, n = part.split("=")
        out[CAUSES.get(phase, "other")] += int(n)
    return out


def parse_game(log: Path) -> dict:
    """Per-faction series and totals for one game log."""
    factions: dict[str, dict] = defaultdict(
        lambda: {
            "cities": [], "board": [], "ast": [], "score": [], "strategy": None,
            "tokens_lost": defaultdict(int), "cities_lost": defaultdict(int),
            "treasury_spent": 0, "built": 0, "gained": 0,
        }
    )
    rounds = 0
    with log.open(errors="replace") as f:
        for line in f:
            if "[" not in line:
                continue
            line = ANSI.sub("", line)
            if m := ROUND.search(line):
                # The [STATE] at the start of round n reports on round n-1.
                rounds = int(m[1]) - 1
            elif m := STATE.search(line):
                fx = factions[m[1]]
                fx["strategy"] = m[2]
                fx["score"].append(int(m[3]))
                fx["ast"].append(int(m[4]))
                fx["cities"].append(int(m[5]))
                fx["board"].append(int(m[6]))
            elif m := LOSS.search(line):
                fx = factions[m[1]]
                for cause, n in parse_counts(m[2]).items():
                    fx["tokens_lost"][cause] += n
                for cause, n in parse_counts(m[3]).items():
                    fx["cities_lost"][cause] += n
                fx["treasury_spent"] += sum(parse_counts(m[4]).values())
                fx["gained"] += sum(parse_counts(m[5] or "").values())
            elif m := BUILT.search(line):
                factions[m[1]]["built"] += 1
    return {"rounds": rounds, "factions": factions}


def mean(xs):
    return statistics.fmean(xs) if xs else 0.0


def summarize(games: list[dict]) -> dict:
    """Averages over games: per faction, and for the whole table."""
    per_faction: dict[str, dict] = {}
    names = sorted({f for g in games for f in g["factions"]})
    for name in names:
        rows = [g["factions"][name] for g in games if name in g["factions"]]
        rounds = [max(len(r["cities"]) - 1, 1) for r in rows]
        per_faction[name] = {
            "games": len(rows),
            "final_cities": mean([r["cities"][-1] for r in rows if r["cities"]]),
            "avg_cities": mean([mean(r["cities"]) for r in rows]),
            "final_board": mean([r["board"][-1] for r in rows if r["board"]]),
            "final_ast": mean([r["ast"][-1] for r in rows if r["ast"]]),
            "final_score": mean([r["score"][-1] for r in rows if r["score"]]),
            "built_per_game": mean([r["built"] for r in rows]),
            "cities_lost_per_game": {
                c: mean([r["cities_lost"].get(c, 0) for r in rows]) for c in CAUSE_ORDER
            },
            "tokens_lost_per_round": {
                c: mean([r["tokens_lost"].get(c, 0) / n for r, n in zip(rows, rounds)])
                for c in CAUSE_ORDER
            },
            "strategies": sorted({r["strategy"] for r in rows if r["strategy"]}),
        }
    rounds = [g["rounds"] for g in games]
    totals_per_round = {
        c: [
            sum(f["tokens_lost"].get(c, 0) for f in g["factions"].values()) / max(g["rounds"], 1)
            for g in games
        ]
        for c in CAUSE_ORDER
    }
    cities_final = [
        sum(f["cities"][-1] for f in g["factions"].values() if f["cities"]) for g in games
    ]
    # built + gained - lost - standing should be 0; anything else is a city that left
    # the board by a path that does not record it in the loss ledger.
    unaccounted = [
        sum(f["built"] + f["gained"] - sum(f["cities_lost"].values()) - (f["cities"][-1] if f["cities"] else 0)
            for f in g["factions"].values())
        for g in games
    ]
    return {
        "games": len(games),
        "rounds": mean(rounds),
        "unaccounted_cities_per_game": mean(unaccounted),
        "factions": per_faction,
        "table": {
            "tokens_lost_per_round": {c: mean(v) for c, v in totals_per_round.items()},
            "tokens_lost_per_round_sd": {
                c: statistics.stdev(v) if len(v) > 1 else 0.0 for c, v in totals_per_round.items()
            },
            "final_cities": mean(cities_final),
            "final_cities_sd": statistics.stdev(cities_final) if len(cities_final) > 1 else 0.0,
            "max_ast": mean(
                [max((f["ast"][-1] for f in g["factions"].values() if f["ast"]), default=0)
                 for g in games]
            ),
        },
    }


def delta(new: float, old: float | None) -> str:
    if old is None:
        return ""
    d = new - old
    return f" ({'+' if d >= 0 else ''}{d:.1f})"


def print_report(s: dict, base: dict | None) -> None:
    bf = (base or {}).get("factions", {})
    print(f"\n{s['games']} games, {s['rounds']:.0f} rounds each on average"
          " (cities, board, AST, score: at the end; avg: cities over the game)\n")
    print(f"{'':52}| tokens lost per round{'':34}| cities lost per game")
    print(f"{'faction':9} {'cities':<11} {'avg':>5} {'board':>6} {'AST':>5} {'score':>6} {'built':>6} | "
          + " ".join(f"{c:>8}" for c in CAUSE_ORDER) + " | surpl+confl+calam+suppt+other")
    for name, f in s["factions"].items():
        b = bf.get(name, {})
        lost = f["tokens_lost_per_round"]
        clost = f["cities_lost_per_game"]
        print(
            f"{name:9} {f['final_cities']:5.1f}{delta(f['final_cities'], b.get('final_cities')):6} "
            f"{f['avg_cities']:5.1f} {f['final_board']:6.1f} {f['final_ast']:5.1f} "
            f"{f['final_score']:6.0f} {f['built_per_game']:6.1f} | "
            + " ".join(f"{lost[c]:8.2f}" for c in CAUSE_ORDER)
            + " | " + "+".join(f"{clost[c]:.1f}" for c in CAUSE_ORDER[:-1])
            + (f"   {','.join(f['strategies'])}" if f["strategies"] else "")
        )
    t = s["table"]
    bt = (base or {}).get("table", {})
    print("\nwhole table, tokens lost per round (mean ± sd over games):")
    for c in CAUSE_ORDER:
        old = bt.get("tokens_lost_per_round", {}).get(c)
        print(f"  {c:9} {t['tokens_lost_per_round'][c]:6.1f} ± {t['tokens_lost_per_round_sd'][c]:4.1f}"
              f"{delta(t['tokens_lost_per_round'][c], old)}")
    print(f"  cities on the board at the end: {t['final_cities']:.1f} ± {t['final_cities_sd']:.1f}"
          f"{delta(t['final_cities'], bt.get('final_cities'))}")
    if abs(s.get("unaccounted_cities_per_game", 0)) > 0.01:
        print(f"  ⚠ cities unaccounted for (built + gained - lost - standing): "
              f"{s['unaccounted_cities_per_game']:.1f} per game; a loss path is not in the ledger")
    print(f"  furthest A.S.T. space: {t['max_ast']:.1f}{delta(t['max_ast'], bt.get('max_ast'))}")


def run_games(args, out: Path) -> None:
    binary = REPO / "target" / "debug" / "adv_civ_server"
    if not args.no_build:
        subprocess.run(["cargo", "build", "-p", "adv_civ_server"], cwd=REPO, check=True)
    pending = list(range(args.games))
    running: dict[int, tuple[subprocess.Popen, float]] = {}
    while pending or running:
        while pending and len(running) < args.parallel:
            i = pending.pop(0)
            gdir = out / f"game-{i:02}"
            gdir.mkdir(parents=True, exist_ok=True)
            env = dict(
                os.environ,
                RUST_LOG=os.environ.get("RUST_LOG", "info"),
                BEVY_ASSET_ROOT=str(REPO),
                SEATS="0",
                NUM_PLAYERS=str(args.players),
                PORT=str(args.base_port + 2 * i),
                HTTP_PORT=str(args.base_port + 2 * i + 1),
                CLIENT_DIR="/nonexistent",
                EXIT_AFTER_ROUND=str(args.rounds),
                EXIT_AFTER_GAME_OVER_SECS="1",
            )
            with (gdir / "server.log").open("w") as log:
                proc = subprocess.Popen([binary], cwd=gdir, env=env, stdout=log,
                                        stderr=subprocess.STDOUT)
            running[i] = (proc, time.monotonic())
            print(f"game {i}: started (pid {proc.pid})", flush=True)
        time.sleep(2)
        for i, (proc, started) in list(running.items()):
            code = proc.poll()
            if code is None and time.monotonic() - started > args.timeout:
                print(f"game {i}: timed out after {args.timeout}s, stopping it", flush=True)
                proc.terminate()
                proc.wait()
                code = "timeout"
            if code is not None:
                print(f"game {i}: finished ({code}) in {time.monotonic() - started:.0f}s",
                      flush=True)
                del running[i]


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("-g", "--games", type=int, default=6)
    p.add_argument("-r", "--rounds", type=int, default=30)
    p.add_argument("-j", "--parallel", type=int, default=max(1, min(6, (os.cpu_count() or 2) // 2)))
    p.add_argument("-p", "--players", type=int, default=9)
    p.add_argument("--base-port", type=int, default=6100)
    p.add_argument("--timeout", type=int, default=1800, help="seconds per game")
    p.add_argument("--no-build", action="store_true")
    p.add_argument("--out", type=Path)
    p.add_argument("--save", type=Path, help="write the summary as JSON")
    p.add_argument("--compare", type=Path, help="a --save file to show differences against")
    p.add_argument("--report-only", type=Path, metavar="RUN_DIR",
                   help="skip playing; summarize the logs already in RUN_DIR")
    args = p.parse_args()

    if args.report_only:
        out = args.report_only
    else:
        out = args.out or REPO / "target" / "selfplay" / time.strftime("%Y%m%d-%H%M%S")
        out.mkdir(parents=True, exist_ok=True)
        run_games(args, out)

    games = [parse_game(log) for log in sorted(out.glob("game-*/server.log"))]
    games = [g for g in games if g["rounds"] > 0]
    if not games:
        print(f"no game logs with rounds under {out}", file=sys.stderr)
        return 1
    summary = summarize(games)
    base = json.loads(args.compare.read_text()) if args.compare else None
    print_report(summary, base)
    (out / "summary.json").write_text(json.dumps(summary, indent=1))
    if args.save:
        args.save.write_text(json.dumps(summary, indent=1))
    print(f"\nlogs and summary.json: {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
