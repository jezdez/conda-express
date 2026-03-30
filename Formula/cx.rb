class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "0.5.1"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "f527565ec0b85953cec1286d91ebf5a9022694272a9fc8a7e352acfd5e55a276"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "0e2375aba4e269f10a9b0ef578730e75f149cb54d2b81e180d2dfae63b6972fa"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "9ca7d727107b13734b431e009e6480fca227c334fe31ef30932d5e94afac8d2f"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "cc1361f6ee8d5f6e05fa3ec0c4b0f2db22e18c740cb19a5cfba70245a57bdf1a"
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
