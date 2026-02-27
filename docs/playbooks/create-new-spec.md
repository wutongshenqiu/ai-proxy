# Playbook: Create a New Spec

Step-by-step guide for creating a new specification in the AI Proxy Gateway project following SDD (Spec-Driven Development) methodology.

## Prerequisites

- Familiarity with the project (read `CLAUDE.md` and `AGENTS.md`)
- Check `docs/specs/_index.md` for existing specs and the latest SPEC ID

## Steps

### 1. Determine the Next Spec ID

Open `docs/specs/_index.md` and find the highest existing SPEC-NNN number. Your new spec will use the next sequential ID.

For example, if the latest is SPEC-007, your new spec is SPEC-008.

### 2. Create the Spec Directory

```sh
mkdir -p docs/specs/active/SPEC-NNN
```

Specs start in the `active/` directory while under development.

### 3. Copy the Templates

Copy the PRD and Technical Design templates into your new directory:

```sh
cp docs/specs/_templates/prd.md docs/specs/active/SPEC-NNN/prd.md
cp docs/specs/_templates/technical-design.md docs/specs/active/SPEC-NNN/technical-design.md
```

Optionally, if the spec requires upfront research, also copy the research template:

```sh
cp docs/specs/_templates/research.md docs/specs/active/SPEC-NNN/research.md
```

### 4. Fill in the PRD

Edit `docs/specs/active/SPEC-NNN/prd.md`:

| Section           | What to Write                                              |
|-------------------|------------------------------------------------------------|
| Spec ID / Title   | Your SPEC-NNN ID and a descriptive title                   |
| Status            | Set to `Draft` initially                                   |
| Problem Statement | What problem this solves and why it matters                 |
| Goals             | Concrete, measurable objectives                            |
| Non-Goals         | What is explicitly out of scope                            |
| User Stories      | "As a [user], I want to [action] so that [benefit]"        |
| Success Metrics   | How you will measure whether this succeeded                |
| Constraints       | Technical or business constraints                          |
| Design Decisions  | Key decisions with options considered and rationale         |

### 5. Fill in the Technical Design

Edit `docs/specs/active/SPEC-NNN/technical-design.md`:

| Section                | What to Write                                         |
|------------------------|-------------------------------------------------------|
| Overview               | High-level technical approach, reference the PRD      |
| API Design             | New/modified endpoints, request/response shapes       |
| Backend Implementation | New modules, structs, traits, key functions            |
| Configuration Changes  | New config fields, environment variables               |
| Provider Compatibility | Impact on each provider (OpenAI, Claude, Gemini)      |
| Task Breakdown         | Checklist of implementation tasks                     |
| Test Strategy          | Unit tests, integration tests, manual verification    |

### 6. Register in the Spec Index

Add a row to the **Active** section of `docs/specs/_index.md`:

```markdown
| SPEC-NNN | Your Spec Title | Active | [active/SPEC-NNN/](active/SPEC-NNN/) |
```

### 7. Update Status as Work Progresses

Update the `Status` field in both the PRD and Technical Design as the spec moves through its lifecycle:

| Status     | Meaning                                            |
|------------|----------------------------------------------------|
| Draft      | Spec is being written, not yet approved             |
| Active     | Spec is approved and implementation is in progress  |
| Completed  | Implementation matches spec, verified by tests      |
| Deprecated | Spec is no longer relevant, superseded or removed   |

### 8. Complete the Spec

When implementation is finished and all tests pass:

1. Move the directory from `active/` to `completed/`:
   ```sh
   mv docs/specs/active/SPEC-NNN docs/specs/completed/SPEC-NNN
   ```
2. Update `docs/specs/_index.md`: move the row from **Active** to **Completed** and change the status and path.
3. Set `Status` to `Completed` in both the PRD and Technical Design files.

## Spec Lifecycle Summary

```
Draft --> Active --> Completed
                 \-> Deprecated
```

- **Draft**: Spec is being written. Not ready for implementation.
- **Active**: Approved. Implementation work is in progress.
- **Completed**: Code matches spec. Tests verify correctness.
- **Deprecated**: No longer relevant. Note what supersedes it.

## Tips

- Keep specs focused. One spec per feature area.
- Reference related specs by ID (e.g., "See SPEC-002 for translation details").
- Update specs when implementation deviates from the original design.
- Use the research template for exploratory work before committing to a design.
