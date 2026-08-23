#!/usr/bin/env python3
"""Regression tests for decoded PNG pixel hashing."""

from __future__ import annotations

import binascii
import importlib.util
import struct
import sys
import tempfile
import unittest
import zlib
from pathlib import Path


sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location(
    "axodendron_png_pixel_sha256", SCRIPT_DIRECTORY / "png-pixel-sha256.py"
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load png-pixel-sha256.py")
PIXEL_HASH = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = PIXEL_HASH
SPEC.loader.exec_module(PIXEL_HASH)


def png_chunk(chunk_type: bytes, data: bytes) -> bytes:
    checksum = binascii.crc32(chunk_type + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + chunk_type + data + struct.pack(">I", checksum)


def paeth(left: int, above: int, upper_left: int) -> int:
    estimate = left + above - upper_left
    distances = (
        abs(estimate - left),
        abs(estimate - above),
        abs(estimate - upper_left),
    )
    return (left, above, upper_left)[distances.index(min(distances))]


def encode_rows(width: int, pixels: bytes, filters: list[int]) -> bytes:
    row_size = width * 4
    previous = bytes(row_size)
    output = bytearray()
    for row_index, filter_type in enumerate(filters):
        raw = pixels[row_index * row_size : (row_index + 1) * row_size]
        output.append(filter_type)
        for index, value in enumerate(raw):
            left = raw[index - 4] if index >= 4 else 0
            above = previous[index]
            upper_left = previous[index - 4] if index >= 4 else 0
            if filter_type == 0:
                predictor = 0
            elif filter_type == 1:
                predictor = left
            elif filter_type == 2:
                predictor = above
            elif filter_type == 3:
                predictor = (left + above) // 2
            elif filter_type == 4:
                predictor = paeth(left, above, upper_left)
            else:
                raise ValueError("invalid test filter")
            output.append((value - predictor) & 0xFF)
        previous = raw
    return bytes(output)


def encode_png(
    width: int,
    height: int,
    pixels: bytes,
    filters: list[int],
    compression_level: int,
    include_text: bool = False,
) -> bytes:
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    chunks = [png_chunk(b"IHDR", header)]
    if include_text:
        chunks.append(png_chunk(b"tEXt", b"variant\x00metadata"))
    compressed = zlib.compress(encode_rows(width, pixels, filters), compression_level)
    chunks.extend((png_chunk(b"IDAT", compressed), png_chunk(b"IEND", b"")))
    return PIXEL_HASH.PNG_SIGNATURE + b"".join(chunks)


class PixelHashTests(unittest.TestCase):
    def setUp(self) -> None:
        self.width = 3
        self.height = 5
        self.pixels = bytes((index * 37 + 11) % 256 for index in range(60))

    def test_encoding_details_do_not_change_the_pixel_digest(self) -> None:
        with tempfile.TemporaryDirectory(prefix="axodendron-png-hash-") as directory:
            first = Path(directory) / "first.png"
            second = Path(directory) / "second.png"
            first.write_bytes(
                encode_png(self.width, self.height, self.pixels, [0] * 5, 0)
            )
            second.write_bytes(
                encode_png(
                    self.width,
                    self.height,
                    self.pixels,
                    [0, 1, 2, 3, 4],
                    9,
                    include_text=True,
                )
            )
            self.assertNotEqual(first.read_bytes(), second.read_bytes())
            self.assertEqual(
                PIXEL_HASH.pixel_digest(first), PIXEL_HASH.pixel_digest(second)
            )

    def test_one_changed_channel_changes_the_pixel_digest(self) -> None:
        changed = bytearray(self.pixels)
        changed[17] ^= 1
        with tempfile.TemporaryDirectory(prefix="axodendron-png-hash-") as directory:
            first = Path(directory) / "first.png"
            second = Path(directory) / "second.png"
            first.write_bytes(
                encode_png(self.width, self.height, self.pixels, [0] * 5, 6)
            )
            second.write_bytes(
                encode_png(self.width, self.height, bytes(changed), [0] * 5, 6)
            )
            self.assertNotEqual(
                PIXEL_HASH.pixel_digest(first), PIXEL_HASH.pixel_digest(second)
            )

    def test_crc_corruption_is_rejected(self) -> None:
        encoded = bytearray(
            encode_png(self.width, self.height, self.pixels, [0] * 5, 6)
        )
        encoded[-5] ^= 1
        with tempfile.TemporaryDirectory(prefix="axodendron-png-hash-") as directory:
            corrupted = Path(directory) / "corrupted.png"
            corrupted.write_bytes(encoded)
            with self.assertRaises(PIXEL_HASH.PngError):
                PIXEL_HASH.pixel_digest(corrupted)


if __name__ == "__main__":
    unittest.main()
