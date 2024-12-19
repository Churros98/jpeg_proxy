import socket
import struct
import time
import sys

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python mjpeg_test.py <uuid> <image_path>")
        sys.exit(1)

    print(f"Streaming with uuid {sys.argv[1]} ...")

    with open(sys.argv[2], "rb") as image:
        image_data = image.read()
        size = struct.pack("<Q", len(image_data))

        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect(("192.168.1.15", 1337))
            s.sendall(sys.argv[1].encode("utf-8"))
            while True:
                s.sendall(size)
                s.sendall(image_data)
                time.sleep(1 / 30)
