# {if:codecov}
# Run tests once under coverage instrumentation
[group('dev')]
coverage pattern="":
    cargo llvm-cov --no-report nextest {{ if pattern != "" { "-E 'test(" + pattern + ")'" } else { "" } }}

# Open an HTML report from the last coverage run (run `just coverage` first)
[group('dev')]
coverage-report:
    cargo llvm-cov report --html --open

# Emit lcov.info from the last coverage run (for CI / editors)
[group('dev')]
coverage-lcov:
    cargo llvm-cov report --lcov --output-path target/lcov.info
# {endif:codecov}
