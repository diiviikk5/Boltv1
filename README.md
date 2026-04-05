# bolt

Windows-first, zero-bloat CLI game launcher.

## Global install (Windows)

From source:

```powershell
cargo install --path .
```

From GitHub:

```powershell
cargo install --git https://github.com/diiviikk5/Boltv1 bolt
```

After install, open a new terminal and run:

```powershell
bolt --help
```

If `bolt` is not found, add Cargo bin to `PATH`:

```powershell
$cargoBin = "$env:USERPROFILE\.cargo\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$cargoBin", "User")
```

## Core commands

```powershell
bolt add
bolt add "D:\Games\MyGame\game.exe" --name "My Game"
bolt scan "D:\Games"
bolt list
bolt launch "my game"
bolt import all
bolt config "my game" --show
bolt export "my game"
```

## Command aliases

- `bolt a` -> `bolt add`
- `bolt s` -> `bolt scan`
- `bolt ls` -> `bolt list`
- `bolt run` -> `bolt launch`
- `bolt sync` -> `bolt import`
- `bolt cfg` -> `bolt config`
- `bolt x` -> `bolt export`
