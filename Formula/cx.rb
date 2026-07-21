class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  version "26.5.2.post6"
  license "BSD-3-Clause"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "881fd42981f091714179daa441ae4e2b5ac78617f1e22fdcf79520770241146c"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "7edbe500e3a3dae9ed34670290f43e236b835651a6195548459f55f991921d36"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "5e82bf7aa2e32bfb349ea00d056ec9523e2bff953c181566910e1cad28c07c4e"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "6dbf3b8728a4ad8d9d228f8af60c96592cb2e69733b2061d2e3f587de62ffc9e"
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
