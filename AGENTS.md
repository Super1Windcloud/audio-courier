# Repository Guidelines

## Project Structure & Module Organization
audio-courier mixes a Vite/React client with a Rust Tauri core. `src/` contains UI code: `components/` for screens and atoms, `stores/` for Zustand state, `lib/` for audio/LLM adapters, and `utils/` plus `hooks/` for shared logic. `public/` hosts static assets. Native code lives under `src-tauri/` (Rust sources in `src/`, Tauri config, and helper scripts plus Vosk models). Packaging artifacts drop into `dist/`, while automation assets stay in `.github/`. Keep sensitive values in the root `.env` and the Tauri-specific `src-tauri/.env`.

## Build, Test, and Development Commands
Use `pnpm i` once to install JavaScript dependencies, then `pnpm dev` for the pure web preview or `pnpm td` for a full Tauri dev session (runs Vite + Rust watcher). Ship-ready builds use `pnpm tb` (bundled) or `pnpm dry:tb` (native build without installers). `pnpm release` runs the web build before bundling; `pnpm clean` or `just clean` clears Cargo artifacts. Lint and formatting run via `pnpm lint` (ESLint) and `pnpm check`/`pnpm format:js` (Biome) plus `pnpm format:rs`/`cargo fmt` for Rust. Run `src-tauri/setup_vosk.ps1 -Download` to fetch ASR models, and regenerate icons with `pnpm generate:icons`.

## Coding Style & Naming Conventions
TypeScript code targets ES2020 modules and React 19. Prefer functional components, `PascalCase` filenames for components, and `useX` prefixes for hooks. Shared utilities in `lib/` and `utils/` should export camelCase functions. Tailwind 4 classes drive styling; keep custom CSS scoped to `App.css`. Always run `pnpm check` before committing so Biome applies repository-wide formatting, and rely on ESLint to keep React Hook rules consistent.

## Testing Guidelines
Automated tests are not yet wired into package scripts, so treat every change as test debt. For Rust modules, colocate `#[cfg(test)]` blocks inside the relevant file and run `cargo test` from `src-tauri`. For frontend code, add `*.spec.tsx` files beside the component once a Vitest setup lands; until then, perform manual smoke tests via `pnpm td`, covering audio capture, STT streaming, and shortcut flows. Document any manual test matrix in the pull request.

## Commit & Pull Request Guidelines
Recent history favors short imperative messages such as `update`. Continue that style, ideally adding a lightweight scope (e.g., `fix: update recorder gain`). Each PR should include: a concise summary, linked issue or task ID, screenshots/GIFs for UI changes, notes on manual or automated tests, and mention of relevant `.env` additions. Rebase before opening the PR to avoid merge commits and ensure `pnpm lint`, formatting commands, and `pnpm td` finish cleanly in CI.

## Security & Configuration Tips
Never commit populated `.env` files or downloaded Vosk models; they stay ignored. When debugging audio, rotate temporary `.wav` captures out of `src-tauri/` after use. Review `tauri.conf.json` before enabling new capabilities so the window allowlist stays tight, and document any new permissions in your PR description for downstream packagers.
