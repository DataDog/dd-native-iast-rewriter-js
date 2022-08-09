FROM node:16-alpine@sha256:38bc06c682ae1f89f4c06a5f40f7a07ae438ca437a2a04cf773e66960b2d75bc

RUN apk add py3-pip make g++ curl

WORKDIR /build
COPY ./package.json .
COPY ./package-lock.json .

ENV PATH=$PATH:"$HOME/.cargo/bin"
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y

RUN npm ci --ignore-scripts
COPY . .

CMD ["npm", "run", "buildAndTest:alpine"]
