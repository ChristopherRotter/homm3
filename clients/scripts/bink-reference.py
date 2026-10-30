#!/usr/bin/env python3
"""FFmpeg's decoding of a Bink file as raw YUV 4:2:0 (E17 M5 T4b2a): the reference that
`h3-view video --reference` compares `h3-bink` with (`clients/scripts/bink-check.sh`).

Through PyAV (`pip install av`: FFmpeg's libraries; its Bink decoder is FFmpeg 6.1.1's, the
version ported, unchanged but for includes), each frame copied out and released before the next
is decoded. The `ffmpeg` command line keeps the frame before while it decodes the next; FFmpeg's
decoder then decodes into a new buffer with only the picture's rows copied (`ff_reget_buffer`),
so the block grid's rows below the picture (the chroma's rows 60 to 63 in the game's 256 x 120
clips) hold another frame's pixels, which motion vectors copy into the picture: three of the
game's ten clips then differ in a few pixels (E17-log, M5 T4b2a).

A packet FFmpeg refuses (an empty one, one its decoder fails on) is named on standard error and
skipped, as the `ffmpeg` command line goes on past it; the frames decoded are still written, and
the exit status is then 1; a file FFmpeg cannot open is named and fails the same way (E17 M5
T8b2, the leftover of review R2b).

    python3 -I clients/scripts/bink-reference.py <in.bik> <out.yuv>
"""

import sys

import av


def picture(frame):
    """The frame's planes, each picture row by row, without FFmpeg's row padding."""
    out = bytearray()
    sizes = [(frame.width, frame.height)] + [((frame.width + 1) // 2, (frame.height + 1) // 2)] * 2
    for plane, (width, height) in zip(frame.planes, sizes):
        data = bytes(plane)
        for y in range(height):
            out += data[y * plane.line_size : y * plane.line_size + width]
    return bytes(out)


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: bink-reference.py <in.bik> <out.yuv>")
    try:
        container = av.open(sys.argv[1])
    except av.error.FFmpegError as e:
        sys.exit(f"{sys.argv[1]}: FFmpeg refused the file: {e}")
    stream = container.streams.video[0]
    stream.thread_type = "NONE"
    refused = 0
    with open(sys.argv[2], "wb") as out:
        packets = container.demux(stream)
        n = 0
        while True:
            try:
                packet = next(packets)
            except StopIteration:
                break
            except av.error.FFmpegError as e:
                print(f"{sys.argv[1]}: the demuxer stopped at packet {n}: {e}", file=sys.stderr)
                refused += 1
                break
            try:
                for frame in packet.decode():
                    out.write(picture(frame))
                    del frame
            except av.error.FFmpegError as e:
                print(f"{sys.argv[1]}: packet {n}: FFmpeg refused it: {e}", file=sys.stderr)
                refused += 1
            n += 1
    container.close()
    if refused:
        sys.exit(1)


if __name__ == "__main__":
    main()
