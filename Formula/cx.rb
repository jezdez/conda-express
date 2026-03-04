class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "0.1.7"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-apple-darwin"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-apple-darwin"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    binary = Dir["cx-*"].first || "cx"
    bin.install binary => "cx"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cx --version")
  end
end
