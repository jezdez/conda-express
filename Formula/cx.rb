class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  version "26.9.1"
  license "BSD-3-Clause"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "2a74dfa9767e14c8ab87105bd00d5eb2df23dde195f801a2953791addd4389b4"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "9a0360493817ea832a90be8477630ccc578f5d5b957f74de50beb2211d6d51d5"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "6fa6986ebcc3c7cb85694b9699183bef7d431d29272cb53dcaacd4638da9c7ac"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "90c39c54c4ff2c7098a23c72a13c7ca6a4ba2c75fa0d4c7743c31748337f8382"
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
