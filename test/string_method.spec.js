/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */

const { itEach } = require('mocha-it-each')

const { rewriteAndExpectNoTransformation, rewriteAndExpect } = require('./util')

describe('String method', () => {
  describe('substring', () => {
    it('does not modify literal substring', () => {
      const js = '"a".substring(1);'
      rewriteAndExpectNoTransformation(js)
    })

    it('does modify substring', () => {
      const js = 'a.substring(1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
  (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));\n}`
      )
    })

    it('does modify substring with 2 arguments', () => {
      const js = 'a.substring(1, a.lenght - 2);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = a.lenght - 2, \
_ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 1, __datadog_test_2), __datadog_test_1, \
__datadog_test_0, 1, __datadog_test_2));\n}`
      )
    })

    it('does modify substring after call', () => {
      const js = 'a().substring(1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a(), __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));\n}`
      )
    })

    it('does modify substring after call with argument variable', () => {
      const js = 'a().substring(b);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a(), __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = b, \
_ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0\
, __datadog_test_2));\n}`
      )
    })

    it('does modify substring after call with argument call', () => {
      const js = 'a().substring(b());'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a(), __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = b(), \
_ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, \
__datadog_test_0, __datadog_test_2));\n}`
      )
    })

    it('does modify substring after call with expressions in argument ', () => {
      const js = 'a().substring(c + b());'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4;
(__datadog_test_2 = a(), __datadog_test_3 = __datadog_test_2.substring, __datadog_test_4 = (__datadog_test_0 = c, \
__datadog_test_1 = b(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))\
, _ddiast.stringSubstring(__datadog_test_3.call(__datadog_test_2, __datadog_test_4), __datadog_test_3, \
__datadog_test_2, __datadog_test_4));\n}`
      )
    })

    it('does not modify literal String.prototype.substring.call', () => {
      const js = 'String.prototype.substring.call("hello", 2);'
      rewriteAndExpectNoTransformation(js)
    })

    it('does modify String.prototype.substring.call', () => {
      const js = 'String.prototype.substring.call(b, 2);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));\n}`
      )
    })

    it('does modify String.prototype.substring.call with expression argument', () => {
      const js = 'String.prototype.substring.call(b + c, 2);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
  (__datadog_test_0 = _ddiast.plusOperator(b + c, b, c), __datadog_test_1 = __datadog_test_0.substring, \
_ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));\n}`
      )
    })

    it('does modify String.prototype.substring.call with no arguments', () => {
      const js = 'String.prototype.substring.call(b);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0), __datadog_test_1, __datadog_test_0));
      }`
      )
    })

    it('does modify String.prototype.substring.apply with variable argument', () => {
      const js = 'String.prototype.substring.apply(b, [2]);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));\n}`
      )
    })

    it('does modify String.prototype.substring.apply with more arguments than needed', () => {
      const js = 'String.prototype.substring.apply(b, [2], 1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));
      }`
      )
    })

    it('does not modify String.prototype.substring.apply with incorrect arguments', () => {
      const js = 'String.prototype.substring.apply(b, 2);'
      rewriteAndExpectNoTransformation(js)
    })

    it('does not modify String.prototype.substring.apply with incorrect arguments', () => {
      const js = 'String.prototype.substring.apply();'
      rewriteAndExpectNoTransformation(js)
    })

    it('does not modify String.prototype.substring direct call', () => {
      const js = 'String.prototype.substring(1);'
      rewriteAndExpectNoTransformation(js)
    })

    it('does modify member.prop.substring call', () => {
      const js = 'a.b.c.substring(1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a.b.c, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));
      }`
      )
    })

    it('does modify member.call.prop.substring call', () => {
      const js = 'a.b().c.substring(1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a.b().c, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));
      }`
      )
    })

    it('does modify member.prop.call.substring call', () => {
      const js = 'a.b.c().substring(1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a.b.c(), __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1\
.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));
      }`
      )
    })
  })

  itEach(
    '${value}', // eslint-disable-line no-template-curly-in-string
    [
      'trim',
      'trimStart',
      'trimEnd',
      'toLowerCase',
      'toUpperCase',
      "replace('dog', 'monkey')",
      "replaceAll('dog', 'monkey')",
      'slice(4, 5)',
      "concat('hello', 'world')",
      "toLocaleUpperCase('en-US')",
      "toLocaleLowerCase('en-US')"
    ],
    (value) => {
      function getMethod (value) {
        return value.split('(')[0]
      }

      function getArgs (value) {
        const parts = value.split('(')
        if (parts.length < 2) return ''
        parts.shift()
        return parts[0].substring(0, parts[0].length - 1)
      }

      const method = getMethod(value)
      const args = getArgs(value)
      const argsWithComma = args ? `, ${args}` : ''

      describe(value, () => {
        it(`does not modify "literal".${value}`, () => {
          const js = `"a".${method}(${args});`
          rewriteAndExpectNoTransformation(js)
        })

        it(`does modify ident.${value}`, () => {
          const js = `a.${method}(${args});`
          rewriteAndExpect(
            js,
            `{
      let __datadog_test_0, __datadog_test_1;
      (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.${method}, _ddiast._${method}(__datadog_test_1\
.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));\n}`
          )
        })

        it(`does modify call().${value}`, () => {
          const js = `a().${method}(${args});`
          rewriteAndExpect(
            js,
            `{
      let __datadog_test_0, __datadog_test_1;
      (__datadog_test_0 = a(), __datadog_test_1 = __datadog_test_0.${method}, _ddiast._${method}(\
__datadog_test_1.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));\n}`
          )
        })

        it(`does modify member.${value}`, () => {
          const js = `a.b.${method}(${args});`
          rewriteAndExpect(
            js,
            `{
      let __datadog_test_0, __datadog_test_1;
      (__datadog_test_0 = a.b, __datadog_test_1 = __datadog_test_0.${method}, _ddiast._${method}(\
__datadog_test_1.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));\n}`
          )
        })

        it(`does not modify literal String.prototype.${value}.call`, () => {
          const js = `String.prototype.${method}.call("hello"${argsWithComma});`
          rewriteAndExpectNoTransformation(js)
        })

        it(`does modify String.prototype.${method}.call`, () => {
          const js = `String.prototype.${method}.call(b${argsWithComma});`
          rewriteAndExpect(
            js,
            `{
    let __datadog_test_0, __datadog_test_1;
    (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.${method}, _ddiast._${method}(\
__datadog_test_1.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));\n}`
          )
        })

        it(`does modify String.prototype.${value}.apply with variable argument`, () => {
          const js = `String.prototype.${method}.apply(b, [${args}]);`
          rewriteAndExpect(
            js,
            `{
    let __datadog_test_0, __datadog_test_1;
    (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.${method}, _ddiast._${method}(\
__datadog_test_1.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));\n}`
          )
        })
      })
    }
  )
})
