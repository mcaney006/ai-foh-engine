# Contributing

Do not add a dependency unless the standard library cannot do the job.
The last registry pull in the original sandbox 502'd; the tree is std-only
on purpose.

Rules for a mix-move change:

1. Add or update a diagnosis code in `docs/RULES.md`.
2. Cap it in `propose`.
3. Apply it in `foh-mix` if it is offline-audible.
4. Add a test that fails without the change.
5. If it would fire on a desk, it needs a verify reject path.

Do not encode OSC addresses that are not in `docs/CONTROL.md`.
