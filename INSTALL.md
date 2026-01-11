# Installation Guide

## Requirements

- Rust 1.91+ (for building from source)
- Python 3.6+ (for Claude Code hooks)

## Installation Methods

### From Binary (Recommended)

```bash
curl -L https://github.com/Coxless/ccmon/releases/latest/download/ccmon-linux-x64 -o ccmon
chmod +x ccmon
sudo mv ccmon /usr/local/bin/
```

### From Source

```bash
# Clone the repository
git clone https://github.com/Coxless/ccmon.git
cd ccmon

# Build and install
cargo install --path .
```

## Verification

```bash
ccmon --version
```

## Post-Installation Setup

Initialize Claude Code hooks in your project:

```bash
cd /path/to/your/project
ccmon init
```

To enable hooks globally for all projects:

```bash
cp .claude/settings.json ~/.claude/settings.json
```

## Troubleshooting

### "command not found"

Ensure the installation directory is in your PATH:

```bash
echo $PATH
```

For cargo install, add `~/.cargo/bin` to your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Permission denied

Make sure the binary has execute permissions:

```bash
chmod +x /path/to/ccmon
```

### Hooks not working

1. Verify Python 3 is available:
   ```bash
   python3 --version
   ```

2. Check hook files have execute permissions:
   ```bash
   ls -la .claude/hooks/
   ```

3. Verify settings.json is properly configured:
   ```bash
   cat .claude/settings.json
   ```

## Updating

### Binary

Download the latest release and replace the existing binary.

### From Source

```bash
cd ccmon
git pull
cargo install --path . --force
```
