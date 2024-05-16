echo "Compilation ..."
cargo build -r --target x86_64-unknown-linux-musl

echo "Création de l'image ..."
docker build . --tag=voiturerc/proxy

echo "Terminé."