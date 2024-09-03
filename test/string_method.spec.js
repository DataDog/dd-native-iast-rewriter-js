/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */

const { itEach } = require('mocha-it-each')

const { rewriteAndExpectNoTransformation, rewriteAndExpect, rewriteAndExpectAndExpectEval, fn } = require('./util')

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
  let __datadog_test_0;
  (__datadog_test_0 = a.substring, _ddiast.stringSubstring(__datadog_test_0\
.call(a, 1), __datadog_test_0, a, 1));\n}`
      )
    })

    it('does modify substring with 2 arguments', () => {
      const js = 'a.substring(1, a.lenght - 2);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a.substring, __datadog_test_1 = a.lenght - 2, \
_ddiast.stringSubstring(__datadog_test_0.call(a, 1, __datadog_test_1), __datadog_test_0, \
a, 1, __datadog_test_1));\n}`
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
  let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a(), __datadog_test_1 = __datadog_test_0.substring, \
_ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, b), __datadog_test_1, __datadog_test_0\
, b));\n}`
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
  let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;
(__datadog_test_1 = a(), __datadog_test_2 = __datadog_test_1.substring, __datadog_test_3 = (\
__datadog_test_0 = b(), _ddiast.plusOperator(c + __datadog_test_0, c, __datadog_test_0))\
, _ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, __datadog_test_3), __datadog_test_2, \
__datadog_test_1, __datadog_test_3));\n}`
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
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.substring, _ddiast.stringSubstring(__datadog_test_0\
.call(b, 2), __datadog_test_0, b, 2));\n}`
      )
    })

    it('does modify String.prototype.substring.call with expression argument', () => {
      const js = 'String.prototype.substring.call(b + c, 2);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0, __datadog_test_1;
  (__datadog_test_0 = _ddiast.plusOperator(b + c, b, c), __datadog_test_1 = String.prototype.substring, \
_ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));\n}`
      )
    })

    it('does modify String.prototype.substring.call with no arguments', () => {
      const js = 'String.prototype.substring.call(b);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.substring, _ddiast.stringSubstring(__datadog_test_0\
.call(b), __datadog_test_0, b));
      }`
      )
    })

    it('does modify String.prototype.substring.apply with variable argument', () => {
      const js = 'String.prototype.substring.apply(b, [2]);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.substring, _ddiast.stringSubstring(__datadog_test_0\
.call(b, 2), __datadog_test_0, b, 2));\n}`
      )
    })

    it('does modify String.prototype.substring.apply with more arguments than needed', () => {
      const js = 'String.prototype.substring.apply(b, [2], 1);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.substring, _ddiast.stringSubstring(__datadog_test_0\
.call(b, 2), __datadog_test_0, b, 2));
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

  describe('concat (method that allows literals)', () => {
    it('does not modify String.prototype.concat call if all args are literals', () => {
      const js = 'String.prototype.concat.call("hello", "world");'
      rewriteAndExpectNoTransformation(js)
    })

    it('does modify String.prototype.concat call if some ident', () => {
      const js = 'String.prototype.concat.call("hello", a, "world");'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.concat, _ddiast.concat(__datadog_test_0.call("hello", a, "world")\
, __datadog_test_0, "hello", a, "world"));
      }`
      )
    })

    it('does not modify String.prototype.concat apply if all args are literals', () => {
      const js = 'String.prototype.concat.apply("hello", ["world", null, "moon"]);'
      rewriteAndExpectNoTransformation(js)
    })

    it('does modify String.prototype.concat apply if this is not a literal', () => {
      const js = 'String.prototype.concat.apply(a, ["world", null]);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.concat, _ddiast.concat(__datadog_test_0.call(\
a, "world", null), __datadog_test_0, a, "world", null));
      }`
      )
    })

    it('does modify String.prototype.concat apply if an argument is not a literal', () => {
      const js = 'String.prototype.concat.apply("hello", ["world", a]);'
      rewriteAndExpect(
        js,
        `{
  let __datadog_test_0;
(__datadog_test_0 = String.prototype.concat, \
_ddiast.concat(__datadog_test_0.call("hello", "world", a), __datadog_test_0, \
"hello", "world", a));
      }`
      )
    })
  })

  const methodAllowingLiterals = ['concat', 'replace']

  itEach(
    '${value}', // eslint-disable-line no-template-curly-in-string
    [
      'trim',
      'trimStart',
      'trimEnd',
      'toLowerCase',
      'toUpperCase',
      "replace('dog', 'monkey')",
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
        if (methodAllowingLiterals.indexOf(method) !== -1) {
          it(`does modify "literal".${value}`, () => {
            const builder = fn()
            const js = builder.build(`return 'a'.${method}(${args});`)
            rewriteAndExpectAndExpectEval(
              js,
              builder.build(`let __datadog_test_0;
        return (__datadog_test_0 = 'a'.${method}, _ddiast.${method}(\
__datadog_test_0.call('a'${argsWithComma}), __datadog_test_0, 'a'${argsWithComma}));`)
            )
          })
        } else {
          it(`does not modify "literal".${value}`, () => {
            const js = `'a'.${method}(${args});`
            rewriteAndExpectNoTransformation(js)
          })
        }

        it(`does modify ident.${value}`, () => {
          const builder = fn().args('heLLo')
          const js = builder.build(`return a.${method}(${args});`)
          rewriteAndExpectAndExpectEval(
            js,
            builder.build(`let __datadog_test_0;
      return (__datadog_test_0 = a.${method}, _ddiast.${method}(__datadog_test_0\
.call(a${argsWithComma}), __datadog_test_0, a${argsWithComma}));`)
          )
        })

        it(`does modify call().${value}`, () => {
          const builder = fn().args(() => 'heLLo')
          const js = builder.build(`a().${method}(${args});`)
          rewriteAndExpectAndExpectEval(
            js,
            builder.build(`let __datadog_test_0, __datadog_test_1;
      (__datadog_test_0 = a(), __datadog_test_1 = __datadog_test_0.${method}, _ddiast.${method}(\
__datadog_test_1.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));`)
          )
        })

        it(`does modify member.${value}`, () => {
          const builder = fn().args({ b: 'heLLo' })
          const js = builder.build(`a.b.${method}(${args});`)
          rewriteAndExpectAndExpectEval(
            js,
            builder.build(`let __datadog_test_0, __datadog_test_1;
      (__datadog_test_0 = a.b, __datadog_test_1 = __datadog_test_0.${method}, _ddiast.${method}(\
__datadog_test_1.call(__datadog_test_0${argsWithComma}), __datadog_test_1, __datadog_test_0${argsWithComma}));`)
          )
        })

        if (methodAllowingLiterals.indexOf(method) !== -1) {
          it(`does modify literal String.prototype.${value}.call`, () => {
            const builder = fn().args({ a: 'heLLo' })
            const js = builder.build(`String.prototype.${method}.call(a${argsWithComma});`)
            rewriteAndExpectAndExpectEval(
              js,
              builder.build(`let __datadog_test_0;
        (__datadog_test_0 = String.prototype.${method}, _ddiast.${method}(\
__datadog_test_0.call(a${argsWithComma}), __datadog_test_0, a${argsWithComma}));`)
            )
          })
        } else {
          it(`does not modify literal String.prototype.${value}.call`, () => {
            const js = `String.prototype.${method}.call("hello"${argsWithComma});`
            rewriteAndExpectNoTransformation(js)
          })
        }

        it(`does modify String.prototype.${method}.call`, () => {
          const builder = fn().args('heLLo')
          const js = builder.build(`String.prototype.${method}.call(a${argsWithComma});`)
          rewriteAndExpectAndExpectEval(
            js,
            builder.build(`let __datadog_test_0;
    (__datadog_test_0 = String.prototype.${method}, _ddiast.${method}(\
__datadog_test_0.call(a${argsWithComma}), __datadog_test_0, a${argsWithComma}));`)
          )
        })

        it(`does modify String.prototype.${value}.apply with variable argument`, () => {
          const builder = fn().args('heLLo')
          const js = builder.build(`String.prototype.${method}.apply(a, [${args}]);`)
          rewriteAndExpectAndExpectEval(
            js,
            builder.build(`let __datadog_test_0;
    (__datadog_test_0 = String.prototype.${method}, _ddiast.${method}(\
__datadog_test_0.call(a${argsWithComma}), __datadog_test_0, a${argsWithComma}));`)
          )
        })
      })
    }
  )
})
