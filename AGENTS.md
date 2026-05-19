# AI Agent Behavior Guidelines

> **Note**: `AGENTS.md` is the canonical file. `CLAUDE.md` must be a symlink to
> `AGENTS.md`. Do not edit them separately.

This repository follows the AI-Centered Development workflow.

## Core Responsibilities

1. **Workflow Adherence**:
   - NEVER skip the Execution Plan phase for non-trivial changes.
   - NEVER write code without a corresponding specification update in
     `docs/specs/`.
   - ALWAYS create a new branch from the latest `main` before starting any
     work.
   - ALWAYS go through GitHub PR review for every change, including doc-only
     changes.

2. **Branch and PR Rules**:
   - Create a fresh worktree from `main` for every task with global `ww`:
     `ww create <type>/<description>` from the target repo, or
     `ww create --repo <repo> <type>/<description>` from the workspace root.
   - Never reuse an existing feature branch.
   - Run all lint and test checks that apply before creating a PR.
   - Route PR preparation and bounded post-PR follow-up through `review-task`.

3. **Context Management**:
   - The project memory is the `docs/` directory.
   - `docs/project-plan.md` is the north star.
   - `docs/exec-plan/todo/` is the active task list.
   - `docs/design-decisions/` records architectural reasoning.

4. **Execution Rules**:
   - Plan First: before writing code, ensure a matching plan exists in
     `docs/exec-plan/todo/` unless the change is a docs-only or otherwise
     exempt task.
   - Spec First: update `docs/specs/` before modifying code.
   - Focus: if unrelated problems are found, log them under `docs/issues/`
     instead of silently expanding scope.
   - Completion: when a task is done, move its plan from `todo/` to `done/`.

## Repository Language Policy

- Internal docs must be written in English.
- Public-facing docs in this repository must be written in English.
- Code comments must be written in English.
- PR titles, PR descriptions, and commit messages must be written in English.

## Subagent Strategy

Keep the main context window clean by delegating bounded read-only exploration
and verification tasks when delegation is available.

### Delegate to subagents

- Codebase exploration and search
- Documentation research
- Parallel analysis of multiple files
- Independent verification tasks

### Keep in main context

- Final implementation decisions
- User communication
- Sequential dependent operations

### Rules

- One task per subagent
- Use clear scope boundaries
- Do not let subagents modify files unless explicitly instructed
