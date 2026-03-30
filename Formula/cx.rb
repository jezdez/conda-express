class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "0.4.0"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-apple-darwin"
      sha256 "aac4a4a8ee4c6e944c9a7958fe405e55302d2522ce028620b6973a81a8a02c14"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-apple-darwin"
      sha256 "4e84bce9a621183b0d75f61b29aa002cc1c8f1091046104b51a51059fb242425"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "f12e75c1adc02d7a3003d33445b2e938fe43d61ef06b5d31f7431d0789b79d7d"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "e82ea40a6355f37b22f645098bcdac823da821681a4df65e7aff9ff5d88d23f5"
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
