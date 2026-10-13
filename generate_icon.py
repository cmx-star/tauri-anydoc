import zlib
import struct
import os

def create_png(width, height, filename):
    # 创建一个带有蓝色/青色渐变和边框的 PNG 图标
    raw_data = bytearray()
    for y in range(height):
        raw_data.append(0) # filter byte (None)
        for x in range(width):
            # 渐变背景与内部图案
            r = int(30 + (x / width) * 40)
            g = int(120 + (y / height) * 60)
            b = int(220 + (x / width) * 35)
            # 内部如果接近中心，画一个白色文档形状
            if 0.25 * width <= x <= 0.75 * width and 0.2 * height <= y <= 0.8 * height:
                # 文档主体
                if (x >= 0.6 * width and y <= 0.35 * height):
                    # 折角部分
                    r, g, b = 200, 220, 245
                else:
                    # 页面文字线条
                    if (0.32 * height <= y <= 0.36 * height or
                        0.42 * height <= y <= 0.46 * height or
                        0.52 * height <= y <= 0.56 * height or
                        0.62 * height <= y <= 0.66 * height) and (0.35 * width <= x <= 0.65 * width):
                        r, g, b = 40, 100, 180
                    else:
                        r, g, b = 255, 255, 255
            raw_data.extend([r, g, b, 255])

    def chunk(chunk_type, data):
        c = chunk_type + data
        crc = zlib.crc32(c) & 0xffffffff
        return struct.pack('>I', len(data)) + c + struct.pack('>I', crc)

    header = b'\x89PNG\r\n\x1a\n'
    ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    idat = zlib.compress(bytes(raw_data), 9)

    with open(filename, 'wb') as f:
        f.write(header)
        f.write(chunk(b'IHDR', ihdr))
        f.write(chunk(b'IDAT', idat))
        f.write(chunk(b'IEND', b''))

create_png(512, 512, 'app-icon.png')
print("Successfully generated app-icon.png")
