/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */
const { itEach } = require('mocha-it-each')

const { rewriteAndExpectNoTransformation, rewriteAndExpect, rewriteAndExpectError, wrapBlock } = require('./util')

describe('binary expression', () => {
  it('does not modify sub', () => {
    const js = 'const result = a - " hey!";'
    rewriteAndExpectNoTransformation(js)
  })

  it('does modify add', () => {
    const js = 'const result = a + " hey!";'
    rewriteAndExpect(
      js,
      `{
        const result = global._ddiast.plusOperator(a + " hey!", a, " hey!");
    }`
    )
  })

  it('does not modify parameters of other functions when literals', () => {
    const js = 'const result = 1 + otherMethod(2);'
    rewriteAndExpect(
      js,
      '{\n\
        let __datadog_test_0;\n\
const result = (__datadog_test_0 = otherMethod(2), \
global._ddiast.plusOperator(1 + __datadog_test_0, 1, __datadog_test_0));\n\
      }'
    )
  })

  it('does modify add inside ()', () => {
    const js = 'let c;\
const a = "a" + (c = "_b_", c + message);'
    rewriteAndExpect(
      js,
      '{\n    let __datadog_test_0;\n\
let c;\n    const a = (__datadog_test_0 = (c = "_b_", global._ddiast.plusOperator(c + message, c, message)), \
global._ddiast.plusOperator("a" + __datadog_test_0, "a", __datadog_test_0));\n}'
    )
  })

  it('does modify add inside OR operator (right)', () => {
    const js = 'const result = a || b + c;'
    rewriteAndExpect(
      js,
      `{
        const result = a || global._ddiast.plusOperator(b + c, b, c);
    }`
    )
  })

  it('does modify add inside OR operator (left)', () => {
    const js = 'const result = a + b || c;'
    rewriteAndExpect(
      js,
      `{
        const result = global._ddiast.plusOperator(a + b, a, b) || c;
    }`
    )
  })

  itEach(
    'does not change sum of literals',
    [
      'const result = "a" + "b";',
      'const result = "a" + "b" + "c";',
      'const result = "a" + "b" + "c" + "d";',
      'const result = "a" + "b" + "c" + "d" + "e";',
      'const result = "a" + "b" + "c" + "d" + "e" + "f";'
    ],
    (js) => {
      rewriteAndExpectNoTransformation(js)
    }
  )

  itEach(
    'does change + operator with datadog global._ddiast.*plusOperator functions',
    [
      ['const result = a + b;', 'const result = global._ddiast.plusOperator(a + b, a, b);'],
      ['const result = a + b + c;', 'const result = global._ddiast.plusOperator(a + b + c, a, b, c);'],
      ['const result = a + b + c + d;', 'const result = global._ddiast.plusOperator(a + b + c + d, a, b, c, d);'],
      [
        'const result = a + b + c + d + e;',
        'const result = global._ddiast.plusOperator(a + b + c + d + e, a, b, c, d, e);'
      ],
      [
        'const result = a + b + c + d + e + f;',
        'const result = global._ddiast.plusOperator(a + b + c + d + e + f, a, b, c, d, e, f);'
      ]
    ],
    (value) => {
      const input = value[0]
      const expected = value[1]
      rewriteAndExpect(input, wrapBlock(expected))
    }
  )

  itEach(
    'does change + operator with datadog global._ddiast.*plusOperator functions extracting local variables',
    [
      [
        'const result = a() + b();',
        'let __datadog_test_0, __datadog_test_1;\n\
      const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), \
global._ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));'
      ],
      [
        'const result = a() + b() + c();',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), \
global._ddiast.plusOperator(__datadog_test_0 + __datadog_test_1 + __datadog_test_2, \
__datadog_test_0, __datadog_test_1, __datadog_test_2));'
      ],
      [
        'const result = a() + b() + c() + d();',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;\n\
      const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), __datadog_test_3 = d(), \
global._ddiast.plusOperator(__datadog_test_0 + __datadog_test_1 + __datadog_test_2 + __datadog_test_3, \
__datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3));'
      ],
      [
        'const result = a() + b() + c() + d() + e();',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4;\n\
const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), __datadog_test_3 = d(), \
__datadog_test_4 = e(), global._ddiast.plusOperator(__datadog_test_0 + __datadog_test_1 + __datadog_test_2 + \
__datadog_test_3 + __datadog_test_4, __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, \
__datadog_test_4));'
      ],
      [
        'const result = a() + b() + c() + d() + e() + f();',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, \
__datadog_test_5;\n\
const result = (__datadog_test_0 = a(), __datadog_test_1 = b(), __datadog_test_2 = c(), __datadog_test_3 = d(), \
__datadog_test_4 = e(), __datadog_test_5 = f(), global._ddiast.plusOperator(__datadog_test_0 + __datadog_test_1 + \
__datadog_test_2 + __datadog_test_3 + __datadog_test_4 + __datadog_test_5, __datadog_test_0, __datadog_test_1, \
__datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5));'
      ]
    ],
    (value) => {
      const input = value[0]
      const expected = value[1]
      rewriteAndExpect(input, wrapBlock(expected))
    }
  )

  itEach(
    'does change + operator with datadog global._ddiast.plusOperator function extracting mixed variables',
    [
      //
      // Literals expanding from the beginning
      //
      ['const result = "a" + b;', 'const result = global._ddiast.plusOperator("a" + b, "a", b);'],
      ['const result = "a" + b + c;', 'const result = global._ddiast.plusOperator("a" + b + c, "a", b, c);'],
      ['const result = "a" + b + c + d;', 'const result = global._ddiast.plusOperator("a" + b + c + d, "a", b, c, d);'],
      [
        'const result = "a" + b + c + d + e;',
        'const result = global._ddiast.plusOperator("a" + b + c + d + e, "a", b, c, d, e);'
      ],
      [
        'const result = "a" + b + c + d + e + f;',
        'const result = global._ddiast.plusOperator("a" + b + c + d + e + f, "a", b, c, d, e, f);'
      ],

      //
      // Literals expanding from the end
      //
      ['const result = a + "b";', 'const result = global._ddiast.plusOperator(a + "b", a, "b");'],
      ['const result = a + b + "c";', 'const result = global._ddiast.plusOperator(a + b + "c", a, b, "c");'],
      ['const result = a + b + c + "d";', 'const result = global._ddiast.plusOperator(a + b + c + "d", a, b, c, "d");'],
      [
        'const result = a + b + c + d + "e";',
        'const result = global._ddiast.plusOperator(a + b + c + d + "e", a, b, c, d, "e");'
      ],

      //
      // Literals expanding Middle positions
      //
      ['const result = a + "b" + c;', 'const result = global._ddiast.plusOperator(a + "b" + c, a, "b", c);'],
      ['const result = a + "b" + c + d;', 'const result = global._ddiast.plusOperator(a + "b" + c + d, a, "b", c, d);'],
      ['const result = a + b + "c" + d;', 'const result = global._ddiast.plusOperator(a + b + "c" + d, a, b, "c", d);'],

      //
      // Mix combinations
      //
      ['const result = a + b * c;', 'const result = global._ddiast.plusOperator(a + b * c, a, b * c);'],
      ['const result = a * b + c;', 'const result = global._ddiast.plusOperator(a * b + c, a * b, c);'],
      [
        'const result = a + b + "c" + d + e + "f";',
        'const result = global._ddiast.plusOperator(a + b + "c" + d + e + "f", a, b, "c", d, e, "f");'
      ],
      [
        'const result = a + b() + "c" + d + e() + "f";',
        'let __datadog_test_0, __datadog_test_1;\n\
const result = (__datadog_test_0 = b(), __datadog_test_1 = e(), global._ddiast.plusOperator(a + \
__datadog_test_0 + "c" + d + __datadog_test_1 + "f", a, __datadog_test_0, "c", d, __datadog_test_1, "f"));'
      ],

      // Assignations
      ['a += b;', 'a = global._ddiast.plusOperator(a + b, a, b);'],
      ['a += b + c;', 'a = global._ddiast.plusOperator(a + b + c, a, b, c);'],
      ['a += b + c + d;', 'a = global._ddiast.plusOperator(a + b + c + d, a, b, c, d);'],
      ['a += b + c + d + e;', 'a = global._ddiast.plusOperator(a + b + c + d + e, a, b, c, d, e);'],
      ['a += b + c + d + e + f;', 'a = global._ddiast.plusOperator(a + b + c + d + e + f, a, b, c, d, e, f);']
    ],
    (value) => {
      const input = value[0]
      const expected = value[1]
      rewriteAndExpect(input, wrapBlock(expected))
    }
  )

  it('does modify add inside if assignation', () => {
    const js = 'if ((result = (a + b)) > 100) {}'
    rewriteAndExpect(
      js,
      `{
        if ((result = (global._ddiast.plusOperator(a + b, a, b))) > 100) {}
      }`
    )
  })

  it('does modify add inside if assignation calling a function', () => {
    const js = 'if ((result = (a + b())) > 100) {}'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0;
        if ((result = ((__datadog_test_0 = b(), global._ddiast.plusOperator(a + __datadog_test_0, a\
, __datadog_test_0)))) > 100) {}
      }`
    )
  })

  it('does modify add and variable declaration is in the correct block', () => {
    const js = `function a(){}
      function b(){if ((result = (a() + b)) > 100) {}}`
    rewriteAndExpect(
      js,
      `{
        function a() {}
        function b() {
        let __datadog_test_0;
        if ((result = ((__datadog_test_0 = a(), global._ddiast.plusOperator(__datadog_test_0 + b\
, __datadog_test_0, b)))) > 100) {}
        }
      }`
    )
  })

  it('does fail rewrite if duplicate variable name is found', () => {
    const js = 'const __datadog_test_0 = 0; const c = a + b();'
    rewriteAndExpectError(js)
  })

  it('does fail rewrite if duplicate variable name is found inside a child function', () => {
    const js = 'const __datadog_test_0 = 0; function z(){const c = a + b();}'
    rewriteAndExpectError(js)
  })

  it('does fail rewrite if duplicate variable name is found in a function parameter', () => {
    const js = 'const a = 0; function z(__datadog_test_0){const c = a + b();}'
    rewriteAndExpectError(js)
  })

  it('does not fail rewrite if duplicate variable name is found in a function parameter', () => {
    const js = 'const a = b() + c; function z(__datadog_test_0){const d = a + c;}'
    rewriteAndExpectError(js)
  })

  it('does not fail rewrite if duplicate variable name is found inside a child function', () => {
    const js = 'const a = b() + c; function z(){const __datadog_test_0 = a + c;}'
    rewriteAndExpectError(js)
  })

  it('does modify add in a "if" clause without block', () => {
    const js = `
      function b(){if ((result = (a() + b)) > 100) return c + d();}`
    rewriteAndExpect(
      js,
      `{
        function b() {
          let __datadog_test_0, __datadog_test_1;
        if ((result = ((__datadog_test_0 = a(), global._ddiast.plusOperator(__datadog_test_0 + b\
, __datadog_test_0, b)))) > 100) return (__datadog_test_1 = d(), global._ddiast.plusOperator(c + \
__datadog_test_1, c, __datadog_test_1));
        }
      }`
    )
  })

  it('does modify add in a "if" clause with block', () => {
    const js = `
      function b(){if ((result = (a() + b)) > 100) {return c + d();}}`
    rewriteAndExpect(
      js,
      `{
        function b() {
          let __datadog_test_0;
        if ((result = ((__datadog_test_0 = a(), global._ddiast.plusOperator(__datadog_test_0 + b\
, __datadog_test_0, b)))) > 100) {\nlet __datadog_test_0;\nreturn (__datadog_test_0 = d(), \
global._ddiast.plusOperator(c + __datadog_test_0, c, __datadog_test_0));\n}
        }
      }`
    )
  })

  it('does modify add with typeof operand', () => {
    const js = 'const result = a + typeof a;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0;
        const result = (__datadog_test_0 = typeof a, global._ddiast.plusOperator(a + __datadog_test_0, \
a, __datadog_test_0));
      }`
    )
  })
})
