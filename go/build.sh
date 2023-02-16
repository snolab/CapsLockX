# export PATH=$PATH:/usr/local/go/bin

# solve zlib-x64 problem
# https://github.com/go-vgo/robotgo/issues/100
# https://github.com/lowkey42/MagnumOpus/wiki/TDM-GCC-Mingw64-Installation#zlib-x64
# cp -r _/zlib/bin /TDM/bin
# cp -r _/zlib/bin /Git/bin
# cp -r /zlib/include /TDM/include
# cp -r /zlib/lib /TDM/lib

cp -r ./zlib/include/* /usr/lib/gcc/x86_64-w64-mingw32/10-win32/include
cp -r ./zlib/lib/* /usr/lib/gcc/x86_64-w64-mingw32/10-win32

# solve build cross to windows problem
# https://obel.hatenablog.jp/entry/20210331/1617132600
# apt install gcc-multilib gcc-mingw-w64 binutils-mingw-w64
GOOS=windows GOARCH=amd64 \
    CGO_ENABLED=1 CXX=x86_64-xxw64-mingw32-g++ CC=x86_64-w64-mingw32-gcc \
    go build -o clx_win64.exe main.go mods_windows.go

