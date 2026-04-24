import struct, zlib, shutil

def create_png(width, height, color):
    def chunk(chunk_type, data):
        c = chunk_type + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)
    
    header = b'\x89PNG\r\n\x1a\n'
    ihdr = chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
    
    raw = b''
    for y in range(height):
        raw += b'\x00'
        for x in range(width):
            raw += bytes(color)
    
    idat = chunk(b'IDAT', zlib.compress(raw))
    iend = chunk(b'IEND', b'')
    return header + ihdr + idat + iend

orange = (249, 115, 22)
icons_dir = 'd:/code/code-vs/study-reminder/src-tauri/icons'

for name, size in [('32x32.png', 32), ('128x128.png', 128), ('128x128@2x.png', 256)]:
    png_data = create_png(size, size, orange)
    with open(f'{icons_dir}/{name}', 'wb') as f:
        f.write(png_data)
    print(f'Created {name} ({size}x{size})')

png32 = create_png(32, 32, orange)
ico = struct.pack('<HHH', 0, 1, 1)
ico += struct.pack('<BBBBHHII', 32, 32, 0, 0, 1, 32, len(png32), 22)
ico += png32
with open(f'{icons_dir}/icon.ico', 'wb') as f:
    f.write(ico)
print('Created icon.ico')

shutil.copy(f'{icons_dir}/128x128.png', f'{icons_dir}/icon.icns')
print('Created icon.icns')
