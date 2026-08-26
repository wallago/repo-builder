# ── CI ────────────────────────────────────────────────────────

# act's default image ships no Rust and no sqlite3; full-latest mirrors the
# real GitHub runner, so a local run predicts the hosted one. act also leaves
# USER unset, which trips scripts running under `set -u`; the image runs as
# uid 1001 (runner), which is what the hosted runner reports too.
act_flags := '-W .github/workflows/ci.yml -P ubuntu-latest=catthehacker/ubuntu:full-latest --env USER=runner --env GIT_PAGER=cat --env PAGER=cat'

# Mirror the full CI pipeline locally. Run before pushing.
[group('ci')]
ci-light:
    # {if:rust}
    @echo "▶ format"
    cargo fmt --check
    # {endif:rust}
    # {if:clippy}
    @echo "▶ clippy"
    cargo clippy --all-targets -- -D warnings
    # {endif:clippy}
    # {if:rust}
    @echo "▶ test"
    cargo nextest run
    @echo "▶ doc"
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
    # {endif:rust}
    # {if:typos}
    @echo "▶ typos"
    typos
    # {endif:typos}
    # {if:machete}
    @echo "▶ unused deps"
    cargo machete
    # {endif:machete}
    # {if:deny}
    @echo "▶ deny"
    cargo deny check
    # {endif:deny}
    # {if:audit}
    @echo "▶ audit"
    cargo audit
    # {endif:audit}
    # {if:taplo}
    @echo "▶ toml format"
    taplo fmt --check --diff
    # {endif:taplo}
    # {if:editorconfig}
    @echo "▶ file format"
    editorconfig-checker
    # {endif:editorconfig}
    # {if:committed}
    @echo "▶ commits"
    committed -vv HEAD
    # {endif:committed}
    # {if:lychee}
    @echo "▶ lychee"
    lychee --verbose --no-progress '*.md'
    # {endif:lychee}
    # {if:nix}
    @echo "▶ nix lint"
    statix check -i templates .
    deadnix --fail --exclude templates .
    @echo "▶ flake check"
    nix flake check --print-build-logs --all-systems
    # {endif:nix}
    @echo "✅ CI mirror passed"

# Run the full CI pipeline locally. Run before pushing.
[group('ci')]
ci-deep:
    act {{ act_flags }} -s GITHUB_TOKEN="$(gh auth token)"

# Run a single CI job locally. Usage: just ci-job msrv
[group('ci')]
ci-job JOB:
    act {{ act_flags }} -j {{ JOB }} -s GITHUB_TOKEN="$(gh auth token)"

# List the CI job IDs
[group('ci')]
ci-jobs:
    act -l -W .github/workflows/ci.yml
