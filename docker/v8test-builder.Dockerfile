ARG V8_BRANCH="main"
FROM node:18@sha256:89ad39c0853cb44784f1c73ea076070f0bb88212fac94e8e509086b7ee5f8b77 AS v8builder

ARG V8_BRANCH

WORKDIR /build

ENV PATH=$PATH:/build/depot_tools/
ENV PATH=$PATH:/build/v8/tools/dev/

RUN apt-get update -y \
    && apt-get install -y \
    curl \
    git \
    libglib2.0-dev \
    libxml2 \
    python3 \
    python3-pip \
    lsb-release \
    && apt-get clean;

# fetch and build v8
RUN pip3 install httplib2 six \
    && git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git --progress --verbose \
    && fetch v8 \
    && cd v8 \
    && git checkout $V8_BRANCH \
    && gclient sync \
    && tools/dev/gm.py x64.release \
    && rm -rf .git buildtools third_party;

