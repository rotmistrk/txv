//! Minimal PNG encoder — produces valid uncompressed PNG from RGBA data.
//! No external dependencies. Uses store (uncompressed) zlib blocks.

/// Encode RGBA pixels as a PNG file.
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut png = Vec::new();
    // PNG signature
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    // IHDR
    write_chunk(&mut png, b"IHDR", &ihdr(width, height));
    // IDAT
    let idat_data = build_idat(width, height, rgba);
    write_chunk(&mut png, b"IDAT", &idat_data);
    // IEND
    write_chunk(&mut png, b"IEND", &[]);
    png
}

fn ihdr(w: u32, h: u32) -> [u8; 13] {
    let mut d = [0u8; 13];
    d[0..4].copy_from_slice(&w.to_be_bytes());
    d[4..8].copy_from_slice(&h.to_be_bytes());
    d[8] = 8; // bit depth
    d[9] = 6; // color type: RGBA
    d[10] = 0; // compression
    d[11] = 0; // filter
    d[12] = 0; // interlace
    d
}

fn build_idat(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    // Raw image data: filter byte (0=None) + row pixels for each row
    let row_len = 1 + width as usize * 4; // filter byte + RGBA
    let raw_len = row_len * height as usize;
    let mut raw = Vec::with_capacity(raw_len);
    for y in 0..height as usize {
        raw.push(0); // filter: None
        let start = y * width as usize * 4;
        let end = start + width as usize * 4;
        raw.extend_from_slice(&rgba[start..end]);
    }
    // Wrap in zlib: stored (uncompressed) blocks
    zlib_store(&raw)
}

/// Produce a valid zlib stream using only stored (uncompressed) blocks.
fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    // Zlib header: CM=8, CINFO=7, FCHECK so header%31==0
    out.push(0x78); // CMF
    out.push(0x01); // FLG (no dict, level 0, check bits)
    // Deflate stored blocks (max 65535 bytes each)
    let mut offset = 0;
    while offset < data.len() {
        let remaining = data.len() - offset;
        let block_len = remaining.min(65535);
        let last = if offset + block_len >= data.len() { 1u8 } else { 0u8 };
        out.push(last); // BFINAL + BTYPE=00 (stored)
        let len = block_len as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes()); // NLEN
        out.extend_from_slice(&data[offset..offset + block_len]);
        offset += block_len;
    }
    // Adler-32 checksum
    let checksum = adler32(data);
    out.extend_from_slice(&checksum.to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let len = data.len() as u32;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    let crc = crc32(&crc_input);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
