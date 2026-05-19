# Maintainer: NullCode1337 
# Contributor: NullCode1337 

pkgname=mediamagnet-git
pkgver=0.5.2
pkgrel=1
pkgdesc="Download any and all media" 
arch=('x86_64')
url="https://github.com/NullCode1337/MediaMagnet" 
license=('MIT') 
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'yt-dlp' 'python-curl_cffi')
optdepends=('gallery-dl: Updated version of included prebuilt'
            'spotdl: Spotify music downloading support')
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
  mkdir -p src-tauri/binaries
  touch src-tauri/binaries/yt-dlp-x86_64-unknown-linux-gnu
  touch src-tauri/binaries/gallery-dl-x86_64-unknown-linux-gnu
  chmod +x src-tauri/binaries/*
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

  if [ -f "${pkgdir}/usr/bin/yt-dlp" ]; then
    rm "${pkgdir}/usr/bin/yt-dlp" 
  fi
}