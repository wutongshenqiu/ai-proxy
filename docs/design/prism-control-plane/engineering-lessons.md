# Control Plane Engineering Lessons

This note captures the durable engineering lessons from the recent control-plane redesign, implementation, hardening, i18n pass, artifact cleanup, and live browser verification sessions.

It is not a changelog.
It is the set of defaults that should now be treated as the normal way to work on Prism's control plane.

## Why This Exists

The recent work uncovered the same themes repeatedly:

- legacy page structure was a poor design boundary
- temporary naming (`v2`, `old`, `greenfield`) created drag if left in place
- large pages and handlers accumulated workflow logic too quickly
- visual confidence was not enough; live browser truth mattered
- i18n broke down immediately when logic depended on English labels
- screenshots, reports, and design exports needed a clear local-only policy

These are not one-off observations.
They are now part of the control-plane working model.

## 1. Canonical Means Canonical

Once the new control plane becomes the accepted direction:

- stop using `v2`, `old`, `new`, `greenfield`, or similar transitional names in canonical code paths
- promote the new implementation into the real `web/` and handler/module paths
- keep the previous implementation only as a temporary runtime fallback, not as a naming constraint
- remove superseded names quickly so the repository does not keep teaching the wrong mental model

What proved effective:

- move the new frontend into `web/`
- rename backend modules to canonical control-plane names
- keep transitional language only in design or rollout analysis where historical context is actually needed

## 2. Design Around Product Objects, Not Legacy Pages

The control plane works better when it is organized around operator objects:

- `request session`
- `signal`
- `change`
- `provider identity`
- `route draft`
- `data source`

Do not rebuild the old page tree inside the new shell.

That means:

- `Traffic Lab` owns investigation-first request debugging
- `Provider Atlas` owns provider, auth, capability, and coverage posture
- `Route Studio` owns route authoring, simulation, and draft promotion
- `Change Studio` owns config lifecycle and rollout observation
- `Command Center` owns urgent posture and cross-workspace signal intake

If a feature request starts sounding like "bring back page X", first remap it to one of these objects or workspaces.

## 3. Prefer Internal Module Seams Over More Crates

The Rust workspace is already split enough at the crate boundary.

The recent sessions repeatedly confirmed that the higher-value move is:

- keep the current crate layout
- split `prism-server` by stable internal module seams
- use typed response/view modules instead of broad handler files
- keep async lifetimes narrow around `await`

Good examples of the right direction:

- split provider mutation/probe/read paths into dedicated modules
- split auth profile response/view logic from mutation flow logic
- extract control-plane workspace response types instead of building ad hoc JSON inline

Default rule:

- only add a new crate when the boundary is independently reusable and not just a refactoring convenience
- otherwise keep improving module structure inside the existing crate

See also:

- [rust-crate-boundary-review.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/rust-crate-boundary-review.md)

## 4. Frontend Pages Should Compose Workflows, Not Contain Them

Large workspace pages and giant controller hooks became the main frontend smell.

The sessions showed a better layering:

- page: route-level composition only
- workbench component: visual structure only
- workspace/controller hook: compose domain workflows
- small workflow hooks: own selection, action, polling, draft, or mutation logic
- lib utilities: own pure helpers and reconciliation logic

What to keep doing:

- split giant pages into workspace-specific component folders
- split giant controller hooks into `selection`, `actions`, `polling`, `draft`, `registry`, and `workbench` seams
- keep browser side effects isolated in dedicated helpers

Avoid:

- one page owning all fetches, selection, mutation, polling, and message formatting
- one hook mixing routing, CRUD, selection repair, and browser redirects

## 5. Selection State Must Reconcile Against Fresh Data

One of the most important runtime lessons was that selection state is not "set once and forget".

Whenever data can refresh, reload, or be changed by another action:

- revalidate the current selection
- fall back to the nearest valid entity
- do not leave the UI pointing at deleted or stale objects

This matters especially for:

- provider lists
- auth profiles
- config registry objects
- anything that participates in reload or device/OAuth flows

Treat selection reconciliation as a default behavior, not a patch for edge cases.

## 6. Long-Lived Workflows Must Bind to Stable Session Identity

The device-flow bug made this explicit:

- polling must bind to the identity that started the flow
- it must not drift with whatever the user selects later in the UI

General rule:

- any workflow that spans time should carry its own stable identity
- current UI selection is not a sufficient workflow key

Apply this to:

- device flow
- browser OAuth callbacks
- long-running probes
- replay/compare flows
- staged publish/watch windows

## 7. i18n Must Be Structural, Not Cosmetic

The shell locale toggle alone was not enough.

The sessions established these rules:

- never drive behavior off user-visible English strings
- use stable ids for actions, workspaces, and inspector semantics
- keep translatable copy in message catalogs
- keep technical identifiers raw and untranslated
- use locale-aware formatting for time, numbers, and durations
- maintain a pseudo-locale for layout pressure checks

If a component has logic like `label.includes("provider")`, it is already not i18n-safe.

## 8. Live Browser Truth Is a Real Acceptance Gate

Mocked tests were useful, but they were not enough to catch several real problems.

What proved necessary:

- run real browser flow scripts
- verify no failed network requests
- verify console/page errors stay clean
- verify key actions actually mutate real state
- verify locale switching in the real browser

For control-plane work, the default confidence bundle should include:

- Rust tests
- frontend lint and unit tests
- production build
- real Playwright/browser workflow verification

If the change touches control-plane UX or dashboard handlers, do not stop at unit tests.

## 9. Artifacts Are Local, Source Is Canonical

The repository now has a stronger split:

- source of truth lives in code, docs, specs, and `.pen` workspaces
- screenshots, browser reports, and PNG exports live in `artifacts/`
- `artifacts/` is local-only and not committed

Keep using:

- `artifacts/playwright/...`
- `artifacts/pencil/...`

Do not let screenshots, reports, or exports become substitute source files.

See also:

- [repo-layout-and-artifacts.md](/Users/qiufeng/work/proxy/prism/docs/playbooks/repo-layout-and-artifacts.md)

## 10. Quality Gates Need Explicit Names

The design work improved once acceptance moved from vague taste to named gates.

That same pattern applies to engineering:

- define the cutover bar explicitly
- define the artifact policy explicitly
- define the i18n bar explicitly
- define the browser verification bundle explicitly

If a rule keeps being restated in chat, it should probably become a named document, checklist, or skill instruction.

## Default Checklist for Future Control-Plane Work

Before calling a control-plane change done, verify:

1. The feature is mapped to a control-plane object/workspace, not a legacy page.
2. The code uses canonical names, not transitional ones.
3. Rust changes prefer module seams over unnecessary crate splits.
4. Frontend changes split workflow logic out of pages where needed.
5. Selection state reconciles after live data changes.
6. Long-running workflows bind to stable ids.
7. i18n uses ids/catalogs/formatters rather than raw English coupling.
8. Real browser flow verification has been run if UX or dashboard behavior changed.
9. Generated artifacts stay in `artifacts/` and out of Git.

## What Still Matters Most

The biggest continuing risks are not visual polish.

They are:

- letting new logic drift back into giant handlers or pages
- reintroducing English-coupled UI logic
- treating screenshots as proof instead of running live browser verification
- allowing transitional names to survive long enough to become permanent

Those are the patterns worth guarding against first.
