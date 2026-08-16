# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v1.0.0/memy-macos-aarch64.tar.gz"
      sha256 "b70b583cbb9724d3e555b26219bf386816210d0ad92732ab0d57da7f635cdb84"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v1.0.0/memy-macos-x86_64.tar.gz"
      sha256 "73440f83bd9b1fc58a181c14c78741b6119319d37173c81041b8c1907d7038a2"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v1.0.0/memy-linux-aarch64.tar.gz"
      sha256 "5cc638c0e2705cdd5f9da33487e168996d8be5543c222693828c622bbd17fdc0"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v1.0.0/memy-linux-x86_64.tar.gz"
      sha256 "bc68fb40f77de5322523bb928dd6bf0b6bb8db4f7c500880f0fe9fdef8c12433"
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
