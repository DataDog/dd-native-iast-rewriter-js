ARG NPM_VERSION
ARG REWRITER_MUSL=./datadog-native-iast-rewriter-linux-x64-musl-$NPM_VERSION.tgz
ARG REWRITER=./datadog-native-iast-rewriter-$NPM_VERSION.tgz

FROM node:16-alpine@sha256:38bc06c682ae1f89f4c06a5f40f7a07ae438ca437a2a04cf773e66960b2d75bc

RUN apk add py3-pip make g++ curl

WORKDIR /test
COPY ./package.json .
COPY ./package-lock.json .


COPY ${REWRITER_MUSL} .
COPY ${REWRITER} .

ENV PATH=$PATH:/root/.cargo/bin
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y

RUN npm ci --ignore-scripts; \
  npm i --verbose ${REWRITER_MUSL}; \
  npm i --verbose ${REWRITER};
  
COPY . .

ENV NPM_REWRITER=true

CMD ["npm", "t"]
