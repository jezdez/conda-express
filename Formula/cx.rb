class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  version "26.5.2.post5"
  license "BSD-3-Clause"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "ddf967aa6a37124bfd395b125294dba92cee124b6bd44429f2f1847cefcc365a"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "9c3cb7db06719ac2ad61f27594ab610a1c1fd3ce0e3ec35b0d5a854695764749"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "01a590d125678f401fbb94c1d9e070be4218b817779c7aad73aab20a49fe739f"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "8d79d1bceb03b91344c812bf2077690d3132c14f93a537699c73042aa35b89e3"
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
