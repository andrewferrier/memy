# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "hb3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-macos-aarch64"
      sha256 "3592a14b76b2c63e235e2bc6406128dd6cb40cac04e8db3ef5aa8e11735e2088"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-macos-x86_64"
      sha256 "ee670d275cc57af1c17048f3bf3675a38f9cdaca7402441296adbcf25e12b0cd"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-linux-aarch64"
      sha256 "b811048106b509505ee63a5cfd2470026dceab85c6932be72c0ca43f68439d97"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-linux-x86_64"
      sha256 "a79a6c092df47d0319a49977dc5aa7cc3a00aa7382e1edb47191eaf376a449c4"
    end
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "memy-macos-aarch64" => "memy"
      else
        bin.install "memy-macos-x86_64" => "memy"
      end
    elsif Hardware::CPU.arm?
      bin.install "memy-linux-aarch64" => "memy"
    else
      bin.install "memy-linux-x86_64" => "memy"
    end
  end

  test do
    system "#{bin}/memy", "--version"
  end
end
