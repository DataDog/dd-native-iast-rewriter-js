/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
'use strict'

const { getPrepareStackTrace } = require('./js/stack-trace/')

class DummyRewriter {
  rewrite (code, file) {
    return code
  }
}

function getRewriter () {
  try {
    return require('node-gyp-build')(__dirname).Rewriter
  } catch (e) {
    return DummyRewriter
  }
}

module.exports = {
  Rewriter: getRewriter(),
  getPrepareStackTrace: getPrepareStackTrace
}
