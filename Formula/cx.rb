class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "0.3.0"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-apple-darwin"
      sha256 "3e2670a31852dd752eb5c8398d7c34156c7e0dcf14c8178c7754aa0f32a452a7"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-apple-darwin"
      sha256 "77d3d455f5f831d7e4e36aec23793bf8f838946fdf67fe257fc24cff69dd65a7"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "e764498d61b25796ce69bd518881c725b32ca6230f87285d80c5c8aff210672e"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/v#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "0723a4da73c2980fb9db9cba97159de1b0093e7bd586021c19f6b59a6cae38d3"
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
