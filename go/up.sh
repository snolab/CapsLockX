
killall clx; chmod +x clx && ./clx &
npx -y nodemon -e go -s SIGTERM -x "go build && (killall clx; chmod +x clx && (./clx || exit 1) &)"
