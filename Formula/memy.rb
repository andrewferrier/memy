# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "hb2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-macos-aarch64"
      sha256 "2d3d925e1a1a0d8fb33f9c468367486239ae6ed247c6f9afee00d48dd6652db2"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-macos-x86_64"
      sha256 "dcb43db0974c307ab1ad02f1295ce4b8324b4cc4deb31d9b5eb4481e99d24348"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-linux-aarch64"
      sha256 "70e0b589b90aa362189a98ea335da62ffa722ad16e9c57a15b434a20c532f309"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb2/memy-linux-x86_64"
      sha256 "74a04e2ffa7193902b9db16fe145bfa63c3b6fdf1421cca957dc88adeae16df5"
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
