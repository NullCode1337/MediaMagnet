# Maintainer: NullCode1337 
# Contributor: NullCode1337 

pkgname=mediamagnet-git
pkgver=0.5.3
pkgrel=1
pkgdesc="Download any and all media" 
arch=('x86_64')
url="https://github.com/NullCode1337/MediaMagnet" 
license=('MIT') 
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'yt-dlp' 'python-curl_cffi')
optdepends=('gallery-dl: Required for gallery-dl media downloading support'
            'spotdl: Required for spotify music downloading support'
            'ffmpeg: Required for yt-dlp video/audio merging and spotdl support')
makedepends=('git' 'openssl' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'cargo' 'pnpm' 'nodejs' 'rust' 'bun')
provides=('mediamagnet')
conflicts=('mediamagnet' 'mediamagnet-bin')
source=("${pkgname}::git+${url}.git")
sha256sums=('SKIP')

pkgver() {
  cd "${pkgname}"
  ( set -o pipefail
    git describe --long --tags --abbrev=7 2>/dev/null | sed 's/^v//;s/\([^-]*-g\)/r\1/;s/-/./g' ||
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short=7 HEAD)"
  )
}

prepare() {
  cd "${pkgname}"
  pnpm install
  cd "src-tauri"
  cargo update
}

build() {
  cd "${pkgname}"
  pnpm tauri build -b deb 
}

package() {
  cd "${pkgname}"
  cp -a src-tauri/target/release/bundle/deb/MediaMagnet_*_*/data/* "${pkgdir}/"
}
