/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */
'use strict'

const { Rewriter, rewriteAndExpect, rewriteAndExpectNoTransformation, csiMethods } = require('./util')

describe('rewriter configuration', () => {
  describe('csi exclusions', () => {
    const rewriteAndExpectWithCsiMethods = function (js, expect, csiMethods) {
      const rewriter = new Rewriter({ csiMethods })
      return rewriteAndExpect(js, expect, false, { rewriter })
    }

    const onlySubstringCsiMethod = [{ src: 'substring', dst: 'string_substring' }]
    const plusOperatorAndOthersCsiMethods = [
      { src: 'plusOperator', dst: 'plus', operator: true },
      { src: 'substring', dst: 'string_substring' },
      { src: 'custom_method' }
    ]

    it('does not rewrite excluded method', () => {
      const rewriter = new Rewriter()
      const js = 'const result = a.concat("b");'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })

    it('does rewrite method and keep excluded', () => {
      const js = 'const result = a.substring(2).concat("b");'
      rewriteAndExpectWithCsiMethods(
        js,
        `{
      let __datadog_test_0, __datadog_test_1;
const result = (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(\
__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2)).concat("b");
      }`,
        onlySubstringCsiMethod
      )
    })

    it('does not rewrite multiple excluded methods', () => {
      const rewriter = new Rewriter()
      const js = 'const result = a.substring(2).concat("b");'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })

    it('does not rewrite + operation', () => {
      const rewriter = new Rewriter()
      const js = 'const result = a.concat("b" + c);'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })

    it('does not rewrite += operation', () => {
      const rewriter = new Rewriter()
      const js = 'result += a.concat("b");'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })

    it('does not rewrite template literals operation', () => {
      const rewriter = new Rewriter()
      // eslint-disable-next-line no-template-curly-in-string
      const js = 'const result = `hello ${a}`'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })

    it('does rewrite + with altenative dst name and substring and keep excluded', () => {
      const js = 'const result = a.substring(2).concat("b" + c);'
      rewriteAndExpectWithCsiMethods(
        js,
        `{
      let __datadog_test_0, __datadog_test_1;
const result = (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(\
__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2)).concat(\
_ddiast.plus("b" + c, "b", c));
      }`,
        plusOperatorAndOthersCsiMethods
      )
    })

    it('does rewrite custom_method method', () => {
      const js = 'const result = a.custom_method(2).concat("b" + c);'
      rewriteAndExpectWithCsiMethods(
        js,
        `{
      let __datadog_test_0, __datadog_test_1;
const result = (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.custom_method, _ddiast.custom_method(\
__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2)).concat(\
_ddiast.plus("b" + c, "b", c));
      }`,
        plusOperatorAndOthersCsiMethods
      )
    })

    it('does rewrite Whatever.prototype.custom_method method', () => {
      const js = 'const result = Whatever.prototype.custom_method.call(a, 2).concat("b" + c);'
      rewriteAndExpectWithCsiMethods(
        js,
        `{
      let __datadog_test_0, __datadog_test_1;
const result = (__datadog_test_0 = a, __datadog_test_1 = Whatever.prototype.custom_method, _ddiast.custom_method(\
__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2)).concat(\
_ddiast.plus("b" + c, "b", c));
      }`,
        plusOperatorAndOthersCsiMethods
      )
    })
  })

  describe('csi methods list', () => {
    it('should list all rewritten methods', () => {
      const rewriter = new Rewriter({ csiMethods })

      // eslint-disable-next-line no-unused-expressions
      expect(rewriter.csiMethods()).to.not.be.empty
      expect(rewriter.csiMethods()).to.include('plusOperator')
      expect(rewriter.csiMethods()).to.include('stringSubstring')
      expect(rewriter.csiMethods()).to.include('concat')
    })
  })
})
