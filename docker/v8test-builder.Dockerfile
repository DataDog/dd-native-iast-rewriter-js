ARG V8_BRANCH="main"
FROM node:16@sha256:481d28c3890e832ec54b73ec5ea74d91b91af111ec06ae3a3bcb5a81e91892f0 AS v8builder

WORKDIR /build

ENV PATH=$PATH:/build/depot_tools/
ENV V8_BRANCH_ENV=$V8_BRANCH

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
    && git checkout $V8_BRANCH_ENV \
    && gclient sync \
    && tools/dev/gm.py x64.release;

