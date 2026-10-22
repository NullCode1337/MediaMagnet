# Maintainer: NullCode1337 <contact.nullcode+4323@proton.me>

pkgname=mediamagnet-git
pkgver=0.6.0
pkgrel=1
pkgdesc="Download any and all media"
arch=('x86_64' 'aarch64')
url="https://github.com/NullCode1337/MediaMagnet"
license=('AGPL-3.0-only')
depends=('cairo' 'dbus' 'deno' 'ffmpeg' 'gcc-libs' 'gdk-pixbuf2' 'glib2' 'glibc'
         'gtk3' 'hicolor-icon-theme' 'libsoup3' 'pango' 'curl-impersonate' 'webkit2gtk-4.1')
makedepends=('git' 'bun' 'rust' 'pkgconf')
provides=('mediamagnet')
conflicts=('mediamagnet' 'mediamagnet-bin')

_python_release=20261013
_python_version=3.12.15

_python_url="https://github.com/astral-sh/python-build-standalone/releases/download/${_python_release}"

source=("${pkgname}::git+${url}.git")
sha256sums=('SKIP')

# The interpreter is architecture specific, so its source and checksum are too;
# makepkg merges the matching set into the build. Deno and ffmpeg come from the
# repositories.
source_x86_64=(
  "python.tar.gz::${_python_url}/cpython-${_python_version}+${_python_release}-x86_64-unknown-linux-gnu-install_only.tar.gz"
)
sha256sums_x86_64=(
  '5d546fc004fc2435fa142eb8f42ed8de5eb4725963dc22f7e6671014a80eb560'
)

source_aarch64=(
  "python.tar.gz::${_python_url}/cpython-${_python_version}+${_python_release}-aarch64-unknown-linux-gnu-install_only.tar.gz"
)
sha256sums_aarch64=(
  '34a43010cc05982d8b5debbad7d09f7b1200117ebf018f7acfd656895e408205'
)

noextract=('python.tar.gz')

# Stripping the bundled interpreter's shared library breaks its symbol
# versioning, and the payload needs no debug package.
options=('!strip' '!debug' '!lto')

pkgver() {
  cd "${pkgname}"
  ( set -o pipefail
    git describe --long --abbrev=7 2>/dev/null | sed 's/\([^-]*-g\)/r\1/;s/-/./g' ||
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short=7 HEAD)"
  )
}

prepare() {
  cd "${pkgname}"

  bun install --frozen-lockfile

  # The desktop bundle ships these as resources; the app resolves them at
  # /usr/lib/MediaMagnet/binaries.
  local bindir=apps/desktop/src-tauri/binaries
  rm -rf "${bindir}"
  mkdir -p "${bindir}/python"

  bsdtar -xf "${srcdir}/python.tar.gz" -C "${bindir}/python" --strip-components=1
  "${bindir}/python/bin/python3" -m pip install --no-cache-dir \
    --disable-pip-version-check --upgrade \
    "yt-dlp[default]" "gallery-dl[extra]" "spotdl"
}

build() {
  cd "${pkgname}"
  bun run tauri build --no-bundle
  strip --strip-all target/release/MediaMagnet
}

package() {
  cd "${pkgname}"

  install -Dm755 target/release/MediaMagnet "${pkgdir}/usr/bin/MediaMagnet"

  local icons=apps/desktop/src-tauri/icons hicolor="${pkgdir}/usr/share/icons/hicolor"
  install -Dm644 "${icons}/32x32.png" "${hicolor}/32x32/apps/MediaMagnet.png"
  install -Dm644 "${icons}/128x128.png" "${hicolor}/128x128/apps/MediaMagnet.png"
  install -Dm644 "${icons}/128x128@2x.png" "${hicolor}/256x256/apps/MediaMagnet.png"
  install -Dm644 "${icons}/icon.png" "${hicolor}/512x512/apps/MediaMagnet.png"

  install -d "${pkgdir}/usr/share/applications"
  cat > "${pkgdir}/usr/share/applications/MediaMagnet.desktop" <<-EOF
	[Desktop Entry]
	Type=Application
	Name=MediaMagnet
	Comment=Download any and all media
	Exec=MediaMagnet
	Icon=MediaMagnet
	Terminal=false
	Categories=AudioVideo;Network;
	StartupWMClass=MediaMagnet
	EOF

  install -d "${pkgdir}/usr/lib/MediaMagnet"
  cp -a apps/desktop/src-tauri/binaries "${pkgdir}/usr/lib/MediaMagnet/"

  # pip baked the build path into the console scripts; point them at the
  # installed interpreter. The app itself runs `python -m <module>`.
  local script
  for script in "${pkgdir}"/usr/lib/MediaMagnet/binaries/python/bin/*; do
    if [[ ! -L $script && $(head -c 2 "$script") == '#!' ]]; then
      sed -i "s|${PWD}/apps/desktop/src-tauri/binaries|/usr/lib/MediaMagnet/binaries|g" "$script"
    fi
  done
}
