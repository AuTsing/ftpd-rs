#!/bin/bash

export CC=/home/autsing/Apps/AndroidSdk/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android26-clang
export AR=/home/autsing/Apps/AndroidSdk/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$CC

if [[ "$1" =~ "release" ]]; then
    echo "build release"
    cargo build --target aarch64-linux-android --release
    echo "copying file..."
    cp ./target/aarch64-linux-android/release/libftpd.so ~/OneDrive/Gits/Java/Denort/app/src/main/jniLibs/arm64-v8a/
else
    echo "build debug"
    cargo build --target aarch64-linux-android
    echo "copying file..."
    cp ./target/aarch64-linux-android/debug/libftpd.so ~/OneDrive/Gits/Java/Denort/app/src/main/jniLibs/arm64-v8a/
fi

echo "done."
