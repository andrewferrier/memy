# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.17.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.17.1/memy-macos-aarch64"
      sha256 "977e76266f59e7147008359672c158c2f92ca805ad018aac1be27c105e3a4144"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.17.1/memy-macos-x86_64"
      sha256 "576352774a701153070572601b95ea7e98268ad5d7351c88efd767f719e66450"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.17.1/memy-linux-aarch64"
      sha256 "19fc782a4b6fd2f6783f3cebf86e2a28a6c42dbf97c3614f2dd152f70ce49765"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.17.1/memy-linux-x86_64"
      sha256 "444f2a6225dc4e795561f71173b9ff0a3455709749840de18aefa2c54e1e7500"
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
