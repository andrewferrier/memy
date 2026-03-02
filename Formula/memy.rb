# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.18.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.18.0/memy-macos-aarch64.tar.gz"
      sha256 "829631e40c871f8b37443b7e4991660dac0da9f24dfa7e386bbcd5f532e60c55"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.18.0/memy-macos-x86_64.tar.gz"
      sha256 "43bdf88313defb3c450279d6409f187df8cb7ab9de3e41e449092311563dc04a"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.18.0/memy-linux-aarch64.tar.gz"
      sha256 "6f2ea8baa41d96db631b03f9f9c810a69938e7899b7db852e975f42cf4447adb"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.18.0/memy-linux-x86_64.tar.gz"
      sha256 "1f503d4e40911f27bba5907e6aec524497be5a1ea73047022c31634cdc455c39"
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
