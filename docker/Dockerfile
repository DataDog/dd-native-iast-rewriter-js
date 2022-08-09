FROM node:16@sha256:481d28c3890e832ec54b73ec5ea74d91b91af111ec06ae3a3bcb5a81e91892f0

WORKDIR /build
COPY ./package.json .
COPY ./package-lock.json .

ENV PATH=$PATH:"$HOME/.cargo/bin"
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y

RUN npm ci --ignore-scripts
COPY . .

CMD ["npm", "run", "buildAndTest:linux"]
