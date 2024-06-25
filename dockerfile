from rust:1.63.0

workdir /app

run sudo apt update && apt install lld clang -y
