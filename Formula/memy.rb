# typed: false
# frozen_string_literal: true

class Memy < Formula
  desc "Track and recall frequently and recently used files or directories"
  homepage "https://github.com/andrewferrier/memy"
  version "0.19.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.19.2/memy-macos-aarch64.tar.gz"
      sha256 "573a08f8e11221c9b22bc445d3950ddca60833e168cececbfeac3ad8bb801053"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.19.2/memy-macos-x86_64.tar.gz"
      sha256 "0ff72811b22b1daaee44c904aeef28d7aace238c4997eeb2f85a1c9d59176e4f"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/andrewferrier/memy/releases/download/v0.19.2/memy-linux-aarch64.tar.gz"
      sha256 "8774eb09f83426bdc110858790294ff2cbc78e51d6500dd7d452571f8a57b3fb"
    end

    on_intel do
      url "https://github.com/andrewferrier/memy/releases/download/v0.19.2/memy-linux-x86_64.tar.gz"
      sha256 "9f3a4eec4fc5ae871bcac70d91937f17bccdf87b566ebdefa328c35703942caf"
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
