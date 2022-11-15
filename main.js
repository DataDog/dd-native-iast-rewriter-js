/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
'use strict'
const { getPrepareStackTrace } = require('./js/stack-trace/')
const { cacheRewrittenSourceMap } = require('./js/source-map')

const nativeRewriter = process.env.NATIVE_REWRITER === 'true'

class DummyRewriterConfig {}

class DummyRewriter {
  rewrite (code, file) {
    return code
  }
}

let RewriterConfig = DummyRewriterConfig
let NativeRewriter
class CacheRewriter {
  constructor (config) {
    if (NativeRewriter) {
      this.nativeRewriter = new NativeRewriter(config)
    } else {
      this.nativeRewriter = new DummyRewriter()
    }
  }

  rewrite (code, file) {
    const content = this.nativeRewriter.rewrite(code, file)
    cacheRewrittenSourceMap(file, content)
    return content
  }
}

function getRewriter () {
  try {
    const iastRewriter = nativeRewriter ? require('node-gyp-build')(__dirname) : require('./wasm/native_iast_rewriter')
    NativeRewriter = iastRewriter.Rewriter
    if (!nativeRewriter) {
      RewriterConfig = iastRewriter.RewriterConfig
    }

    return CacheRewriter
  } catch (e) {
    return DummyRewriter
  }
}

module.exports = {
  Rewriter: getRewriter(),
  RewriterConfig: RewriterConfig,
  getPrepareStackTrace: getPrepareStackTrace
}
