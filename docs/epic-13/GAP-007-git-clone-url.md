# GAP-007 — Fix placeholder git clone URL in quickstart

## Problem

`docs/quickstart.md` instructs users to clone from a placeholder URL:

```
git clone https://github.com/your-org/zealot.git
```

The actual repository is at `git@github.com:AlexanderFarrell/zealot` (or the HTTPS
equivalent). A user following the quickstart literally would get a 404.

The same placeholder also appears in the architecture / developer guide. Both should
reference the real repository URL.

## Fix

Replace `https://github.com/your-org/zealot.git` with the actual repository URL.

Since this is a personal project, the HTTPS clone URL would be:
`https://github.com/AlexanderFarrell/zealot.git`

## Files to change

- `docs/quickstart.md` — clone URL in the Installation section
- `docs/architecture.md` — any other placeholder clone URL references
