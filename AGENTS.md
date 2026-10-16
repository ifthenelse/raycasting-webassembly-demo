# raycasting-webassembly-demo: instructions for AI coding agents

## Scope
- This file is the canonical contract for coding agents (Claude Code, Codex, Copilot, Aider).
  Tool adapters (`CLAUDE.md`, `.github/copilot-instructions.md`) only point here.
- Purpose and non-goals: `intent.md`. Architecture: `docs/architecture.md`. Decisions: `docs/adr/`.

## Stack
- Node.js 26, React, Vite, TypeScript, ESLint, Prettier, Vitest.

## Commands
- Verify everything (typecheck, lint, format check, tests): `scripts/verify`
- `npm run dev` / `npm run build` / `npm test` / `npm run lint` / `npm run format`

## Verification
- Run `scripts/verify` before handing work back; it must pass. CI runs the same command and is the authoritative gate.
- Add or update tests for behaviour you change. Live or state-changing tests must be opt-in.

## Constraints
- Commit messages follow Conventional Commits (see `.gitmessage.txt`). Keep changes small and focused.
- Do not reformat unrelated code. Do not edit generated files, lockfiles by hand, or the single version source.
- Never commit secrets or `.env` files. `.env.example` holds names and safe placeholders only.

## Approval boundaries
Ask the user before: adding or upgrading a dependency, changing CI/CD or infrastructure, changing a data
schema or public contract, deleting files, touching secrets or credentials, pushing or opening a PR.

## Hand-off
Summarise what changed and why, which checks you ran with their result, and any open risk or follow-up.
