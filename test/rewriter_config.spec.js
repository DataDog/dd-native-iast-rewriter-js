/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */
/* eslint-disable no-unused-expressions */
'use strict'

const { expect } = require('chai')
const {
  Rewriter,
  DummyRewriter,
  rewriteAndExpect,
  rewriteAndExpectNoTransformation,
  csiMethods,
  resourceFile
} = require('./util')
const { generateSourceMapFromFileContent } = require('../js/source-map')

describe('rewriter configuration', () => {
  describe('csi exclusions', () => {
    const rewriteAndExpectWithCsiMethods = function (js, expect, csiMethods) {
      const rewriter = new Rewriter({ csiMethods, localVarPrefix: 'test' })
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
      const rewriter = new Rewriter({ csiMethods, localVarPrefix: 'test' })

      expect(rewriter.csiMethods()).to.not.be.empty
      expect(rewriter.csiMethods()).to.include('plusOperator')
      expect(rewriter.csiMethods()).to.include('stringSubstring')
      expect(rewriter.csiMethods()).to.include('concat')
    })

    it('should not throw Error with no RewriterConfig', () => {
      const rewriter = new Rewriter()

      expect(rewriter.csiMethods()).to.be.empty
    })

    it('should not throw Error', () => {
      const rewriter = new Rewriter([1, 2])
      expect(rewriter.csiMethods()).to.be.empty
    })
  })

  describe('telemetry verbosity', () => {
    it('should accept OFF verbosity', () => {
      const rewriter = new Rewriter({ csiMethods, telemetryVerbosity: 'OFF' })
      const response = rewriter.rewrite('{const a = b + c}', 'index.js')
      expect(response).to.have.property('content')
      expect(response).to.have.property('metrics')

      const metrics = response.metrics
      expect(metrics).to.not.be.undefined
      expect(metrics.status).eq('Modified')
      expect(metrics.instrumentedPropagation).eq(0)
      expect(metrics.propagationDebug).to.be.undefined
    })

    it('should accept MANDATORY verbosity', () => {
      const rewriter = new Rewriter({ csiMethods, telemetryVerbosity: 'MANDATORY' })
      const response = rewriter.rewrite('{const a = b + c}', 'index.js')
      expect(response).to.have.property('content')
      expect(response).to.have.property('metrics')

      const metrics = response.metrics
      expect(metrics).to.not.be.undefined
      expect(metrics.status).eq('Modified')
      expect(metrics.instrumentedPropagation).eq(1)
      expect(metrics.propagationDebug).to.be.undefined
    })

    it('should accept INFORMATION verbosity', () => {
      const rewriter = new Rewriter({ csiMethods, telemetryVerbosity: 'INFORMATION' })
      const response = rewriter.rewrite('{const a = b + c}', 'index.js')
      expect(response).to.have.property('content')
      expect(response).to.have.property('metrics')

      const metrics = response.metrics
      expect(metrics).to.not.be.undefined
      expect(metrics.status).eq('Modified')
      expect(metrics.instrumentedPropagation).eq(1)
      expect(metrics.propagationDebug).to.be.undefined
    })

    it('should accept DEBUG verbosity', () => {
      const rewriter = new Rewriter({ csiMethods, telemetryVerbosity: 'DEBUG' })
      const response = rewriter.rewrite('{const a = b + c}', 'index.js')
      expect(response).to.have.property('content')
      expect(response).to.have.property('metrics')

      const metrics = response.metrics
      expect(metrics).to.not.be.undefined
      expect(metrics.status).eq('Modified')
      expect(metrics.instrumentedPropagation).eq(1)
      expect(metrics.propagationDebug.size).eq(1)
      expect(metrics.propagationDebug.get('+')).eq(1)
    })

    it('should accept unknown verbosity and set it as INFORMATION', () => {
      const rewriter = new Rewriter({ csiMethods, telemetryVerbosity: 'unknown' })
      const response = rewriter.rewrite('{const a = b + c}', 'index.js')
      expect(response).to.have.property('content')
      expect(response).to.have.property('metrics')

      const metrics = response.metrics
      expect(metrics).to.not.be.undefined
      expect(metrics.status).eq('Modified')
      expect(metrics.instrumentedPropagation).eq(1)
    })

    it('should apply Information verbosity as default', () => {
      const rewriter = new Rewriter({ csiMethods })
      const response = rewriter.rewrite('{const a = b + c}', 'index.js')
      expect(response).to.have.property('content')
      expect(response).to.have.property('metrics')

      const metrics = response.metrics
      expect(metrics).to.not.be.undefined
      expect(metrics.status).eq('Modified')
      expect(metrics.instrumentedPropagation).eq(1)
      expect(metrics.propagationDebug).to.be.undefined
    })
  })

  describe('dummy rewriter', () => {
    describe('rewrite method', () => {
      it('should have same return type as Rewriter.rewrite', () => {
        const rewriter = new DummyRewriter()
        const response = rewriter.rewrite('{const a = b + c}', 'index.js')
        expect(response).to.have.property('content')
      })
    })
  })

  describe('chainSourceMap', () => {
    it('should not chain original source map', () => {
      const rewriter = new Rewriter({ csiMethods })

      const resource = resourceFile('sourcemap', 'StrUtil_external.js')
      const result = rewriter.rewrite(resource.content, resource.filename)

      const content = result.content
      expect(content).to.not.undefined

      const sourceMap = generateSourceMapFromFileContent(content, resource.filename)
      expect(sourceMap).to.not.undefined

      for (const source in sourceMap._sources) {
        expect(source).to.contain('StrUtil_external.js')
      }
    })

    it('should chain original source map', () => {
      const rewriter = new Rewriter({ csiMethods, chainSourceMap: true })

      const resource = resourceFile('sourcemap', 'StrUtil_external.js')
      const result = rewriter.rewrite(resource.content, resource.filename)

      const content = result.content
      expect(content).to.not.undefined

      const sourceMap = generateSourceMapFromFileContent(content, resource.filename)
      expect(sourceMap).to.not.undefined

      for (const source in sourceMap._sources) {
        expect(source).to.contain('StrUtil.ts')
      }
    })
  })
})
