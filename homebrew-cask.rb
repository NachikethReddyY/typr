# Typr Homebrew Cask Formula
# Install with: brew install --cask typr
# Or add tap first: brew tap nachikethreddyy/typr

cask 'typr' do
  version '0.2.0'
  
  if Hardware::CPU.type == :arm
    url "https://github.com/NachikethReddyY/typr/releases/download/v0.2.0/Typr_0.2.0_aarch64.dmg"
    sha256 'REPLACE_WITH_SHA256_AFTER_RELEASE'
  else
    url "https://github.com/NachikethReddyY/typr/releases/download/v0.2.0/Typr_0.2.0_x64.dmg"
    sha256 'REPLACE_WITH_SHA256_AFTER_RELEASE'
  end

  name 'Typr'
  desc 'Minimal dictation app with auto-routing (local >90s, cloud <90s)'
  homepage 'https://github.com/NachikethReddyY/typr'
  
  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: '>='12.0
  
  app 'Typr.app'
  zap trash: [
    '~/Library/Application Support/com.typr.app',
    '~/Library/Caches/com.typr.app',
  ]
end
