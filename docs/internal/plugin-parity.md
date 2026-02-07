# Plugin Parity Internal Policy

This document is internal engineering policy.
Do not copy this process section into `PLUGINS.md`.

## Default Rule

If bot supports a Core/TG/TT capability, plugin runtime should support it too.

## Allowed Exceptions

Use exception only when mapping is currently unsafe or impossible:

- security-sensitive capability
- secret-handling risk
- non-deterministic behavior not acceptable for plugin runtime
- upstream SDK limitation

## Exception Record Format

Each exception must be recorded in this file with:

- capability name
- scope (`core`, `tg`, `tt`)
- reason
- owner
- date
- re-evaluation trigger

Template:

```text
- capability: <name>
  scope: <core|tg|tt>
  reason: <why mapping is blocked>
  owner: <github handle or team>
  date: <YYYY-MM-DD>
  revisit_when: <condition>
```

## Review Rule

Any Core/TG/TT feature PR must include one of:

- plugin mapping implementation, or
- exception record update in this file.
