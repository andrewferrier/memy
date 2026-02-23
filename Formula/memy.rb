# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.17.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-macos-aarch64"
      sha256 "9422cdbb81eebefa8fecaffd83cbdc774dba2867c722cb9645bfdc316c2b9715"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-macos-x86_64"
      sha256 "36a6f9fa5547836c12c98cacbbea04b9952b106cf1cf759f3c25fdba9f664548"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-linux-aarch64"
      sha256 "c7112e8b9a8a2d1a34e0ed08981cf5b5ad5458ecd3dd297f27d6f047656d257e"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-linux-x86_64"
      sha256 "10854475644c792418cf7af7f6c63931e8b3dbd4b89a62dfab9e62bbda9a68cd"
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
