FROM golang as CodeGolang


# https://obel.hatenablog.jp/entry/20210331/1617132600
RUN apt-get update && apt-get install -y gcc-multilib gcc-mingw-w64 binutils-mingw-w64
# GOOS=windows GOARCH=amd64 \
# CGO_ENABLED=1 CXX=x86_64-w64-mingw32-g++ CC=x86_64-w64-mingw32-gcc \
# go build -o clx_win64.exe

WORKDIR /code

# fix golang path
RUN echo "export PATH=\$PATH:/usr/local/go/bin" >> ~/.bashrc

CMD bash build.sh

# CMD \
#     go mod tidy && \
#     GOOS=windows GOARCH=amd64 go build -o clx_win64.exe && \
#