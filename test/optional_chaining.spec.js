/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */

const { rewriteAndExpectNoTransformation, rewriteAndExpect } = require('./util')

describe('Optional chaining', () => {
  describe('substring', () => {
    it('does not rewrite if it is not necessary', () => {
      const js = 'a?.customMethod(1);'

      rewriteAndExpectNoTransformation(js, { logLevel: 'DEBUG', logger: console })
    })

    it('should not modify optional method', () => {
      const js = 'a?.substring?.(1);'

      rewriteAndExpectNoTransformation(js, { logLevel: 'DEBUG', logger: console })
    })

    it('should modify a?.substring(1)', () => {
      const js = 'a?.substring(1);'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify a.b?.method()', () => {
      const js = 'a.b?.substring(1);'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a.b, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify a.b()?.substring(1)', () => {
      const js = 'a.b()?.substring(1);'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a.b(), __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify a.b?.substring()?.substring(1)', () => {
      const js = 'a.b?.substring()?.substring(1);'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5;
(__datadog_test_0 = (__datadog_test_1 = a.b, __datadog_test_1 == null ? undefined : \
(__datadog_test_2 = __datadog_test_1, __datadog_test_3 = __datadog_test_2.substring, \
_ddiast.stringSubstring(__datadog_test_3.call(__datadog_test_2), __datadog_test_3, __datadog_test_2))), \
__datadog_test_0 == null ? undefined : (__datadog_test_4 = __datadog_test_0, \
__datadog_test_5 = __datadog_test_4.substring, _ddiast.stringSubstring(\
__datadog_test_5.call(__datadog_test_4, 1), __datadog_test_5, __datadog_test_4, 1)));
}`,
        false
      )
    })

    it('should modify a?.b.substring(1)', () => {
      const js = 'a?.b.substring(1);'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0.b, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify a?.b?.substring(1)', () => {
      const js = 'a?.b?.substring(1);'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a?.b, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify a.b?.substring(1).otherMethod(2)', () => {
      const js = 'a.b?.substring(1).otherMethod(2)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a.b, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1))\
.otherMethod(2));
}`,
        false
      )
    })

    it('should modify a.b?.substring(1).c?.otherMethod(2)', () => {
      const js = 'a.b?.substring(1).c?.otherMethod(2)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a.b, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1))\
.c?.otherMethod(2));
}`,
        false
      )
    })

    it('should modify a.b?.(param).substring(1)', () => {
      const js = 'a.b?.(param).substring(1)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;
(__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.b, __datadog_test_1 == null ? undefined : \
(__datadog_test_2 = __datadog_test_1.call(__datadog_test_0, param), \
__datadog_test_3 = __datadog_test_2.substring, _ddiast.stringSubstring(__datadog_test_3.call(__datadog_test_2, 1), \
__datadog_test_3, __datadog_test_2, 1)));
}`,
        false
      )
    })

    it('should modify b?.().substring(1)', () => {
      const js = 'b?.().substring(1)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = b, __datadog_test_0 == null ? undefined : (__datadog_test_1 = __datadog_test_0(), \
__datadog_test_2 = __datadog_test_1.substring, _ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), \
__datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify (b?.substring(1)).otherMethod()', () => {
      const js = '(b?.substring(1)).otherMethod()'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
((__datadog_test_0 = b, __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, \
__datadog_test_1, 1)))).otherMethod();
}`,
        false
      )
    })

    it('should modify a?.b()?.param.substring(1)', () => {
      const js = 'a?.b()?.param.substring(1)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a?.b(), __datadog_test_0 == null ? undefined : \
(__datadog_test_1 = __datadog_test_0.param, __datadog_test_2 = __datadog_test_1.substring, \
_ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), __datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })

    it('should modify a?.substring(1)?.b.substring(2)', () => {
      const js = 'a?.substring(1)?.b.substring(2)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5;
(__datadog_test_0 = (__datadog_test_1 = a, __datadog_test_1 == null ? undefined : \
(__datadog_test_2 = __datadog_test_1, __datadog_test_3 = __datadog_test_2.substring, _ddiast.stringSubstring(\
__datadog_test_3.call(__datadog_test_2, 1), __datadog_test_3, __datadog_test_2, 1))), __datadog_test_0 == null ? \
undefined : (__datadog_test_4 = __datadog_test_0.b, __datadog_test_5 = __datadog_test_4.substring, \
_ddiast.stringSubstring(__datadog_test_5.call(__datadog_test_4, 2), __datadog_test_5, __datadog_test_4, 2)));
}`,
        false
      )
    })

    it('should modify a?().b.substring(1)', () => {
      const js = 'a?.().b.substring(1)'

      rewriteAndExpect(
        js,
        `{
let __datadog_test_0, __datadog_test_1, __datadog_test_2;
(__datadog_test_0 = a, __datadog_test_0 == null ? undefined : (__datadog_test_1 = __datadog_test_0().b, \
__datadog_test_2 = __datadog_test_1.substring, _ddiast.stringSubstring(__datadog_test_2.call(__datadog_test_1, 1), \
__datadog_test_2, __datadog_test_1, 1)));
}`,
        false
      )
    })
  })
})
