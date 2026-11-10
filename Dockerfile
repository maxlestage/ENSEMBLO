# Le site d'Ensemblo (site/), une app Paquets : serveur Rust, îlots WebAssembly et 3D.
# Pour Heroku (heroku.yml, pile « container ») et tout hôte Docker.
#
#   docker build -t ensemblo-site . && docker run --rm -p 8080:8080 ensemblo-site
#
# Paquets et ses paquets (Vitesse, Securite, Memoire, Active) viennent de dépôts GitHub
# privés : passez un jeton qui peut les lire, `--build-arg REPOS_TOKEN=…`. Il ne sert
# qu'à l'étape de compilation et n'est pas copié dans l'image finale.

FROM rust:1-slim-trixie AS build
ARG REPOS_TOKEN=""
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates git \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN if [ -n "$REPOS_TOKEN" ]; then \
      git config --global credential.https://github.com.helper \
        '!f() { test "$1" = get && echo username=x-access-token && echo "password=$REPOS_TOKEN"; }; f'; \
    fi
WORKDIR /src
COPY site ./site
WORKDIR /src/site
# Les îlots en WebAssembly, puis wasm-bindgen à la version de Cargo.lock (un binaire
# publié, ou compilé s'il ne se télécharge pas).
RUN cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && (curl -sSfL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${version}/wasm-bindgen-${version}-x86_64-unknown-linux-musl.tar.gz" \
            | tar -xz --strip-components=1 -C /usr/local/bin \
        || cargo install wasm-bindgen-cli --version "$version" --locked) \
    && wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
        --out-dir pkg target/wasm32-unknown-unknown/wasm-release/ensemblo_site.wasm
# Le serveur.
RUN cargo build --release --bin ensemblo-site && cp target/release/ensemblo-site /server

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 ensemblo
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY --from=build /src/site/public ./public
COPY --from=build /src/site/pkg ./pkg
USER ensemblo
# Heroku donne PORT au démarrage ; 8080 ailleurs.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
