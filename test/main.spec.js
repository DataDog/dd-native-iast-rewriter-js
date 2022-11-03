/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str, no-unused-expressions */

'use strict'

const { expect } = require('chai')
const proxyquire = require('proxyquire')

function requireMain () {
  return proxyquire('../main', {
    'node-gyp-build': function () {
      throw new Error()
    }
  })
}

describe('main', () => {
  describe('if node-gyp-build cannot load addon', () => {
    it('does not throw Error', () => {
      const Rewriter = requireMain().Rewriter
      expect(Rewriter).to.not.be.null
    })

    it('returns a Rewriter constructor', () => {
      const Rewriter = requireMain().Rewriter
      expect(Rewriter).to.be.an('function')
      const rewriter = new Rewriter()
      expect(rewriter.rewrite).to.not.be.null
    })

    it('returns original code when rewrite is invoked', () => {
      const Rewriter = requireMain().Rewriter
      const js = 'function() { return a + b }'
      const rewriter = new Rewriter()
      expect(rewriter.rewrite(js)).equal(js)
    })

    it('returns getPrepareStackTrace function', () => {
      const getPrepareStackTrace = requireMain().getPrepareStackTrace
      expect(getPrepareStackTrace).to.not.be.null
      expect(getPrepareStackTrace).to.be.an('function')
    })
  })
})
