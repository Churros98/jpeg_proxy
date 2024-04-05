import socket
import struct

if __name__ == "__main__":
    print("Test d'envoi de jpeg au serveur ...")
    
    with open("no_signal.jpg", "rb") as image:
        image_data = image.read()
        size = struct.pack("<Q", len(image_data))

        print(f"Taille: {size}")

        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect(("172.31.128.1", 1337))
            s.sendall(size)
            s.sendall(image_data)

    print("Envoyé.")