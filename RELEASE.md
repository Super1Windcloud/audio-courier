# Release

`tauri-plugin-updater` is configured to read:

`https://github.com/Super1Windcloud/audio-courier/releases/latest/download/latest.json`

Do not point the updater to `raw.githubusercontent.com/.../updater/latest.json`.
That file is only a local build artifact before upload and can also be delayed by CDN caching.

That means every release must upload:

- the platform bundle used by the updater
- its matching `.sig`
- `latest.json`

## Local release

1. Keep these versions identical:
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
2. Make sure a signing key exists at `.tauri/audio-courier_signer.key`, or set:
   - `TAURI_SIGNING_PRIVATE_KEY`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if needed
3. Export a GitHub token with `contents:write`:
   - PowerShell: `$env:GITHUB_TOKEN="..."`
4. Run:

```powershell
pnpm release
```

`license_tool` is no longer part of the main Tauri crate, so app bundles on macOS, Linux, and Windows do not include it as a nested application binary.
If you need the licensing helper, run it from the separate tools crate at `src-tauri/tools`.

This script will:

- build the signed Tauri bundle
- find the updater artifact and `.sig`
- create or update release `audio-courier-v<version>`
- merge the current platform into `latest.json`
- upload all release assets

## Useful env vars

- `RELEASE_TAG`: override the default tag `audio-courier-v<version>`
- `RELEASE_NAME`: override the release title
- `RELEASE_NOTES`: inline release notes
- `RELEASE_NOTES_FILE`: path to a release note file
- `RELEASE_TAURI_ARGS`: forwarded to `pnpm tauri build --ci`
- `RELEASE_TARGET_TRIPLE`: set platform mapping for publish-only runs
- `GITHUB_REPOSITORY`: override `owner/repo`

Example:

```powershell
$env:GITHUB_TOKEN="..."
$env:RELEASE_NOTES_FILE="release-notes.md"
$env:RELEASE_TAURI_ARGS="--target x86_64-pc-windows-msvc"
pnpm release
```

If you want to run the licensing helper from `just`, use:

```powershell
just license_tool -- generate-keypair
```

## CI release

`.github/workflows/release.yml` now triggers on tags matching:

```text
audio-courier-v*
```

Recommended flow:

```powershell
git tag audio-courier-v1.0.2
git push origin audio-courier-v1.0.2
```
