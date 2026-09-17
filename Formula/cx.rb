class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  version "26.7.2.post1"
  license "BSD-3-Clause"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "c7ceaad631de30473ac3578a0dbcf5b26e4536bdbeb101df07040abc676a56d4"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "69432168fbc7eadaffc0441c762003f46e2fc78a8f288a18b738e7e82d78aeed"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "8c3adf6c8c95a968708dcb2f393b77d06472fa212174ad4625104dea234b8c66"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "0ab20dd544905264f5f3ecef51c280feba430720bb60e21c73fc526e87b60427"
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
