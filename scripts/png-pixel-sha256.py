#!/usr/bin/env python3
"""Hash decoded pixels from the constrained PNGs used by visual regression."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import struct
import sys
import zlib
from pathlib import Path


PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


class PngError(ValueError):
    """Raised when a PNG cannot be decoded without ambiguity."""


def paeth_predictor(left: int, above: int, upper_left: int) -> int:
    estimate = left + above - upper_left
    left_distance = abs(estimate - left)
    above_distance = abs(estimate - above)
    upper_left_distance = abs(estimate - upper_left)
    if left_distance <= above_distance and left_distance <= upper_left_distance:
        return left
    if above_distance <= upper_left_distance:
        return above
    return upper_left


def decode_rgba8(path: Path) -> tuple[int, int, bytes]:
    source = path.read_bytes()
    if not source.startswith(PNG_SIGNATURE):
        raise PngError("invalid PNG signature")

    offset = len(PNG_SIGNATURE)
    header: tuple[int, int, int, int, int, int, int] | None = None
    compressed = bytearray()
    saw_end = False
    while offset < len(source):
        if offset + 12 > len(source):
            raise PngError("truncated PNG chunk")
        length = struct.unpack(">I", source[offset : offset + 4])[0]
        chunk_type = source[offset + 4 : offset + 8]
        data_start = offset + 8
        data_end = data_start + length
        crc_end = data_end + 4
        if crc_end > len(source):
            raise PngError("PNG chunk exceeds the file boundary")
        chunk_data = source[data_start:data_end]
        declared_crc = struct.unpack(">I", source[data_end:crc_end])[0]
        actual_crc = binascii.crc32(chunk_type + chunk_data) & 0xFFFFFFFF
        if declared_crc != actual_crc:
            raise PngError(f"CRC mismatch in {chunk_type!r}")
        if chunk_type == b"IHDR":
            if header is not None or length != 13:
                raise PngError("invalid or duplicate IHDR")
            header = struct.unpack(">IIBBBBB", chunk_data)
        elif chunk_type == b"IDAT":
            if header is None:
                raise PngError("IDAT precedes IHDR")
            compressed.extend(chunk_data)
        elif chunk_type == b"IEND":
            if length != 0:
                raise PngError("IEND must be empty")
            saw_end = True
            offset = crc_end
            break
        offset = crc_end

    if header is None or not saw_end or offset != len(source):
        raise PngError("incomplete PNG structure")
    width, height, bit_depth, color_type, compression, filtering, interlace = header
    if width == 0 or height == 0:
        raise PngError("zero-sized PNG")
    if (bit_depth, color_type, compression, filtering, interlace) != (8, 6, 0, 0, 0):
        raise PngError("visual baselines require non-interlaced 8-bit RGBA PNGs")

    decompressor = zlib.decompressobj()
    filtered = decompressor.decompress(bytes(compressed)) + decompressor.flush()
    if not decompressor.eof or decompressor.unused_data or decompressor.unconsumed_tail:
        raise PngError("invalid or trailing zlib stream")
    bytes_per_pixel = 4
    row_size = width * bytes_per_pixel
    expected_size = height * (row_size + 1)
    if len(filtered) != expected_size:
        raise PngError("decompressed image size does not match IHDR")

    pixels = bytearray(height * row_size)
    previous = bytearray(row_size)
    source_offset = 0
    output_offset = 0
    for _ in range(height):
        filter_type = filtered[source_offset]
        source_offset += 1
        encoded = filtered[source_offset : source_offset + row_size]
        source_offset += row_size
        decoded = bytearray(row_size)
        for index, value in enumerate(encoded):
            left = decoded[index - bytes_per_pixel] if index >= bytes_per_pixel else 0
            above = previous[index]
            upper_left = previous[index - bytes_per_pixel] if index >= bytes_per_pixel else 0
            if filter_type == 0:
                predictor = 0
            elif filter_type == 1:
                predictor = left
            elif filter_type == 2:
                predictor = above
            elif filter_type == 3:
                predictor = (left + above) // 2
            elif filter_type == 4:
                predictor = paeth_predictor(left, above, upper_left)
            else:
                raise PngError(f"unsupported PNG row filter {filter_type}")
            decoded[index] = (value + predictor) & 0xFF
        pixels[output_offset : output_offset + row_size] = decoded
        output_offset += row_size
        previous = decoded

    return width, height, bytes(pixels)


def pixel_digest(path: Path) -> str:
    width, height, pixels = decode_rgba8(path)
    digest = hashlib.sha256()
    digest.update(b"axodendron-rgba8-v1\0")
    digest.update(struct.pack(">II", width, height))
    digest.update(pixels)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Print a SHA-256 digest of PNG dimensions and decoded RGBA8 pixels."
    )
    parser.add_argument("png", type=Path)
    arguments = parser.parse_args()
    try:
        print(pixel_digest(arguments.png))
    except (OSError, PngError, zlib.error) as error:
        print(f"PNG pixel hashing failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
