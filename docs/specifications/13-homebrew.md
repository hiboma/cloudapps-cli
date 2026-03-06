# 13 - Homebrew Distribution

## Overview

The CLI is distributed via a Homebrew tap for macOS users.

## Tap Repository

Repository: `github.com/hiboma/homebrew-tap`

## Formula

File: `Formula/cloudapps.rb`

```ruby
class Cloudapps < Formula
  desc "CLI tool for Microsoft Defender for Cloud Apps REST API"
  homepage "https://github.com/hiboma/cloudapps-cli"
  version "0.1.0"

  if Hardware::CPU.arm?
    url "https://github.com/hiboma/cloudapps-cli/releases/download/v#{version}/cloudapps-aarch64-apple-darwin.tar.gz"
    sha256 "<sha256>"
  else
    url "https://github.com/hiboma/cloudapps-cli/releases/download/v#{version}/cloudapps-x86_64-apple-darwin.tar.gz"
    sha256 "<sha256>"
  end

  def install
    bin.install "cloudapps"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cloudapps --version")
  end
end
```

## Installation

```bash
brew tap hiboma/tap
brew install cloudapps
```

## Update Process

1. The release workflow builds binaries and creates a GitHub Release.
2. A post-release step updates the Homebrew formula with new version and SHA256 checksums.
3. This can be automated in the release workflow or done manually.

## Binary Packaging

- macOS binaries are packaged as `.tar.gz` archives.
- Each archive contains the `cloudapps` binary.
- Archive naming convention: `cloudapps-{target}.tar.gz`
  - `cloudapps-x86_64-apple-darwin.tar.gz`
  - `cloudapps-aarch64-apple-darwin.tar.gz`

## Notes

- The formula does not build from source; it downloads pre-built binaries.
- Linux and Windows binaries are distributed via GitHub Releases directly.
