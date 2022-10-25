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
(__datadog_test_0 = a, _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));\n}`
    )
  })

  it('does modify substring with 2 arguments', () => {
    const js = 'a.substring(1, a.lenght - 2);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a, __datadog_test_1 = a.lenght - 2, _ddiast.string_substring(__datadog_test_0.substring(1, \
__datadog_test_1), __datadog_test_0, 1, __datadog_test_1));\n}`
    )
  })

  it('does modify substring after call', () => {
    const js = 'a().substring(1);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = a(), _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));\n}`
    )
  })

  it('does modify substring after call with argument variable', () => {
    const js = 'a().substring(b);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a(), __datadog_test_1 = b, _ddiast.string_substring(__datadog_test_0.substring(__datadog_test_1), \
__datadog_test_0, __datadog_test_1));\n}`
    )
  })

  it('does modify substring after call with argument call', () => {
    const js = 'a().substring(b());'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1;
(__datadog_test_0 = a(), __datadog_test_1 = b(), _ddiast.string_substring(__datadog_test_0.substring(__datadog_test_1)\
, __datadog_test_0, __datadog_test_1));\n}`
    )
  })

  it('does modify substring after call with expressions in argument ', () => {
    const js = 'a().substring(c + b());'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;
(__datadog_test_2 = a(), __datadog_test_3 = (__datadog_test_0 = c, __datadog_test_1 = b(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.string_substring(\
__datadog_test_2.substring(__datadog_test_3), __datadog_test_2, __datadog_test_3));\n}`
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
(__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(2), __datadog_test_0, 2));\n}`
    )
  })

  it('does modify String.prototype.substring.call with expression argument', () => {
    const js = 'String.prototype.substring.call(b + c, 2);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = _ddiast.plusOperator(b + c, b, c), _ddiast.string_substring(__datadog_test_0.substring(2), \
__datadog_test_0, 2));\n}`
    )
  })

  it('does modify String.prototype.substring.call with no arguments', () => {
    const js = 'String.prototype.substring.call(b);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(), __datadog_test_0));
    }`
    )
  })

  it('does modify String.prototype.substring.apply with variable argument', () => {
    const js = 'String.prototype.substring.apply(b, [2]);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(2), __datadog_test_0, 2));\n}`
    )
  })

  it('does modify String.prototype.substring.apply with more arguments than needed', () => {
    const js = 'String.prototype.substring.apply(b, [2], 1);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(2), __datadog_test_0, 2));
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
let __datadog_test_0;
(__datadog_test_0 = a.b.c, _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));
    }`
    )
  })

  it('does modify member.call.prop.substring call', () => {
    const js = 'a.b().c.substring(1);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = a.b().c, _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));
    }`
    )
  })

  it('does modify member.prop.call.substring call', () => {
    const js = 'a.b.c().substring(1);'
    rewriteAndExpect(
      js,
      `{
let __datadog_test_0;
(__datadog_test_0 = a.b.c(), _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));
    }`
    )
  })
})