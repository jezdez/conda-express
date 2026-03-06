class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "0.3.1"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-apple-darwin"
      sha256 "5922de7231646935ea18fcf9323a43cb835f82e20e8a234e8adc6a331bedbedc"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-apple-darwin"
      sha256 "338cac5b1cfff27782598eff776b7fc13d8bcfc8eb2efdb454e703d2160d28fe"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "6a89680fa80af442f1fd636111a50bfd56b40c1fece0a438e1c9339e9f023656"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "cc85c5063ddbde5eacfcd1211e633e6077288d3cb03d37c772ef47fac7f75223"
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
