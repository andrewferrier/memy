# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "hb1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v#{version}/memy-macos-aarch64"
      sha256 "96ef15018d9f9d5acf6ad86a9ac634a310dbdc1eaf01130e7f50a3d27a431f4c"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v#{version}/memy-macos-x86_64"
      sha256 "c942391c852c78991f7887e78f717cad329964f4fd810c8ddbc6f21868e4bc12"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v#{version}/memy-linux-aarch64"
      sha256 "349af5c453d78dc36baad4d69d4439f425a2de5ed175187adfe33479fe4b109e"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v#{version}/memy-linux-x86_64"
      sha256 "00d600ee5d57de07e2bc0bd5a5dc171518ce70a051ec8377af057e842ece8e04"
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
