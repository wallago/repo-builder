# {if:committed}
# Verify commit messages follow Conventional Commits
[group('release')]
commits base=`git rev-parse --abbrev-ref origin/HEAD`:
    committed -vv {{ `git merge-base HEAD ` + base }}..HEAD
# {endif:committed}
