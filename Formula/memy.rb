# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.20.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.20.0/memy-macos-aarch64.tar.gz"
      sha256 "c7f5dc43081d71d0fc1cca3e3b5cd56f460bb3f61fc72c452d6449801a2381f5"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.20.0/memy-macos-x86_64.tar.gz"
      sha256 "e1f8bd85e89b90916382e39b4643532377eb142c9ff4b64eed706c44631326a8"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.20.0/memy-linux-aarch64.tar.gz"
      sha256 "2c81358f0b987008f65cd9c5df0a8dab9aac6cf12593dd9e2e0c0ce2266986a7"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.20.0/memy-linux-x86_64.tar.gz"
      sha256 "fdc5ecadc5a8fa652313b8f4216f33fc4b7b6298c8425021af3f3fc6c8f1789a"
    end
  end

  def install
    bin.install "memy"
    man1.install Dir["man/*.1"]
    man5.install Dir["man/*.5"]
    doc.install "README.md"
    generate_completions_from_executable(bin/"memy", "completions")
  end

  test do
    system "#{bin}/memy", "--version"
  end
end
