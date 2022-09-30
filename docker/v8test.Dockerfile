FROM node:16@sha256:481d28c3890e832ec54b73ec5ea74d91b91af111ec06ae3a3bcb5a81e91892f0 AS v8builder

WORKDIR /build

ENV PATH=$PATH:/build/depot_tools/
ENV V8_BRANCH="main"

RUN apt-get update -y \
    && apt-get install -y \
    curl \
    git \
    libglib2.0-dev \
    libxml2 \
    python3 \
    python3-pip;

# fetch and build v8
RUN pip3 install httplib2 six \
    && git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git --progress --verbose \
    && fetch v8 \
    && cd v8 \
    && git checkout $V8_BRANCH \
    && gclient sync \
    && tools/dev/gm.py x64.release;

FROM node:16@sha256:481d28c3890e832ec54b73ec5ea74d91b91af111ec06ae3a3bcb5a81e91892f0 AS v8tester
WORKDIR /test
RUN mkdir v8
COPY --from=v8builder /build/v8/test /test/v8/test
# asm tests exclude
RUN rm -rf /test/v8/test/mjsunit/asm
COPY --from=v8builder /build/v8/out /test/v8/out
COPY --from=v8builder /build/v8/tools /test/v8/tools

# copy rewriter npm packages and extract them
COPY ./datadog-native-iast-rewriter* /test/
RUN cat *.tgz | tar zxvf - -i
COPY ./scripts/crawler.js /test/package/scripts/crawler.js

# rewrite v8 mjsunit test files
RUN node package/scripts/crawler.js --override v8/test/mjsunit/ '^(str|arr|arg|num|rege|mod|glob|obj|val|whit|this|throw|try|unbox|pro|call|code|comp|func|for|field).*'

# launch mjsunit tests
CMD ["/test/v8/tools/run-tests.py", "--outdir=out/x64.release", "mjsunit"]
