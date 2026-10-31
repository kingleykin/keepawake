cask "keepawake" do
  version "1.1.0"
  sha256 "10261ae0f299181363da931706be4147be5113427f8e88ef8925abc23ab5c3e9"

  url "https://github.com/kingleykin/keepawake/releases/download/v#{version}/KeepAwake_#{version}_universal.dmg"
  name "KeepAwake"
  desc "Keep your computer awake from the menu bar"
  homepage "https://github.com/kingleykin/keepawake"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates true
  depends_on :macos

  app "KeepAwake.app"

  uninstall quit: "io.github.kingleykin.keepawake"

  zap trash: [
    "~/Library/Application Support/io.github.kingleykin.keepawake",
    "~/Library/Caches/io.github.kingleykin.keepawake",
    "~/Library/LaunchAgents/KeepAwake.plist",
    "~/Library/WebKit/io.github.kingleykin.keepawake",
  ]

  caveats <<~EOS
    KeepAwake is not notarized by Apple yet. If macOS refuses to open it, run:
      xattr -dr com.apple.quarantine #{appdir}/KeepAwake.app
  EOS
end
