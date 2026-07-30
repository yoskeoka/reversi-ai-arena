# Completed plan and local issue history retention

> **Execution**: Use `/execute-task` to implement this plan. After implementation is complete, use `/review-task` to prepare and create the PR.

## Objective

Eliminate checked-out `docs/exec-plan/done/` and `docs/issues/done/` task
history so search exposes active plans and unresolved issues only. Keep
plan/implementation PRs and Git history as the audit path, without embedding
plan bodies in commit messages.

## Changes

- (MODIFY) `AGENTS.md`, active-directory READMEs, and any workflow spec/guidance that instructs a move to `done/`.
- (MODIFY) `tools/workflow-lint.sh` and its tests: preserve pre-implementation matching-plan enforcement; on closeout read the deleted plan from the base side of the diff, enforce deletion of linked local issues, and preserve external closure metadata validation.
- (DELETE) `docs/exec-plan/done/**`, `docs/issues/done/**`, and empty directories.

## Verification

Cover active-plan, compliant deletion, missing local issue deletion, and external issue metadata cases; run repository tests/lint, workflow-linter checks, `git diff --check`, and `git log --all -- docs/exec-plan` retrieval.
