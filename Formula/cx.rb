class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "0.2.0"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-apple-darwin"
      sha256 "c46e34c65d4b3f31b5d9cc42e1c83e97c5cbb2fa49b4de65583d2132d0e76453"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-apple-darwin"
      sha256 "bccc86b6ac7f5e74c3b4a7e887f677c49a002350727fe8b9dbc6ab3725e5c2e7"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "59b16e5d81608dd5a90d316db92d8c31b29ba2d8935509bf36963641d214978e"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "8aaed8985c7af600c1a6925c04abe246b7012d33153fbc4be5dd6f4e8d36de6e"
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
