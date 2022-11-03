/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */
'use strict'

const { Rewriter, rewriteAndExpect, rewriteAndExpectNoTransformation } = require('./util')

describe('rewriter configuration', () => {
  describe('csi exclusions', () => {
    const rewriteAndExpectWithExclusions = function (js, expect, exclusions) {
      const rewriter = new Rewriter({ csiExclusions: exclusions ?? ['String.prototype.concat'] })
      return rewriteAndExpect(js, expect, false, rewriter)
    }

    it('does not rewrite excluded method', () => {
      const rewriter = new Rewriter({ csiExclusions: ['String.prototype.concat'] })
      const js = 'const result = a.concat("b");'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })

    it('does rewrite method and keep excluded', () => {
      const js = 'const result = a.substring(2).concat("b");'
      rewriteAndExpectWithExclusions(
        js,
        `{
      let __datadog_test_0, __datadog_test_1;
const result = (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(\
__datadog_test_1(2), __datadog_test_1, __datadog_test_0, 2)).concat("b");
      }`
      )
    })

    it('does not rewrite multiple excluded methods', () => {
      const rewriter = new Rewriter({ csiExclusions: ['String.prototype.concat', 'String.prototype.substring'] })
      const js = 'const result = a.substring(2).concat("b");'
      rewriteAndExpectNoTransformation(js, { rewriter })
    })
  })

  describe('csi methods list', () => {
    it('should list all rewritten methods', () => {
      const rewriter = new Rewriter()

      // eslint-disable-next-line no-unused-expressions
      expect(rewriter.csiMethods()).to.not.be.empty
      expect(rewriter.csiMethods()).to.include('plusOperator')
      expect(rewriter.csiMethods()).to.include('string_substring')
      expect(rewriter.csiMethods()).to.include('string_concat')
    })
  })
})
