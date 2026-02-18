# Maintainer: NullCode1337
# Contributor: NullCode1337
pkgname=MediaMagnet
pkgver=0.4.2
pkgrel=1
pkgdesc="Download any and all media"
arch=('x86_64')
url="https://github.com/NullCode1337/MediaMagnet"
license=('MIT')
depends=('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'gallery-dl' 'yt-dlp' 'python-curl_cffi')
options=('!strip' '!emptydirs')
install=${pkgname}.install
source_x86_64=("${url}/releases/download/v${pkgver}-alpha/MediaMagnet_${pkgver}_amd64.deb")
sha256sums_x86_64=('c6a567947da30f23bdc16bfe3dc4a6b296ea618be9a4f7c1f999c075abe886d3')
package() {
  tar -xvf data.tar.gz -C "${pkgdir}"
  rm "${pkgdir}/usr/bin/gallery-dl"
}