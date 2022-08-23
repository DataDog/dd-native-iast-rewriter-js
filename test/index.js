/* eslint-disable no-multi-str */

const { expect } = require('chai')
const { it, describe } = require('mocha')
const { itEach } = require('mocha-it-each')
const os = require('os')
const path = require('path')

const { Rewriter } = require('../index')

const rewriteAst = (code, opts) => {
  opts = opts || {}
  const rewriter = opts.rewriter ?? new Rewriter()
  const file = opts.file ?? path.join(process.cwd(), 'index.js')
  const sourceMap = opts.sourceMap
  return rewriter.rewrite(code, file, sourceMap)
}

const wrapBlock = (code) => `{${os.EOL}${code}${os.EOL}}`

const rewriteAndExpectNoTransformation = (code) => {
  rewriteAndExpect(wrapBlock(code), wrapBlock(code), true)
}

const rewriteAndExpect = (code, expect, block) => {
  code = !block ? `{${code}}` : code
  const rewrited = rewriteAst(code)
  expectAst(rewrited, expect)
}

const expectAst = (received, expected) => {
  const rLines = received.split(os.EOL).map(l => l.trim()).join(os.EOL)
  const eLines = expected.split(os.EOL).map(l => l.trim()).join(os.EOL)

  expect(rLines).to.be.eq(eLines)
}

describe('binary expression', () => {
  it('does not modify sub', () => {
    const js = 'const result = a - " hey!";'
    rewriteAndExpectNoTransformation(js)
  })

  it('does modify add', () => {
    const js = 'const result = a + " hey!";'
    rewriteAndExpect(js, `{
        const result = global._ddiast.twoItemsPlusOperator(a + " hey!", a, " hey!");
    }`)
  })

  it('does not modify parameters of other functions when literals', () => {
    const js = 'const result = 1 + otherMethod(2);'
    rewriteAndExpect(js, '{\n\
        let __datadog_test_0;\n\
const result = (__datadog_test_0 = otherMethod(2), \
global._ddiast.twoItemsPlusOperator(1 + __datadog_test_0, 1, __datadog_test_0));\n\
      }')
  })

  itEach('does not change sum of literals', [
    'const result = "a" + "b";',
    'const result = "a" + "b" + "c";',
    'const result = "a" + "b" + "c" + "d";',
    'const result = "a" + "b" + "c" + "d" + "e";',
    'const result = "a" + "b" + "c" + "d" + "e" + "f";'
  ], (js) => {
    rewriteAndExpectNoTransformation(js)
  })

  itEach('does change + operator with datadog global._ddiast.*plusOperator functions', [
    ['const result = a + b;', 'const result = global._ddiast.twoItemsPlusOperator(a + b, a, b);'],
    [
      'const result = a + b + c;',
      'const result = global._ddiast.threeItemsPlusOperator(a + b + c, a, b, c);'],
    [
      'const result = a + b + c + d;',
      'const result = global._ddiast.fourItemsPlusOperator(a + b + c + d, a, b, c, d);'],
    [
      'const result = a + b + c + d + e;',
      'const result = global._ddiast.fiveItemsPlusOperator(a + b + c + d + e, a, b, c, d, e);'
    ],
    [
      'const result = a + b + c + d + e + f;',
      'const result = global._ddiast.anyPlusOperator(a + b + c + d + e + f, a, b, c, d, e, f);'
    ]
  ], (value) => {
    const input = value[0]
    const expected = value[1]
    rewriteAndExpect(input, wrapBlock(expected))
  })

  itEach('does change + operator with datadog global._ddiast.*plusOperator functions extracting local variables', [
    ['const result = a() + b();', 'let __datadog_test_0, __datadog_test_1;\n\
      const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), \
global._ddiast.twoItemsPlusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));'],
    [
      'const result = a() + b() + c();',
      'let __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), \
global._ddiast.threeItemsPlusOperator(__datadog_test_0 + __datadog_test_1 + __datadog_test_2, \
__datadog_test_0, __datadog_test_1, __datadog_test_2));'
    ],
    [
      'const result = a() + b() + c() + d();',
      'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;\n\
      const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), __datadog_test_3 = d(), \
global._ddiast.fourItemsPlusOperator(__datadog_test_0 + __datadog_test_1 + __datadog_test_2 + __datadog_test_3, \
__datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3));'
    ],
    [
      'const result = a() + b() + c() + d() + e();',
      'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4;\n\
const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), __datadog_test_3 = d(), \
__datadog_test_4 = e(), global._ddiast.fiveItemsPlusOperator(__datadog_test_0 + __datadog_test_1 + __datadog_test_2 + \
__datadog_test_3 + __datadog_test_4, __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, \
__datadog_test_4));'
    ],
    [
      'const result = a() + b() + c() + d() + e() + f();',
      'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, \
__datadog_test_5;\n\
const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), __datadog_test_3 = d(), \
__datadog_test_4 = e(), __datadog_test_5 = f(), global._ddiast.anyPlusOperator(__datadog_test_0 + __datadog_test_1 + \
__datadog_test_2 + __datadog_test_3 + __datadog_test_4 + __datadog_test_5, __datadog_test_0, __datadog_test_1, \
__datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5));'
    ]
  ], value => {
    const input = value[0]
    const expected = value[1]
    rewriteAndExpect(input, wrapBlock(expected))
  })

  itEach('does change + operator with datadog global._ddiast.*plusOperator functions extracting mixed variables', [
    //
    // Literals expanding from the beginning
    //
    ['const result = "a" + b;', 'const result = global._ddiast.twoItemsPlusOperator("a" + b, "a", b);'],
    ['const result = "a" + b + c;', 'const result = global._ddiast.threeItemsPlusOperator("a" + b + c, "a", b, c);'],
    [
      'const result = "a" + b + c + d;',
      'const result = global._ddiast.fourItemsPlusOperator("a" + b + c + d, "a", b, c, d);'
    ],
    [
      'const result = "a" + b + c + d + e;',
      'const result = global._ddiast.fiveItemsPlusOperator("a" + b + c + d + e, "a", b, c, d, e);'
    ],
    [
      'const result = "a" + b + c + d + e + f;',
      'const result = global._ddiast.anyPlusOperator("a" + b + c + d + e + f, "a", b, c, d, e, f);'
    ],
    ['const result = a + b * c;', 'const result = global._ddiast.twoItemsPlusOperator(a + b * c, a, b * c);'],
    ['const result = a * b + c;', 'const result = global._ddiast.twoItemsPlusOperator(a * b + c, a * b, c);'],

    // Assignations
    ['a += b;', 'a = global._ddiast.twoItemsPlusOperator(a + b, a, b);'],
    [
      'a += b + c;',
      'a = global._ddiast.threeItemsPlusOperator(a + b + c, a, b, c);'
    ],
    [
      'a += b + c + d;',
      'a = global._ddiast.fourItemsPlusOperator(a + b + c + d, a, b, c, d);'
    ],
    [
      'a += b + c + d + e;',
      'a = global._ddiast.fiveItemsPlusOperator(a + b + c + d + e, a, b, c, d, e);'
    ],
    [
      'a += b + c + d + e + f;',
      'a = global._ddiast.anyPlusOperator(a + b + c + d + e + f, a, b, c, d, e, f);'
    ]
  ], value => {
    const input = value[0]
    const expected = value[1]
    rewriteAndExpect(input, wrapBlock(expected))
  })
})
