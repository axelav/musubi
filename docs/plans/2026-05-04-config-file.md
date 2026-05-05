# Config file support

## Goal

Let users put `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `MUSUBI_LINKS_DIR`, and
`MUSUBI_NOW_DIR` in a TOML file instead of (or in addition to) the shell
environment, so the CLI works in contexts where exporting env vars is awkward
(GUIs, cron jobs, fresh shells, etc).

## Design decisions

- **Location**: `$XDG_CONFIG_HOME/musubi/config.toml`, falling back to
  `~/.config/musubi/config.toml`. Resolved via the `directories` crate.
  Override with `MUSUBI_CONFIG=/path/to/file.toml` for testing and
  non-standard setups.
- **Format**: TOML, all keys optional.
  ```toml
  anthropic_api_key = "sk-ant-..."
  openai_api_key    = "sk-..."
  links_dir         = "~/links"
  now_dir           = "~/notes/now"
  ```
- **Precedence per field** (high → low): CLI flag → env var → config file →
  built-in default. Resolved field-by-field — env can supply the key while the
  file supplies the dir.
- **Missing file**: not an error. Empty config + env-only behavior.
- **Malformed file**: hard error with the underlying parse error. Better to
  fail loudly than silently fall back.
- **Tilde expansion**: `~` and `~/...` in path values are expanded against
  `$HOME`. Other env-var interpolation is out of scope.
- **File permissions**: read regardless of mode. Emit a warning to stderr if
  the file is world- or group-readable AND contains an api key. Don't chmod
  the user's file for them.
- **API surface**: rename `Config::from_env` → `Config::load`. No shim — this
  is pre-1.0 and the only caller is `main.rs`.

## Tasks

1. Add deps: `toml = "0.8"`, `directories = "5"`, `shellexpand = "3"`.
2. Add `FileConfig` struct (serde Deserialize) in `src/config.rs`.
3. Implement file resolution: `MUSUBI_CONFIG` override → XDG path. Read +
   parse, returning `FileConfig::default()` for missing files.
4. Rewrite `Config::load` to merge env over file, field-by-field. Tilde-expand
   paths from the file.
5. Permission warning helper (unix only; behind `#[cfg(unix)]`).
6. Update `main.rs` call site (`from_env` → `load`).
7. Tests in `tests/config_tests.rs`:
   - existing env-only tests still pass under the new name
   - file-only key picked up
   - env beats file when both set
   - file beats default when env unset
   - tilde expansion works
   - missing file is fine
   - malformed file errors
   - `MUSUBI_CONFIG` overrides location
8. Update `CLAUDE.md` "Configuration and Environment" section + main `README`
   if it mentions env vars.

## Future work

- [ ] `musubi config init` to write a starter file with comments.
- [ ] Per-provider model overrides in the file (currently hardcoded).
