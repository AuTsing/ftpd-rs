import os
import shutil


def build():
    os.environ['ANDROID_NDK_ROOT'] = '/home/autsing/Apps/AndroidSdk/ndk/23.2.8568313'
    os.environ['LLVM_ROOT'] = f'{os.environ["ANDROID_NDK_ROOT"]}/toolchains/llvm/prebuilt/linux-x86_64'
    os.environ['CC_aarch64_linux_android'] = f'{os.environ["LLVM_ROOT"]}/bin/aarch64-linux-android28-clang'
    os.environ['AR_aarch64_linux_android'] = f'{os.environ["LLVM_ROOT"]}/bin/llvm-ar'
    os.environ['CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER'] = os.environ['CC_aarch64_linux_android']
    return os.system('cargo build --target aarch64-linux-android')


def export():
    src = './target/aarch64-linux-android/debug/libftpd.so'
    dst = '/home/autsing/OneDrive/Gits/Java/Denort/app/src/main/jniLibs/arm64-v8a'

    print('exporting...')

    shutil.copy(src, dst)


exit_code = build()
if exit_code == 0:
    export()
    print('done')
