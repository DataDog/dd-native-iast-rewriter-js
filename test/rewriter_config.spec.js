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

    const onlySubstringCsiMethod = {
      'String.prototype': ['substring']
    }

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
  })

  describe('csi methods list', () => {
    it('should list all rewritten methods', () => {
      const rewriter = new Rewriter({ csiMethods })

      // eslint-disable-next-line no-unused-expressions
      expect(rewriter.csiMethods()).to.not.be.empty
      expect(rewriter.csiMethods()).to.include('plusOperator')
      expect(rewriter.csiMethods()).to.include('string_substring')
      expect(rewriter.csiMethods()).to.include('string_concat')
    })
  })
})
