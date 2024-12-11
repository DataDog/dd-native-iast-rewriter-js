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
        const result = _ddiast.plusOperator(a + " hey!", a, " hey!");
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
_ddiast.plusOperator(1 + __datadog_test_0, 1, __datadog_test_0));\n\
      }'
    )
  })

  it('does modify add inside ()', () => {
    const js = 'let c;\
const a = "a" + (c = "_b_", c + message);'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nlet c;\n\
        const a = (__datadog_test_0 = (c = "_b_", _ddiast.plusOperator(c + message, c, message)), \
_ddiast.plusOperator("a" + __datadog_test_0, "a", __datadog_test_0));\n}'
    )
  })

  it('does modify add inside OR operator (right)', () => {
    const js = 'const result = a || b + c;'
    rewriteAndExpect(
      js,
      `{
        const result = a || _ddiast.plusOperator(b + c, b, c);
    }`
    )
  })

  it('does modify add inside OR operator (left)', () => {
    const js = 'const result = a + b || c;'
    rewriteAndExpect(
      js,
      `{
        const result = _ddiast.plusOperator(a + b, a, b) || c;
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
    'does change + operator with datadog _ddiast.*plusOperator functions',
    [
      ['const result = a + b;', 'const result = _ddiast.plusOperator(a + b, a, b);'],
      [
        'const result = a + b + c;',
        'let __datadog_test_0;\n\
        const result = (__datadog_test_0 = _ddiast.plusOperator(a + b, a, b), _ddiast.plusOperator(__datadog_test_0 \
+ c, __datadog_test_0, c));'
      ]
    ],
    (value) => {
      const input = value[0]
      const expected = value[1]
      rewriteAndExpect(input, wrapBlock(expected))
    }
  )

  itEach(
    'does change + operator with datadog _ddiast.*plusOperator functions extracting local variables',
    [
      [
        'const result = a() + b();',
        'let __datadog_test_0, __datadog_test_1;\nconst result = (__datadog_test_0 = a(), __datadog_test_1 = b(), \
_ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));'
      ],
      [
        'const result = a() + b() + c();',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;\n\
const result = (__datadog_test_2 = (__datadog_test_0 = a(), __datadog_test_1 = b(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), __datadog_test_3 = c(), \
_ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3));'
      ]
    ],
    (value) => {
      const input = value[0]
      const expected = value[1]
      rewriteAndExpect(input, wrapBlock(expected))
    }
  )

  itEach(
    'does change + operator with datadog _ddiast.plusOperator function extracting mixed variables',
    [
      //
      // Literals expanding from the beginning
      //
      ['const result = "a" + b;', 'const result = _ddiast.plusOperator("a" + b, "a", b);'],
      [
        'const result = "a" + b + c;',
        'let __datadog_test_0;\n\
const result = (__datadog_test_0 = _ddiast.plusOperator("a" + b, "a", b), _ddiast.plusOperator(__datadog_test_0 + c, \
__datadog_test_0, c));'
      ],

      //
      // Literals expanding from the end
      //
      ['const result = a + "b";', 'const result = _ddiast.plusOperator(a + "b", a, "b");'],
      [
        'const result = a + b + "c";',
        'let __datadog_test_0;\n\
const result = (__datadog_test_0 = _ddiast.plusOperator(a + b, a, b), _ddiast.plusOperator(__datadog_test_0 + "c", \
__datadog_test_0, "c"));'
      ],

      //
      // Literals expanding Middle positions
      //
      [
        'const result = a + "b" + c;',
        'let __datadog_test_0;\n\
const result = (__datadog_test_0 = _ddiast.plusOperator(a + "b", a, "b"), _ddiast.plusOperator(__datadog_test_0 + c, \
__datadog_test_0, c));'
      ],
      [
        'const result = a + "b" + c + d;',
        'let __datadog_test_0, __datadog_test_1;\n\
const result = (__datadog_test_1 = (__datadog_test_0 = _ddiast.plusOperator(a + "b", a, "b"), _ddiast.plusOperator(\
__datadog_test_0 + c, __datadog_test_0, c)), _ddiast.plusOperator(__datadog_test_1 + d, __datadog_test_1, d));'
      ],

      //
      // Mix combinations
      //
      [
        'const result = a + b * c;',
        'let __datadog_test_0, __datadog_test_1;\n\
const result = (__datadog_test_0 = a, __datadog_test_1 = b * c, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));'
      ],
      [
        'const result = a * b + c;',
        'let __datadog_test_0;\n\
const result = (__datadog_test_0 = a * b, _ddiast.plusOperator(__datadog_test_0 + c, __datadog_test_0, c));'
      ],
      [
        'const result = a + b + "c" + d + e + "f";',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;\n\
const result = (__datadog_test_3 = (__datadog_test_2 = (__datadog_test_1 = (__datadog_test_0 = _ddiast.plusOperator(\
a + b, a, b), _ddiast.plusOperator(__datadog_test_0 + "c", __datadog_test_0, "c")), _ddiast.plusOperator(\
__datadog_test_1 + d, __datadog_test_1, d)), _ddiast.plusOperator(__datadog_test_2 + e, __datadog_test_2, e)), \
_ddiast.plusOperator(__datadog_test_3 + "f", __datadog_test_3, "f"));'
      ],
      [
        'const result = a + b() + "c" + d + e() + "f";',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, \
__datadog_test_5, __datadog_test_6;\n\
const result = (__datadog_test_6 = (__datadog_test_4 = (__datadog_test_3 = (__datadog_test_2 = (__datadog_test_0 = a\
, __datadog_test_1 = b(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1\
)), _ddiast.plusOperator(__datadog_test_2 + "c", __datadog_test_2, "c")), _ddiast.plusOperator(__datadog_test_3 + d, \
__datadog_test_3, d)), __datadog_test_5 = e(), _ddiast.plusOperator(__datadog_test_4 + __datadog_test_5, \
__datadog_test_4, __datadog_test_5)), _ddiast.plusOperator(__datadog_test_6 + "f", __datadog_test_6, "f"));'
      ],

      // Assignations
      ['a += b;', 'a = _ddiast.plusOperator(a + b, a, b);'],
      [
        'a += b + c;',
        'let __datadog_test_0, __datadog_test_1;\n\
a = (__datadog_test_0 = a, __datadog_test_1 = _ddiast.plusOperator(b + c, b, c), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));'
      ],
      [
        'a += b + c + d;',
        'let __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        a = (__datadog_test_1 = a, __datadog_test_2 = (__datadog_test_0 = _ddiast.plusOperator(b + c, b, c), \
_ddiast.plusOperator(__datadog_test_0 + d, __datadog_test_0, d)), _ddiast.plusOperator(__datadog_test_1 + \
__datadog_test_2, __datadog_test_1, __datadog_test_2));'
      ]
    ],
    (value) => {
      const input = value[0]
      const expected = value[1]
      rewriteAndExpect(input, wrapBlock(expected))
    }
  )

  it('does not change assignation', () => {
    const js = `let a = 0;
    a -= b;`
    rewriteAndExpectNoTransformation(js)
  })

  it('does change assignation child', () => {
    const js = `let a = 0;
    a -= b + c;`
    rewriteAndExpect(
      js,
      `{
        let a = 0;
        a -= _ddiast.plusOperator(b + c, b, c);
    }`
    )
  })

  it('does change assignation with conditional value', () => {
    const js = 'a += b ? c : d;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        a = (__datadog_test_0 = a, __datadog_test_1 = b ? c : d, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));
    }`
    )
  })

  it('does change assignation with conditional value and call', () => {
    const js = 'a += b ? c() : d;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        a = (__datadog_test_0 = a, __datadog_test_1 = b ? c() : d, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));
    }`
    )
  })

  it('does change assignation with conditional value and call and child add', () => {
    const js = 'a += b ? c(e() + f) : d;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1, __datadog_test_2;
a = (__datadog_test_1 = a, __datadog_test_2 = b ? c((__datadog_test_0 = e(), _ddiast.plusOperator(__datadog_test_0 \
+ f, __datadog_test_0, f))) : d, _ddiast.plusOperator(__datadog_test_1 + __datadog_test_2, __datadog_test_1, \
__datadog_test_2));
    }`
    )
  })

  it('does modify add inside if assignation', () => {
    const js = 'if ((result = (a + b)) > 100) {}'
    rewriteAndExpect(
      js,
      `{
        if ((result = (_ddiast.plusOperator(a + b, a, b))) > 100) {}
      }`
    )
  })

  it('does modify add inside if assignation calling a function', () => {
    const js = 'if ((result = (a + b())) > 100) {}'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        if ((result = ((__datadog_test_0 = a, __datadog_test_1 = b(), _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1)))) > 100) {}
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
          if ((result = ((__datadog_test_0 = a(), _ddiast.plusOperator(__datadog_test_0 + b, __datadog_test_0, b)))) \
> 100) {}
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
          let __datadog_test_0, __datadog_test_1, __datadog_test_2;
          if ((result = ((__datadog_test_0 = a(), _ddiast.plusOperator(__datadog_test_0 + b, __datadog_test_0, b)))) \
> 100) return (__datadog_test_1 = c, __datadog_test_2 = d(), _ddiast.plusOperator(__datadog_test_1 + __datadog_test_2\
, __datadog_test_1, __datadog_test_2));
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
          if ((result = ((__datadog_test_0 = a(), _ddiast.plusOperator(__datadog_test_0 + b, __datadog_test_0, b)))) \
> 100) {
let __datadog_test_0, __datadog_test_1;
  return (__datadog_test_0 = c, __datadog_test_1 = d(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, \
__datadog_test_0, __datadog_test_1));\n}\n}
      }`
    )
  })

  it('does modify add with typeof operand', () => {
    const js = 'const result = a + typeof a;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        const result = (__datadog_test_0 = a, __datadog_test_1 = typeof a, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })

  it('does modify add with await', () => {
    const js = 'const result = a + await fs.readFile();'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        const result = (__datadog_test_0 = a, __datadog_test_1 = await fs.readFile(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })

  it('does modify add with await and nested add', () => {
    const js = 'const result = a + await fs.readFile(c + d);'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
const result = (__datadog_test_0 = a, __datadog_test_1 = await fs.readFile(_ddiast.plusOperator(c + d, c, d)), \
_ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })

  it('does modify add with await and nested add with call', () => {
    const js = 'const result = a + await fs.readFile(c + d());'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;
        const result = (__datadog_test_2 = a, __datadog_test_3 = await fs.readFile((__datadog_test_0 = c, \
__datadog_test_1 = d(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, \
__datadog_test_1))), _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3));
      }`
    )
  })

  it('does modify add with increment last', () => {
    const js = 'const a = b + c++;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        const a = (__datadog_test_0 = b, __datadog_test_1 = c++, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })

  it('does modify add with increment first', () => {
    const js = 'const a = b++ + c;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0;
const a = (__datadog_test_0 = b++, _ddiast.plusOperator(__datadog_test_0 + c, __datadog_test_0, c));
      }`
    )
  })

  it('does modify add with double increment first', () => {
    const js = 'const a = b++ + c++;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
  const a = (__datadog_test_0 = b++, __datadog_test_1 = c++, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })

  it('does modify add with decrement last', () => {
    const js = 'const a = b + c--;'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        const a = (__datadog_test_0 = b, __datadog_test_1 = c--, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })

  it('does modify add with call and another add inside', () => {
    const js = 'const a = e + b.c(d + c--);'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;
        const a = (__datadog_test_2 = e, __datadog_test_3 = b.c((__datadog_test_0 = d, __datadog_test_1 = c--, \
_ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))), _ddiast.plusOperator(\
__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3));
      }`
    )
  })

  it('does modify add with paren', () => {
    const js = 'const a = b + (c + d);'
    rewriteAndExpect(
      js,
      `{
        let __datadog_test_0, __datadog_test_1;
        const a = (__datadog_test_0 = b, __datadog_test_1 = (_ddiast.plusOperator(c + d, c, d)), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));
      }`
    )
  })
})
