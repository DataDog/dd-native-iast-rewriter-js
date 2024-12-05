/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-unused-expressions */

const { rewriteAndExpectNoTransformation, rewriteAst, wrapBlock } = require('./util')

const testOptions = {
  keepPrefix: true,
  csiMethods: [{
    src: 'trim'
  }]
}

const EXPECTED_PREFIX = `;
if (typeof _ddiast === 'undefined') (function(globals) {
    const noop = (res)=>res;
    globals._ddiast = globals._ddiast || {
        trim: noop
    };
}((1, eval)('this')));`

describe('Initialization prefix', () => {
  describe('Rewrites', () => {
    it('should not add prefix when the file is not modified', () => {
      const js = 'const a = 12'

      rewriteAndExpectNoTransformation(js, testOptions)
    })

    it('should add prefix in rewritten files', () => {
      const js = 'a.trim();'
      const rewritten = rewriteAst(wrapBlock(js), testOptions)
      expect(rewritten.startsWith(EXPECTED_PREFIX)).to.be.true
    })

    it('should maintain \'use strict\' at the begining', () => {
      const js = `'use strict'
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)
      expect(rewritten).to.include(EXPECTED_PREFIX)
      expect(rewritten.startsWith('\'use strict\'')).to.be.true
    })

    it('should maintain \'use strict\' at the beginning ignoring comments', () => {
      const js = `// test
'use strict'
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)

      expect(rewritten.startsWith(`'use strict';\n${EXPECTED_PREFIX}`)).to.be.true
    })

    it('should maintain "use strict" at the begining', () => {
      const js = `"use strict"
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)
      expect(rewritten.startsWith(`"use strict";\n${EXPECTED_PREFIX}`)).to.be.true
    })
  })

  describe('Execution', () => {
    let _ddiast
    beforeEach(() => {
      _ddiast = global._ddiast
      delete global._ddiast
    })

    afterEach(() => {
      global._ddiast = _ddiast
    })

    it('should execute valid code when _ddiast is not defined yet', () => {
      const code = `(val) => {
  return val.trim()
}`
      const rewrittenCode = rewriteAst(code, testOptions)
      // eslint-disable-next-line no-eval
      const rewrittenFunction = (1, eval)(rewrittenCode)

      expect(rewrittenFunction('   test   ')).to.be.equals('test')
    })
  })
})
