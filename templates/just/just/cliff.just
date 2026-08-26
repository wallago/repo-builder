# {if:cliff}
# Regenerate CHANGELOG.md from git history
[group('release')]
changelog:
    git-cliff -o CHANGELOG.md
# {endif:cliff}
