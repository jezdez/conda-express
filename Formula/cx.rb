class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  version "26.5.2.post7"
  license "BSD-3-Clause"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "1db28ed16f888843fe9aae7045930726d874860e7fa73a6252d8513b14a06a0d"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "450c24cec18f32f9917fddecb89567820a0650796625391c9d0f4a7a21bd133e"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "28edb0f12bb5739b2643d3a36cac3d7e19d6a47e9e88e14ee2e2d2e093937993"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "131536a0d5e3b51443c590bb9f24784d44bdf52b9ea268026f61de43362bd37b"
    end
  end

  def install
    binary = Dir["cx-*"].first || "cx"
    bin.install binary => "cx"
  end

  test do
    assert_predicate bin/"cx", :executable?
  end
end
