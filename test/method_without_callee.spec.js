'use strict'

const { rewriteAndExpect } = require('./util')

describe('Method without callee', () => {
  it('should rewrite aloneMethod', () => {
    const js = 'aloneMethod(arg0)'
    rewriteAndExpect(
      js,
      `{
_ddiast.aloneMethod(aloneMethod(arg0), aloneMethod, undefined, arg0);
}`
    )
  })

  it('should rewrite aloneMethod with 2 args', () => {
    const js = 'aloneMethod(arg0, obj.arg1)'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = obj.arg1, _ddiast.aloneMethod(aloneMethod(arg0, __datadog_test_0), aloneMethod, undefined, \
arg0, __datadog_test_0));
}`
    )
  })
  it('should rewrite aloneMethod without args', () => {
    const js = 'aloneMethod()'
    rewriteAndExpect(
      js,
      `{
_ddiast.aloneMethod(aloneMethod(), aloneMethod, undefined);
}`
    )
  })

  it('should rewrite aloneMethod when it is called with callee', () => {
    const js = 'obj.aloneMethod(arg0)'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = obj.aloneMethod, _ddiast.aloneMethod(__datadog_test_0.call(obj, arg0), __datadog_test_0, \
obj, arg0));
}`
    )
  })

  it('should not rewrite method not configured as alone when it is used alone', () => {
    const js = 'cantAloneMethod(arg0)'
    rewriteAndExpect(js, '{cantAloneMethod(arg0)}')
  })
})
