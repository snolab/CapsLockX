# CapsLockX - Mac Version

## RoadMap

- CLX:
  - [ ] CapsLock + ...
  - [x] Space + ...
- [ ] CLX-Edit
  - [x] Cursor Control by CLX + HJKL
    - [ ] With Smoothy physics model.
- [x] CLX-Mouse
  - [x] Mouse Control by CLX + WASD
    - [x] With Smoothy physics model.
  - [x] Mouse Button by CLX + QE
  - [x] Vertical Scroll Control by CLX + RF
  - [ ] Horizon Scroll Control by CLX + Shift + RF
- [x] CLX + HJKL g
- [ ] Desktop Switch
  - [x] CLX + 12 = Left Desktop or Right Desktop
  - [ ] CLX + 12345678 Jump to that Desktop

## Development

### Build Once

```bash

git clone

# Note: you need install golang by your self
go build

chmod +x clx
./clx

```

### Build (Watch mode)

```bash

git clone

# Note: you need install nodejs by your self
npm i -g nodemon

# Note: you need install golang by your self
go build

bash watch.sh
```
