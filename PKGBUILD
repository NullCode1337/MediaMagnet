# Maintainer: NullCode1337 <contact.nullcode+4323@proton.me>

pkgname=mediamagnet-git
pkgver=0.5.3
pkgrel=1
pkgdesc="Download any and all media" 
arch=('x86_64' 'aarch64')
url="https://github.com/NullCode1337/MediaMagnet" 
license=('AGPL-3.0-only')
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'yt-dlp' 'python-curl_cffi')
optdepends=('gallery-dl: Required for gallery-dl media downloading support'
            'spotdl: Required for spotify music downloading support'
            'ffmpeg: Required for yt-dlp video/audio merging and spotdl support')
makedepends=('git' 'openssl' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'cargo' 'bun')
provides=('mediamagnet')
conflicts=('mediamagnet' 'mediamagnet-bin')
source=("${pkgname}::git+${url}.git")
sha256sums=('SKIP')

pkgver() {
  cd "${pkgname}"
  ( set -o pipefail
    git describe --long --abbrev=7 2>/dev/null | sed 's/\([^-]*-g\)/r\1/;s/-/./g' ||
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short=7 HEAD)"
  )
}

prepare() {
  cd "${pkgname}"
  bun install
}

build() {
  cd "${pkgname}"
  bun tauri build -b deb 
}

package() {
  cd "${pkgname}"
  cp -a src-tauri/target/release/bundle/deb/MediaMagnet_*_*/data/* "${pkgdir}/"

  if [ -f "${pkgdir}/usr/bin/gallery-dl" ]; then
    rm "${pkgdir}/usr/bin/gallery-dl" 
  fi

  if [ -f "${pkgdir}/usr/bin/yt-dlp" ]; then
    rm "${pkgdir}/usr/bin/yt-dlp" 
  fi
}
