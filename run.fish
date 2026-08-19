#!/usr/bin/env fish
#
# Launcher for aliens-vs-suburbia.
#
#   ./run.fish run_dev [-- cargo/game args...]   fastest iteration build
#   ./run.fish run_rel [-- cargo/game args...]   optimised release build
#
# run_dev mirrors the `superoptimized` job in bacon.toml: nightly toolchain,
# cranelift as the dev codegen backend, and sccache in front of rustc. Cranelift
# compiles much faster than LLVM but produces slower code, so it is dev-only.
#
# run_rel keeps sccache but drops nightly/cranelift and builds with --release,
# so the binary is actually fast at runtime.
#
# Extra arguments are appended to the cargo invocation. Anything after a `--`
# separator is passed through to the game itself, e.g.
#   ./run.fish run_dev --features map-editor
#   ./run.fish run_rel -- --some-game-flag

set -l script_dir (dirname (status --current-filename))
cd $script_dir; or exit 1

set -l mode $argv[1]
set -l extra $argv[2..-1]

# Number of codegen jobs. Override with e.g. `JOBS=4 ./run.fish run_dev`.
set -q JOBS; or set -l JOBS (nproc)

function __usage
    echo "usage: ./run.fish {run_dev|run_rel} [extra cargo args...]"
    echo
    echo "  run_dev  nightly + cranelift + sccache, dev profile (fastest build)"
    echo "  run_rel  sccache, --release (fastest runtime)"
end

function __require
    if not command -q $argv[1]
        echo "run.fish: '$argv[1]' not found on PATH." >&2
        echo "run.fish: $argv[2]" >&2
        return 1
    end
end

switch $mode
    case run_dev
        __require sccache "install it with: cargo install sccache"; or exit 1

        if not rustup toolchain list | string match -qr '^nightly-'
            echo "run.fish: no nightly toolchain installed." >&2
            echo "run.fish: install it with: rustup toolchain install nightly" >&2
            exit 1
        end

        if not rustup component list --toolchain nightly \
                | string match -qr 'rustc-codegen-cranelift.*\(installed\)'
            echo "run.fish: the cranelift codegen backend is not installed for nightly." >&2
            echo "run.fish: install it with:" >&2
            echo "run.fish:   rustup component add rustc-codegen-cranelift-preview --toolchain nightly" >&2
            exit 1
        end

        echo "==> dev build: nightly + cranelift + sccache (-j $JOBS)"
        env RUSTC_WRAPPER=sccache \
            ADV_CIV_AUTOLOAD=1 \
#            CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift \
            cargo +nightly run --timings -j $JOBS $extra #-Z codegen-backend

    case run_rel
        __require sccache "install it with: cargo install sccache"; or exit 1

        echo "==> release build: sccache (-j $JOBS)"
        env RUSTC_WRAPPER=sccache \
            cargo run --timings --release -j $JOBS $extra

    case '' -h --help help
        __usage
        test -z "$mode"; and exit 1
        exit 0

    case '*'
        echo "run.fish: unknown mode '$mode'" >&2
        echo >&2
        __usage >&2
        exit 1
end
