# Creating a Release

This project uses GitHub Actions to automatically build binaries for multiple platforms.

## How to Create a Release

1. **Update version in Cargo.toml** (if needed):
   ```toml
   [package]
   version = "0.2.0"
   ```

2. **Commit your changes**:
   ```bash
   git add .
   git commit -m "chore: bump version to 0.2.0"
   git push
   ```

3. **Create and push a tag**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **GitHub Actions will automatically**:
   - Build binaries for Linux, Windows, and macOS (Intel & Apple Silicon)
   - Create a GitHub Release
   - Upload all binaries to the release
   - Generate release notes

## Manual Trigger

You can also manually trigger the release workflow from the GitHub Actions tab without creating a tag.

## Supported Platforms

The workflow builds for:
- **Linux**: x86_64-unknown-linux-gnu
- **Windows**: x86_64-pc-windows-msvc
- **macOS Intel**: x86_64-apple-darwin
- **macOS Apple Silicon**: aarch64-apple-darwin

## Binary Names

- `free-erd-linux-x86_64`
- `free-erd-windows-x86_64.exe`
- `free-erd-macos-x86_64`
- `free-erd-macos-aarch64`
