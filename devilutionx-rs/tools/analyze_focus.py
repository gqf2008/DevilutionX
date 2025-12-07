#!/usr/bin/env python3
"""
验证 focus PCX 文件的帧分割是否正确
"""
import struct
import sys

def read_pcx_info(data):
    """读取 PCX 头信息"""
    if len(data) < 128:
        return None

    # PCX header structure
    manufacturer = data[0]
    version = data[1]
    encoding = data[2]
    bits_per_pixel = data[3]

    xmin = struct.unpack('<H', data[4:6])[0]
    ymin = struct.unpack('<H', data[6:8])[0]
    xmax = struct.unpack('<H', data[8:10])[0]
    ymax = struct.unpack('<H', data[10:12])[0]

    width = xmax - xmin + 1
    height = ymax - ymin + 1

    return {
        'manufacturer': manufacturer,
        'version': version,
        'encoding': encoding,
        'bits_per_pixel': bits_per_pixel,
        'width': width,
        'height': height,
        'xmin': xmin,
        'ymin': ymin,
        'xmax': xmax,
        'ymax': ymax
    }

def decode_pcx_pixels(data, width, height):
    """解码 PCX RLE 像素数据"""
    pixels = []
    idx = 128  # Skip header

    while len(pixels) < width * height and idx < len(data) - 768:
        byte = data[idx]
        idx += 1

        if byte >= 0xC0:  # RLE run
            count = byte & 0x3F
            value = data[idx]
            idx += 1
            pixels.extend([value] * count)
        else:
            pixels.append(byte)

    return pixels[:width * height]

def analyze_frame_differences(pixels, width, height, num_frames):
    """分析每帧之间的差异"""
    frame_height = height // num_frames
    frames = []

    for i in range(num_frames):
        start_row = i * frame_height
        frame_pixels = []
        for y in range(frame_height):
            row_start = (start_row + y) * width
            frame_pixels.extend(pixels[row_start:row_start + width])
        frames.append(frame_pixels)

    print(f"\n每帧大小: {width}x{frame_height}")
    print(f"帧像素数: {len(frames[0])}")

    # 计算每帧的唯一颜色索引
    for i, frame in enumerate(frames):
        unique = set(frame)
        # 统计非透明像素 (非 250)
        non_trans = [p for p in frame if p != 250]
        print(f"  帧 {i}: {len(unique)} 种颜色, {len(non_trans)} 非透明像素")

    # 计算相邻帧的差异
    print("\n帧差异分析:")
    for i in range(1, num_frames):
        diff_count = sum(1 for a, b in zip(frames[i-1], frames[i]) if a != b)
        diff_pct = diff_count * 100.0 / len(frames[0])
        print(f"  帧{i-1} vs 帧{i}: {diff_count} 像素不同 ({diff_pct:.1f}%)")

    # 检查是否所有帧都相同
    all_same = all(frames[0] == f for f in frames[1:])
    if all_same:
        print("\n⚠️ 警告: 所有帧完全相同! 动画不会有变化!")

    return frames

def main():
    # 这个脚本用于分析从 MPQ 提取的 focus PCX 文件
    # 实际运行需要先提取 PCX 文件
    print("Focus PCX 帧分析工具")
    print("=" * 50)

    # 预期结构
    files = [
        ("focus16.pcx", 16, 128, 8),   # 16x16, 8帧 = 16x128
        ("focus.pcx", 30, 240, 8),     # 30x30, 8帧 = 30x240
        ("focus42.pcx", 42, 336, 8),   # 42x42, 8帧 = 42x336
    ]

    for filename, expected_w, expected_h, frames in files:
        print(f"\n{filename}:")
        print(f"  预期: {expected_w}x{expected_h}, {frames}帧, 每帧 {expected_w}x{expected_h // frames}")

    print("\n" + "=" * 50)
    print("C++ 动画帧计算公式:")
    print("  frame_idx = (SDL_GetTicks() / 60) % 8")
    print("  完整循环: 8 * 60ms = 480ms")
    print("\nRust 应匹配此公式!")

    print("\n" + "=" * 50)
    print("检查点:")
    print("  1. PCX 是否正确按高度分割为8帧?")
    print("  2. 每帧的像素是否不同? (应该有轻微差异)")
    print("  3. 帧索引是否正确更新? (每60ms变化)")
    print("  4. 纹理是否正确创建? (ABGR8888格式)")

if __name__ == "__main__":
    main()
