FROM golang as CodeGolang


WORKDIR /code

# https://obel.hatenablog.jp/entry/20210331/1617132600
RUN apt-get update && apt-get install -y gcc-multilib gcc-mingw-w64 binutils-mingw-w64
# GOOS=windows GOARCH=amd64 \
# CGO_ENABLED=1 CXX=x86_64-w64-mingw32-g++ CC=x86_64-w64-mingw32-gcc \
# go build -o clx_win64.exe


# fix golang path
RUN echo "export PATH=\$PATH:/usr/local/go/bin" >> ~/.bashrc


# vscode server install and run
RUN apt-get update && apt-get install -y git curl
RUN curl -fsSL https://aka.ms/install-vscode-server/setup.sh | sh
CMD echo start... \
    && echo http://localhost:35882/?folder=/code\
    && code-server serve-local \
    --install-extension golang.go \
    && code-server serve-local \
    --accept-server-license-terms \
    --host 0.0.0.0 \
    --without-connection-token
EXPOSE 8000

# CMD \
#     go mod tidy && \
#     GOOS=windows GOARCH=amd64 go build -o clx_win64.exe && \
#