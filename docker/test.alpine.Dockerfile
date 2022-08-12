FROM node:16-alpine@sha256:38bc06c682ae1f89f4c06a5f40f7a07ae438ca437a2a04cf773e66960b2d75bc

ARG NPM_VERSION
ENV REWRITER_MUSL=datadog-native-iast-rewriter-linux-x64-musl-${NPM_VERSION}.tgz
ENV REWRITER=datadog-native-iast-rewriter-${NPM_VERSION}.tgz

RUN apk add py3-pip make g++ curl

WORKDIR /test
COPY ./package.json .
COPY ./package-lock.json .
COPY ./${REWRITER_MUSL} .
COPY ./${REWRITER} .

RUN npm ci --ignore-scripts
RUN npm i --verbose ${REWRITER_MUSL}
RUN npm i --verbose ${REWRITER}

COPY . .

ENV NPM_REWRITER=true

CMD ["npm", "t"]
