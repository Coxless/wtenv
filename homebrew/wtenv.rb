# Homebrew formula for wtenv
# To install:
#   brew tap USERNAME/wtenv
#   brew install wtenv
#
# Or directly:
#   brew install USERNAME/wtenv/wtenv

class Wtenv < Formula
  desc "Git worktree environment manager"
  homepage "https://github.com/USERNAME/wtenv"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/USERNAME/wtenv/releases/download/v#{version}/wtenv-macos-arm64"
      sha256 "PLACEHOLDER_SHA256_ARM64"

      def install
        bin.install "wtenv-macos-arm64" => "wtenv"
      end
    else
      url "https://github.com/USERNAME/wtenv/releases/download/v#{version}/wtenv-macos-x64"
      sha256 "PLACEHOLDER_SHA256_X64"

      def install
        bin.install "wtenv-macos-x64" => "wtenv"
      end
    end
  end

  on_linux do
    url "https://github.com/USERNAME/wtenv/releases/download/v#{version}/wtenv-linux-x64"
    sha256 "PLACEHOLDER_SHA256_LINUX"

    def install
      bin.install "wtenv-linux-x64" => "wtenv"
    end
  end

  test do
    system "#{bin}/wtenv", "--version"
  end
end
