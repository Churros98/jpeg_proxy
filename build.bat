echo "Compilation ..."
cargo build -r --target x86_64-unknown-linux-musl

echo "Création de l'image ..."
docker build . --tag=voiturerc/proxy

echo "Sauvegarde de l'image en .tar ..."
docker save -o ./vrc_proxy_dkimg.tar voiturerc/proxy:latest

echo "Terminé."