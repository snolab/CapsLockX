FROM golang as CodeGolangBuildForWin64

# https://obel.hatenablog.jp/entry/20210331/1617132600
RUN apt-get update && apt-get install -y gcc-multilib gcc-mingw-w64 binutils-mingw-w64
# GOOS=windows GOARCH=amd64 \
# CGO_ENABLED=1 CXX=x86_64-w64-mingw32-g++ CC=x86_64-w64-mingw32-gcc \
# go build -o clx_win64.exe

# fix golang path
# export PATH=$PATH:/usr/local/go/bin
RUN echo "export PATH=\$PATH:/usr/local/go/bin" >> ~/.bashrc

# solve zlib-x64 problem
# https://github.com/go-vgo/robotgo/issues/100
# https://github.com/lowkey42/MagnumOpus/wiki/TDM-GCC-Mingw64-Installation#zlib-x64
# cp -r _/zlib/bin /TDM/bin
# cp -r _/zlib/bin /Git/bin
# cp -r /zlib/include /TDM/include
# cp -r /zlib/lib /TDM/lib
RUN cp -r ./zlib/include/* /usr/lib/gcc/x86_64-w64-mingw32/10-win32/include
RUN cp -r ./zlib/lib/* /usr/lib/gcc/x86_64-w64-mingw32/10-win32

COPY . /src/go
WORKDIR /src/go

# build for windows x64
# solve build cross to windows problem
# https://obel.hatenablog.jp/entry/20210331/1617132600
# apt install gcc-multilib gcc-mingw-w64 binutils-mingw-w64
RUN GOOS=windows GOARCH=amd64 \
    CGO_ENABLED=1 CXX=x86_64-xxw64-mingw32-g++ CC=x86_64-w64-mingw32-gcc \
    go build -o bin/capslockx.exe

# build for OSX 64-bit
RUN GOOS=darwin GOARCH=amd64 go build -o bin/capslockx main.go

