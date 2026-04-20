# Maintainer: NullCode1337 
# Contributor: NullCode1337 

pkgname=mediamagnet-git
pkgver=0.4.2.r15.g7a2b3c4
pkgrel=1
pkgdesc="Download any and all media" 
arch=('x86_64')
url="https://github.com/NullCode1337/MediaMagnet" 
license=('MIT') 
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'gallery-dl' 'yt-dlp' 'python-curl_cffi')
makedepends=('git' 'openssl' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'cargo' 'pnpm' 'nodejs')
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
  
  if [ -f "${pkgdir}/usr/bin/gallery-dl" ]; then
    rm "${pkgdir}/usr/bin/gallery-dl" 
  fi
}