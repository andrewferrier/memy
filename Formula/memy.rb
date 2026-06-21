# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.21.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.21.0/memy-macos-aarch64.tar.gz"
      sha256 "27c52a51d79e5fbbb60dd9d4959a781b2e1397f06d618ccd55d97a4ad7aba3c4"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.21.0/memy-macos-x86_64.tar.gz"
      sha256 "d34dcff09e74435d0a0c64d85f0c82e5305aaa7ac5bc3109ec77e638d7697623"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.21.0/memy-linux-aarch64.tar.gz"
      sha256 "d434f7a6066b3e888aa915144a765646595e8c81f221c172cf700070d481f6ff"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.21.0/memy-linux-x86_64.tar.gz"
      sha256 "5c004e290525536a25951aae626fdc5fc34b4dfefb0c3f5e75cb66ace6351d55"
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
