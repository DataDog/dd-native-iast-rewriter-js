/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-unused-expressions */

const { rewriteAndExpectNoTransformation, rewriteAst, wrapBlock } = require('./util')

const testOptions = {
  keepPrefix: true,
  csiMethods: [
    {
      src: 'trim'
    }
  ]
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

    it('should add prefix in rewritten files in ESM modules', () => {
      const js = 'import { a } from "a"; { a.trim() }'
      const rewritten = rewriteAst(js, testOptions)
      expect(rewritten.startsWith(EXPECTED_PREFIX)).to.be.true
    })

    it("should maintain 'use strict' at the beginning", () => {
      const js = `'use strict'
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)
      expect(rewritten).to.include(EXPECTED_PREFIX)
      expect(rewritten.startsWith("'use strict'")).to.be.true
    })

    it("should maintain 'use strict' at the beginning in ESM modules", () => {
      const js = `'use strict'
import { a } from "a";
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)
      expect(rewritten).to.include(EXPECTED_PREFIX)
      expect(rewritten.startsWith("'use strict'")).to.be.true
    })

    it("should maintain 'use strict' at the beginning ignoring comments", () => {
      const js = `// test
'use strict'
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)

      expect(rewritten.startsWith(`'use strict';\n${EXPECTED_PREFIX}`)).to.be.true
    })

    it('should maintain "use strict" at the beginning', () => {
      const js = `"use strict"
function a() { a.trim() }`
      const rewritten = rewriteAst(js, testOptions)
      expect(rewritten.startsWith(`"use strict";\n${EXPECTED_PREFIX}`)).to.be.true
    })

    it('should maintain "use strict" if it comes after /**/ comment and {comments: true} in config', () => {
      const comment = `/* this is a
 * multiline comment
 */`
      const js = `${comment}
"use strict"
function a() { a.trim() }`
      const rewritten = rewriteAst(js, { ...testOptions, comments: true })

      expect(rewritten.startsWith(`${comment} "use strict";\n${EXPECTED_PREFIX}`)).to.be.true
    })

    it('should maintain "use strict" if it comes after // comment and {comments: true} in config', () => {
      const comment = '// this is a comment'
      const js = `${comment}
"use strict"
function a() { a.trim() }`
      const rewritten = rewriteAst(js, { ...testOptions, comments: true })

      expect(rewritten.startsWith(`${comment}\n"use strict";\n${EXPECTED_PREFIX}`)).to.be.true
    })

    it('should maintain "use strict" if it comes before // comment and { comments: true } in config', () => {
      const comment = '// this is a comment'
      const js = `"use strict"
${comment}
function a() { a.trim() }`

      const rewritten = rewriteAst(js, { ...testOptions, comments: true })

      expect(rewritten.startsWith(`"use strict";\n${EXPECTED_PREFIX}\n${comment}`)).to.be.true
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

    it('should execute valid code when _ddiast is not defined yet and src and dst are differents', () => {
      const code = `(val) => {
  return val.trim()
}`
      const newTestOptions = {
        ...testOptions,
        csiMethods: [
          {
            src: 'trim',
            dst: 'modifiedTrim'
          }
        ]
      }
      const rewrittenCode = rewriteAst(code, newTestOptions)
      // eslint-disable-next-line no-eval
      const rewrittenFunction = (1, eval)(rewrittenCode)

      expect(rewrittenFunction('   test   ')).to.be.equals('test')
    })
  })
})
