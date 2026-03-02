# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "hb4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb4/memy-macos-aarch64"
      sha256 "5f329dc01d921821a86a59494e7c6b5e4de4abb35bb833f3ebb37f46a5cabaef"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb4/memy-macos-x86_64"
      sha256 "fc9a208a6e930f9847a984026a6e23e5bf1466f5ff534bbe5b17560c6ce7b921"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/hb4/memy-linux-aarch64"
      sha256 "e6ddb9aae9d60e1870e0fb037006d98fc5c65d5c08a51b95198b4b022a006512"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/hb4/memy-linux-x86_64"
      sha256 "5ed042314978a5ff3046ce3b220af475bbecb2529b240cbd03996b12c4a8ea4c"
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
