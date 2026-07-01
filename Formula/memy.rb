# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.22.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.22.0/memy-macos-aarch64.tar.gz"
      sha256 "6afcae62fdb89055df30e0a6baf12e46e1bc35bcb478642c21e7d891d90062f3"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.22.0/memy-macos-x86_64.tar.gz"
      sha256 "2dd15e5722a60e71aba8575c5dc32c18838e410c1e1bcc05c030dbe72d9d3bdc"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.22.0/memy-linux-aarch64.tar.gz"
      sha256 "a507540f94022d19dc1cd4e269c3bfb74edf0b4367f33f318f6c61910ea9f347"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.22.0/memy-linux-x86_64.tar.gz"
      sha256 "f9a2b3f83ba37382ff9ec6be9270c7853b31ac606c983983c8b8528f6b977319"
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
