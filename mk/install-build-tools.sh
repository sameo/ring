#!/usr/bin/env bash
#
# Copyright 2020 Brian Smith.
#
# Permission to use, copy, modify, and/or distribute this software for any
# purpose with or without fee is hereby granted, provided that the above
# copyright notice and this permission notice appear in all copies.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
# WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
# MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
# SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
# WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
# OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
# CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

set -eux -o pipefail
IFS=$'\n\t'

target=$1
features=${2-}

function install_packages {
  sudo apt-get -yq --no-install-suggests --no-install-recommends install "$@"
}

use_clang=
case $target in
--target*android*)
  mkdir -p "${ANDROID_HOME}/licenses"
  ndk_version=25.1.8937393
  android_license_file="${ANDROID_HOME}/licenses/android-sdk-license"
  accept_android_license=24333f8a63b6825ea9c5514f83c2829b004d1fee
  grep --quiet --no-messages "$accept_android_license" "$android_license_file" \
    || echo $accept_android_license  >> "$android_license_file"
  "${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager" "ndk;$ndk_version"

  # XXX: Older Rust toolchain versions link with `-lgcc` instead of `-lunwind`;
  # see https://github.com/rust-lang/rust/pull/85806.
  find -L ${ANDROID_NDK_ROOT:-${ANDROID_HOME}/ndk/$ndk_version} -name libunwind.a \
          -execdir sh -c 'echo "INPUT(-lunwind)" > libgcc.a' \;
  ;;
esac

case $target in
--target=aarch64-unknown-linux-gnu)
  # Clang is needed for code coverage.
  use_clang=1
  install_packages \
    qemu-user \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross
  ;;
--target=aarch64-unknown-linux-musl|--target=armv7-unknown-linux-musleabihf)
  use_clang=1
  install_packages \
    qemu-user
  ;;
--target=arm-unknown-linux-gnueabihf)
  install_packages \
    qemu-user \
    gcc-arm-linux-gnueabihf \
    libc6-dev-armhf-cross
  ;;
--target=i686-unknown-linux-gnu)
  use_clang=1
  install_packages \
    gcc-multilib \
    libc6-dev-i386
  ;;
--target=i686-unknown-linux-musl|--target=x86_64-unknown-linux-musl)
  use_clang=1
  ;;
--target=mipsel-unknown-linux-gnu)
  install_packages \
    gcc-mipsel-linux-gnu \
    libc6-dev-mipsel-cross \
    qemu-user
  ;;
--target=riscv64gc-unknown-linux-gnu)
  install_packages \
    gcc-riscv64-linux-gnu \
    libc6-dev-riscv64-cross \
    qemu-user
  ;;
--target=wasm32-unknown-unknown)
  cargo install wasm-bindgen-cli --bin wasm-bindgen-test-runner
  use_clang=1
  ;;
--target=*)
  ;;
esac

case "$OSTYPE" in
linux*)
  ubuntu_codename=$(lsb_release --codename --short)
  llvm_version=15
  sudo apt-key add mk/llvm-snapshot.gpg.key
  sudo add-apt-repository "deb http://apt.llvm.org/$ubuntu_codename/ llvm-toolchain-$ubuntu_codename-$llvm_version main"
  sudo apt-get update
  # We need to use `llvm-nm` in `mk/check-symbol-prefixes.sh`.
  install_packages llvm-$llvm_version
  if [ -n "$use_clang" ]; then
    install_packages clang-$llvm_version
  fi
  ;;
esac
