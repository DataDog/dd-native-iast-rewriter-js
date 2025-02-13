
ARG V8_BRANCH="main"
FROM v8builder:${V8_BRANCH}

WORKDIR /build

# asm tests excluded
RUN rm -rf /build/v8/test/mjsunit/asm

# copy rewriter npm packages and extract them
COPY ./datadog-wasm-js-rewriter* /build/
RUN cat *.tgz | tar zxvf - -i
COPY ./scripts/crawler.js /build/package/scripts/crawler.js
RUN npm i --prefix package

# rewrite v8 mjsunit test files
RUN node package/scripts/crawler.js --override v8/test/mjsunit/ '^(str|arr|arg|num|rege|mod|glob|obj|val|whit|this|throw|try|unbox|pro|call|code|comp|func|for|field|substr|unicode).*'

# launch mjsunit tests
CMD ["/build/v8/tools/run-tests.py", "--outdir=out/x64.release", "mjsunit"]
