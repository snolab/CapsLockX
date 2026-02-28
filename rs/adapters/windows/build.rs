fn main() {
    // Icons must be generated before tauri_build::build() checks for them.
    generate_icons();
    tauri_build::build();
}

fn generate_icons() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest.join("icons");
    std::fs::create_dir_all(&dir).expect("cannot create icons/");

    let png = make_png();

    let tray = dir.join("tray.png");
    if !tray.exists() {
        std::fs::write(&tray, &png).expect("cannot write icons/tray.png");
    }

    let ico = dir.join("icon.ico");
    if !ico.exists() {
        std::fs::write(&ico, make_ico(&png)).expect("cannot write icons/icon.ico");
    }

    println!("cargo:rerun-if-changed=icons/tray.png");
    println!("cargo:rerun-if-changed=icons/icon.ico");
}

// ── 32×32 RGBA PNG (Catppuccin Mocha palette) ────────────────────────────────

fn make_png() -> Vec<u8> {
    const W: u32 = 32;
    const H: u32 = 32;
    const BG: [u8; 4] = [0x1e, 0x1e, 0x2e, 0xff]; // Mocha base
    const AC: [u8; 4] = [0x89, 0xb4, 0xfa, 0xff]; // Mocha blue

    let row_stride = 1 + (W as usize) * 4;
    let mut img = vec![0u8; (H as usize) * row_stride];
    for row in 0..H as usize {
        img[row * row_stride] = 0; // filter byte: None
        for col in 0..W as usize {
            let border = col < 4 || col >= (W as usize - 4)
                || row < 4  || row >= (H as usize - 4);
            let color = if border { AC } else { BG };
            let i = row * row_stride + 1 + col * 4;
            img[i]   = color[0];
            img[i+1] = color[1];
            img[i+2] = color[2];
            img[i+3] = color[3];
        }
    }

    let compressed = zlib_store(&img);
    let mut png = Vec::with_capacity(512 + compressed.len());
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = [0u8; 13];
    ihdr[0..4].copy_from_slice(&W.to_be_bytes());
    ihdr[4..8].copy_from_slice(&H.to_be_bytes());
    ihdr[8] = 8; // bit depth
    ihdr[9] = 6; // colour type: RGBA
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &compressed);
    chunk(&mut png, b"IEND", &[]);
    png
}

/// Wrap PNG bytes in an ICO container (Vista+ PNG-in-ICO format).
fn make_ico(png: &[u8]) -> Vec<u8> {
    let size   = png.len() as u32;
    let offset = 22u32; // 6-byte header + 16-byte dir entry

    let mut ico = Vec::with_capacity(22 + png.len());
    // ICO file header
    ico.extend_from_slice(&[0x00, 0x00]); // reserved
    ico.extend_from_slice(&[0x01, 0x00]); // type: icon
    ico.extend_from_slice(&[0x01, 0x00]); // image count: 1
    // Image directory entry
    ico.push(32);                         // width  (0 = 256)
    ico.push(32);                         // height
    ico.push(0);                          // colour count (0 = truecolor)
    ico.push(0);                          // reserved
    ico.extend_from_slice(&[0x01, 0x00]); // colour planes
    ico.extend_from_slice(&[0x20, 0x00]); // bits per pixel (32)
    ico.extend_from_slice(&size.to_le_bytes());
    ico.extend_from_slice(&offset.to_le_bytes());
    // Payload
    ico.extend_from_slice(png);
    ico
}

// ── Minimal zlib / PNG helpers ────────────────────────────────────────────────

fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01]; // zlib header (deflate, fastest; 0x7801 % 31 == 0)
    let mut offset = 0;
    while offset < data.len() {
        let end   = (offset + 65535).min(data.len());
        let block = &data[offset..end];
        let bfinal = u8::from(end == data.len());
        let len   = block.len() as u16;
        let nlen  = !len;
        out.push(bfinal);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&nlen.to_le_bytes());
        out.extend_from_slice(block);
        offset = end;
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a)           % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, ty: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(ty);
    out.extend_from_slice(data);
    let mut crc = 0xFFFF_FFFFu32;
    for &b in ty.iter().chain(data) {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { 0xEDB8_8320 ^ (crc >> 1) } else { crc >> 1 };
        }
    }
    out.extend_from_slice(&(crc ^ 0xFFFF_FFFF).to_be_bytes());
}
