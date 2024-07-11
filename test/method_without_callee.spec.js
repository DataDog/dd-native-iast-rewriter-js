'use strict'

const { rewriteAndExpect } = require('./util')

describe('Method without callee', () => {
  it('should rewrite aloneMethod', () => {
    const js = 'aloneMethod(arg0)'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = undefined, __datadog_test_1 = aloneMethod, __datadog_test_2 = arg0, _ddiast.aloneMethod\
(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0, __datadog_test_2));
}`
    )
  })

  it('should rewrite aloneMethod when it is called with callee', () => {
    const js = 'obj.aloneMethod(arg0)'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = obj, __datadog_test_1 = __datadog_test_0.aloneMethod, __datadog_test_2 = arg0, _ddiast.aloneMethod\
(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0, __datadog_test_2));
}`
    )
  })

  it('should not rewrite method not configured as alone when it is used alone', () => {
    const js = 'cantAloneMethod(arg0)'
    rewriteAndExpect(js, '{cantAloneMethod(arg0)}')
  })
})
