class Cx < Formula
  desc "Lightweight single-binary conda bootstrapper powered by rattler"
  homepage "https://github.com/jezdez/conda-express"
  license "BSD-3-Clause"
  version "26.5.2.post1"

  on_macos do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-apple-darwin"
      sha256 "23f05d1cd639f3d4b21075f0504df0f062da5db9d28049093e50882ef9be091b
65dddccedf11844737e01603233d4ec39813c3c0bb96d885408ae6205177cb47
dbb56cc817792d3ec3a35c7e1ef24ffba0ba986e770b57c6b661fb8d346bd180
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
85590dd746be045378a235c86b2f2372956a0b1c0bbd179f8315491700a8cdb2
dbb56cc817792d3ec3a35c7e1ef24ffba0ba986e770b57c6b661fb8d346bd180
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-apple-darwin"
      sha256 "7fa72ba7e379910350308c7966087a0ed3c67b93ddacb85a1729b534f91b82a3
6525cb0503d604f2c8a50e1670eb4105740f2e0ac6c24706f0d07d38a7451e8f
a91ad71dac918148a2c9214d4852ffbef67ae3f66718cff6e55ced76418432ee
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
01d3eb1f6c24a536b43e7320c9c4ec89ec50f3b2c107c9b28bb4fcf4c2a12bb0
a91ad71dac918148a2c9214d4852ffbef67ae3f66718cff6e55ced76418432ee
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-aarch64-unknown-linux-gnu"
      sha256 "d12c4c215ca097267fa02edd4433fbf639c6e9b8be46519228abc0c074718025
df0facaca9545b4de5b5f25b900357b24fc52dd2e8e46367d78b4f20c8a7abf7
5f2d4949c467f223a2decb50f9f420d11bf21ac112299d16768b1ea9d7e30431
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
7ecf382e3328f5f1a934510f81a0b9ced153ea00ecfdd4a7cde1b0a674013fa9
5f2d4949c467f223a2decb50f9f420d11bf21ac112299d16768b1ea9d7e30431
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
    end
    on_intel do
      url "https://github.com/jezdez/conda-express/releases/download/#{version}/cx-x86_64-unknown-linux-gnu"
      sha256 "eb36d67891e90d2d6a86058cfeda817c0c1e6cf06bbc3ea70f4366b8614caa82
36a7309c60b486451ffe00186c0dff574ee1773da05ea7f6bc9c40ad089b2b58
3f3319f0352bc6b4e1456b84ee7e0bd745aaeee5f386d3515c4f6a706a1dcfe6
f591f22ae5941656115ad82b0ee9258a6abb6051d632cb2e68af5143089bf259"
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
