/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */

const { rewriteAndExpectNoTransformation, rewriteAndExpect } = require('./util')

describe('string method', () => {
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
(__datadog_test_0 = a, _ddiast.substring(__datadog_test_0.substring(1), __datadog_test_0, 1));\n}`
    )
  })

  it('does modify substring after call', () => {
    const js = 'a().substring(1);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = a(), _ddiast.substring(__datadog_test_0.substring(1), __datadog_test_0, 1));\n}`
    )
  })

  it('does modify substring after call with argument variable', () => {
    const js = 'a().substring(b);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a(), __datadog_test_1 = b, _ddiast.substring(__datadog_test_0.substring(__datadog_test_1), \
__datadog_test_0, __datadog_test_1));\n}`
    )
  })

  it('does modify substring after call with argument call', () => {
    const js = 'a().substring(b());'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a(), __datadog_test_1 = b(), _ddiast.substring(__datadog_test_0.substring(__datadog_test_1), \
__datadog_test_0, __datadog_test_1));\n}`
    )
  })
})
