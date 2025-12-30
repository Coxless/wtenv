# Installation Guide

## Requirements

- Git 2.5+ (for worktree support)
- Rust 1.92+ (for building from source)

## Installation Methods

### From Binary (Recommended)

1. Download the appropriate binary from [Releases](https://github.com/Coxless/wtenv/releases)
2. Extract and place in your PATH

#### Linux

```bash
curl -L https://github.com/Coxless/wtenv/releases/latest/download/wtenv-linux-x64 -o wtenv
chmod +x wtenv
sudo mv wtenv /usr/local/bin/
```

#### macOS

```bash
# Intel Mac
curl -L https://github.com/Coxless/wtenv/releases/latest/download/wtenv-macos-x64 -o wtenv

# Apple Silicon Mac
curl -L https://github.com/Coxless/wtenv/releases/latest/download/wtenv-macos-arm64 -o wtenv

chmod +x wtenv
sudo mv wtenv /usr/local/bin/
```

#### Windows

1. Download `wtenv-windows-x64.exe` from Releases
2. Rename to `wtenv.exe`
3. Add to your PATH

### From Source

```bash
# Clone the repository
git clone https://github.com/Coxless/wtenv.git
cd wtenv

# Build and install
cargo install --path .
```

### Using Cargo

```bash
cargo install wtenv
```

## Verification

```bash
wtenv --version
```

## Shell Completion (Optional)

wtenv supports shell completion generation via clap.

### Bash

```bash
# Add to ~/.bashrc
eval "$(wtenv completions bash)"
```

### Zsh

```bash
# Add to ~/.zshrc
eval "$(wtenv completions zsh)"
```

### Fish

```bash
# Add to ~/.config/fish/config.fish
wtenv completions fish | source
```

### PowerShell

```powershell
# Add to your PowerShell profile
Invoke-Expression (&wtenv completions powershell)
```

## Troubleshooting

### "command not found"

Ensure the installation directory is in your PATH:

```bash
echo $PATH
```

### Permission denied

Make sure the binary has execute permissions:

```bash
chmod +x /path/to/wtenv
```

### Git worktree errors

Ensure you have Git 2.5 or later:

```bash
git --version
```

## Updating

### Binary

Download the latest release and replace the existing binary.

### From Source

```bash
cd wtenv
git pull
cargo install --path . --force
```

### Cargo

```bash
cargo install wtenv --force
```
