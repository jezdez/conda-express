class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "26.5.2"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "2fd5f00a20c8d1cdc044e48d19d7522ad42ba0d176019cfb2be36df797948d6f
85590dd746be045378a235c86b2f2372956a0b1c0bbd179f8315491700a8cdb2
dbb56cc817792d3ec3a35c7e1ef24ffba0ba986e770b57c6b661fb8d346bd180
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "09acb95d44aaa7ffb57a09320d64563356e1d791678243e578e36482c21df146
01d3eb1f6c24a536b43e7320c9c4ec89ec50f3b2c107c9b28bb4fcf4c2a12bb0
a91ad71dac918148a2c9214d4852ffbef67ae3f66718cff6e55ced76418432ee
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "beddbd452f8453d1fe98abb422cc2cb2da0deda638c8c06259e4a8789039b16b
7ecf382e3328f5f1a934510f81a0b9ced153ea00ecfdd4a7cde1b0a674013fa9
5f2d4949c467f223a2decb50f9f420d11bf21ac112299d16768b1ea9d7e30431
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "059ff69f8a9598cfbccdd5de86c5dbad6a1e21d0f669e6ef057e15271ddbf2ee
e729c77f7740b05f1e769f904a92b8803b37c583ddc935a4e298bdb8c81ca667
3f3319f0352bc6b4e1456b84ee7e0bd745aaeee5f386d3515c4f6a706a1dcfe6
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
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
