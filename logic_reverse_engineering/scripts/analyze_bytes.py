#!/usr/bin/env python3
"""
Analyze bytes at a specific offset in a binary file.
Interprets bytes as various data types to identify the format.
"""

import struct
import sys
import argparse
from pathlib import Path


def analyze_offset(filename, offset, byte_count=4):
    """Analyze bytes at a specific offset."""
    filepath = Path(filename)

    if not filepath.exists():
        print(f"Error: File not found: {filename}")
        sys.exit(1)

    with open(filepath, 'rb') as f:
        f.seek(offset)
        data = f.read(byte_count)

    if len(data) < byte_count:
        print(f"Warning: Only read {len(data)} bytes (requested {byte_count})")

    print(f"\n{'='*60}")
    print(f"File: {filepath.name}")
    print(f"Offset: 0x{offset:X} ({offset} decimal)")
    print(f"{'='*60}\n")

    # Raw hex
    hex_str = ' '.join(f'{b:02x}' for b in data)
    print(f"Raw Bytes: {hex_str}")
    print()

    # Try various interpretations if we have 4 bytes
    if len(data) == 4:
        print("Interpretations:")
        print("-" * 40)

        # Float (little-endian)
        try:
            val = struct.unpack('<f', data)[0]
            print(f"  Float (LE):     {val:>15.6f}")
        except:
            print(f"  Float (LE):     [invalid]")

        # Float (big-endian)
        try:
            val = struct.unpack('>f', data)[0]
            print(f"  Float (BE):     {val:>15.6f}")
        except:
            print(f"  Float (BE):     [invalid]")

        # Unsigned int (little-endian)
        try:
            val = struct.unpack('<I', data)[0]
            print(f"  Uint32 (LE):    {val:>15}")
        except:
            print(f"  Uint32 (LE):    [invalid]")

        # Unsigned int (big-endian)
        try:
            val = struct.unpack('>I', data)[0]
            print(f"  Uint32 (BE):    {val:>15}")
        except:
            print(f"  Uint32 (BE):    [invalid]")

        # Signed int (little-endian)
        try:
            val = struct.unpack('<i', data)[0]
            print(f"  Int32 (LE):     {val:>15}")
        except:
            print(f"  Int32 (LE):     [invalid]")

        # Signed int (big-endian)
        try:
            val = struct.unpack('>i', data)[0]
            print(f"  Int32 (BE):     {val:>15}")
        except:
            print(f"  Int32 (BE):     [invalid]")

    elif len(data) == 2:
        print("Interpretations:")
        print("-" * 40)

        # Unsigned short
        try:
            val = struct.unpack('<H', data)[0]
            print(f"  Uint16 (LE):    {val:>15}")
        except:
            pass

        try:
            val = struct.unpack('>H', data)[0]
            print(f"  Uint16 (BE):    {val:>15}")
        except:
            pass

    elif len(data) == 1:
        print("Interpretations:")
        print("-" * 40)
        print(f"  Uint8:          {data[0]:>15}")
        print(f"  Int8:           {struct.unpack('b', data)[0]:>15}")
        if 32 <= data[0] <= 126:
            print(f"  ASCII:          '{chr(data[0])}'")

    # String interpretation (if printable)
    try:
        text = data.decode('utf-8')
        if text.isprintable():
            print(f"\nUTF-8 String: \"{text}\"")
    except:
        pass

    print()


def scan_for_value(filename, target_value, value_type='float'):
    """Scan file for a specific value."""
    filepath = Path(filename)

    if not filepath.exists():
        print(f"Error: File not found: {filename}")
        sys.exit(1)

    with open(filepath, 'rb') as f:
        data = f.read()

    print(f"\n{'='*60}")
    print(f"Scanning: {filepath.name}")
    print(f"Looking for: {target_value} (as {value_type})")
    print(f"{'='*60}\n")

    matches = []

    if value_type == 'float':
        # Scan for float (little-endian)
        target_bytes = struct.pack('<f', float(target_value))
        for i in range(len(data) - 3):
            if data[i:i+4] == target_bytes:
                matches.append((i, 'float_le'))

        # Scan for float (big-endian)
        target_bytes = struct.pack('>f', float(target_value))
        for i in range(len(data) - 3):
            if data[i:i+4] == target_bytes:
                matches.append((i, 'float_be'))

    elif value_type == 'int':
        # Scan for uint32 (little-endian)
        target_bytes = struct.pack('<I', int(target_value))
        for i in range(len(data) - 3):
            if data[i:i+4] == target_bytes:
                matches.append((i, 'uint32_le'))

        # Scan for uint32 (big-endian)
        target_bytes = struct.pack('>I', int(target_value))
        for i in range(len(data) - 3):
            if data[i:i+4] == target_bytes:
                matches.append((i, 'uint32_be'))

    if matches:
        print(f"Found {len(matches)} match(es):\n")
        for offset, encoding in matches:
            hex_bytes = ' '.join(f'{b:02x}' for b in data[offset:offset+4])
            print(f"  Offset 0x{offset:04X} ({offset:>5}): {hex_bytes} [{encoding}]")
    else:
        print("No matches found.")

    print()


def main():
    parser = argparse.ArgumentParser(
        description='Analyze bytes in Logic Pro ProjectData binary files'
    )
    subparsers = parser.add_subparsers(dest='command', help='Command to run')

    # Analyze command
    analyze_parser = subparsers.add_parser('analyze', help='Analyze bytes at offset')
    analyze_parser.add_argument('file', help='Binary file to analyze')
    analyze_parser.add_argument('offset', type=lambda x: int(x, 0),
                               help='Offset (decimal or hex like 0x18B)')
    analyze_parser.add_argument('--bytes', type=int, default=4,
                               help='Number of bytes to read (default: 4)')

    # Scan command
    scan_parser = subparsers.add_parser('scan', help='Scan file for a value')
    scan_parser.add_argument('file', help='Binary file to scan')
    scan_parser.add_argument('value', help='Value to search for')
    scan_parser.add_argument('--type', choices=['float', 'int'], default='float',
                            help='Data type (default: float)')

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(1)

    if args.command == 'analyze':
        analyze_offset(args.file, args.offset, args.bytes)
    elif args.command == 'scan':
        scan_for_value(args.file, args.value, args.type)


if __name__ == '__main__':
    main()
