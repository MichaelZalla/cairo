# Cairo

A single-threaded, 3D software-rendering pipeline in Rust.

## Postinstall

Install the SDL2 library.

```bash
# macOS

brew install sdl2

michaelzalla@r269-boh-2960-3 lib % ls -al /opt/homebrew/Cellar/sdl2/2.0.22/lib
total 7552
drwxr-xr-x   9 michaelzalla  admin      288 Jul  2 18:45 .
drwxr-xr-x  11 michaelzalla  admin      352 Jul  2 18:45 ..
drwxr-xr-x   3 michaelzalla  admin       96 Apr 25 11:10 cmake
-rw-r--r--   1 michaelzalla  admin  1350576 Jul  2 18:45 libSDL2-2.0.0.dylib
-r--r--r--   1 michaelzalla  admin  2250256 Apr 25 11:10 libSDL2.a
lrwxr-xr-x   1 michaelzalla  admin       19 Apr 25 11:10 libSDL2.dylib -> libSDL2-2.0.0.dylib
-r--r--r--   1 michaelzalla  admin   255520 Apr 25 11:10 libSDL2_test.a
-r--r--r--   1 michaelzalla  admin      736 Apr 25 11:10 libSDL2main.a
drwxr-xr-x   3 michaelzalla  admin       96 Jul  2 18:45 pkgconfig

# ~/.bash_profile
export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/Cellar/sdl2/2.0.22/lib"
```
