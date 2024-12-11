/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-template-curly-in-string */
/* eslint-disable no-multi-str */
const { rewriteAst, rewriteAndExpectNoTransformation, rewriteAndExpect, csiMethods } = require('./util')
const { readFileSync } = require('fs')
const path = require('path')

describe('template literal', () => {
  describe('rewriting tests', () => {
    it('empty', () => {
      const js = 'const result = `Hello World!`;'
      rewriteAndExpectNoTransformation(js)
    })

    it('literal', () => {
      const js = 'const result = `Hello${" "}World!`;'
      rewriteAndExpectNoTransformation(js)
    })

    it('not enabled tplOperator', () => {
      const js = 'const result = `Hello${a}World!`;'
      rewriteAndExpectNoTransformation(js, {
        csiMethods: csiMethods.filter((m) => m.src !== 'tplOperator')
      })
    })

    it('middle', () => {
      const js = 'const result = `Hello${a}World!`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
          const result = (__datadog_test_0 = a, _ddiast.tplOperator(`Hello${__datadog_test_0}World!`, \
__datadog_test_0));\n}'
      )
    })

    it('start', () => {
      const js = 'const result = `${a}Hello World!`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
const result = (__datadog_test_0 = a, _ddiast.tplOperator(`${__datadog_test_0}Hello World!`, __datadog_test_0));\n}'
      )
    })

    it('multiline string', () => {
      const js = `router.get('/xss', function(req, res) {
  res.header('content-type', 'text/html');
  res.send(\`<html lang="en">
    <body>
        <h1>XSS vulnerability</h1>
        <p>Received param: \${req.query.param}</p>
    </body>
</body>
</html>\`);
  });`
      const expected = `{\nrouter.get('/xss', function(req, res) {
    let __datadog_test_0;
    res.header('content-type', 'text/html');
    res.send((__datadog_test_0 = req.query.param, _ddiast.tplOperator(\`<html lang="en">
      <body>
      <h1>XSS vulnerability</h1>
      <p>Received param: \${__datadog_test_0}</p>
      </body>
      </body>
      </html>\`, __datadog_test_0)));
});\n}`
      rewriteAndExpect(js, expected)
    })

    it('Only vars', () => {
      const js = 'const result = `${a}${b}`'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1;\n\
const result = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.tplOperator(\
`${__datadog_test_0}${__datadog_test_1}`, __datadog_test_0, __datadog_test_1));\n}'
      )
    })

    it('end', () => {
      const js = 'const result = `Hello World!${a}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
const result = (__datadog_test_0 = a, _ddiast.tplOperator(`Hello World!${__datadog_test_0}`, __datadog_test_0));\n}'
      )
    })

    it('with binary operations', () => {
      const js = 'const result = `Hello World!${a + b}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
        const result = (__datadog_test_0 = _ddiast.plusOperator(a + b, a, b), _ddiast.tplOperator(\
`Hello World!${__datadog_test_0}`, __datadog_test_0));\n}'
      )
    })

    it('with call', () => {
      const js = 'const result = `Hello World!${a()}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\nconst result = (__datadog_test_0 = a(), _ddiast.tplOperator(\
`Hello World!${__datadog_test_0}`, __datadog_test_0));\n}'
      )
    })

    it('with binary operations and call', () => {
      const js = 'const result = `Hello World!${a + b()}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.tplOperator(\
`Hello World!${__datadog_test_2}`, __datadog_test_2));\n}'
      )
    })

    it('with binary operations and object property access', () => {
      const js = 'const result = `Hello World!${a + b.x}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b.x, _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.tplOperator(\
`Hello World!${__datadog_test_2}`, __datadog_test_2));\n}'
      )
    })

    it('with binary operations and chained object property access', () => {
      const js = 'const result = `Hello World!${a + b.x.y.z}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b.x.y.z, _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.tplOperator(\
`Hello World!${__datadog_test_2}`, __datadog_test_2));\n}'
      )
    })

    it('inside if test', () => {
      const js = 'const c = a === `Hello${b}` ? "world" : "moon";'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
const c = a === (__datadog_test_0 = b, _ddiast.tplOperator(`Hello${__datadog_test_0}`, __datadog_test_0)) ? "world" : \
"moon";\n}'
      )
    })

    it('inside if cons', () => {
      const js = 'const c = a === "hello" ? `World ${b}` : "Moon";'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
const c = a === "hello" ? (__datadog_test_0 = b, _ddiast.tplOperator(`World ${__datadog_test_0}`, __datadog_test_0)) \
: "Moon";\n}'
      )
    })

    it('typeof among variables is replaced by a variable', () => {
      const js = 'const a = `He${typeof b}llo wor${a}ld`'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1;\n\
const a = (__datadog_test_0 = typeof b, __datadog_test_1 = a, _ddiast.tplOperator(`He${__datadog_test_0}llo \
wor${__datadog_test_1}ld`, __datadog_test_0, __datadog_test_1));\n}'
      )
    })

    it('tagged are not mofified', () => {
      const js = 'const a = func`Hello${b}World`;'
      rewriteAndExpectNoTransformation(js)
    })

    it('tagged with child expressions are mofified', () => {
      const js = 'const a = func`Hello${b + c}World`;'
      rewriteAndExpect(js, '{\nconst a = func`Hello${_ddiast.plusOperator(b + c, b, c)}World`;\n}')
    })

    it('nested template literal with +', () => {
      const js = "const a = `Hello ${c} ${'how are u ' + `${'bye ' + d}`} world`;"
      rewriteAndExpect(
        js,
        "{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;\n\
const a = (__datadog_test_2 = c, __datadog_test_3 = (__datadog_test_1 = (__datadog_test_0 = _ddiast.plusOperator(\
'bye ' + d, 'bye ', d), _ddiast.tplOperator(`${__datadog_test_0}`, __datadog_test_0)), _ddiast.plusOperator(\
'how are u ' + __datadog_test_1, 'how are u ', __datadog_test_1)), _ddiast.tplOperator(\
`Hello ${__datadog_test_2} ${__datadog_test_3} world`, __datadog_test_2, __datadog_test_3));\n}"
      )
    })

    it('with update expression ++', () => {
      const js = 'const a = `Hello ${c++}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\nconst a = (__datadog_test_0 = c++, _ddiast.tplOperator(`Hello ${__datadog_test_0}`, \
__datadog_test_0));\n}'
      )
    })

    it('with update expression --', () => {
      const js = 'const a = `Hello ${--c}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\nconst a = (__datadog_test_0 = --c, _ddiast.tplOperator(`Hello ${__datadog_test_0}`, \
__datadog_test_0));\n}'
      )
    })

    it('with await expression', () => {
      const js = 'const a = `Hello ${await b()}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
const a = (__datadog_test_0 = await b(), _ddiast.tplOperator(`Hello ${__datadog_test_0}`, __datadog_test_0));\n}'
      )
    })

    it('with await expression inside +', () => {
      const js = 'const a = `Hello ${b + await c()}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
          const a = (__datadog_test_2 = (__datadog_test_0 = b, __datadog_test_1 = await c(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.tplOperator(\
`Hello ${__datadog_test_2}`, __datadog_test_2));\n}'
      )
    })

    it('with conditional', () => {
      const js = 'const a = `Hello ${b + c ? d : e}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0;\n\
          const a = (__datadog_test_0 = _ddiast.plusOperator(b + c, b, c) ? d : e, _ddiast.tplOperator(\
`Hello ${__datadog_test_0}`, __datadog_test_0));\n}'
      )
    })

    it('with new', () => {
      const js = 'const a = `Hello ${a + new B()}`;'
      rewriteAndExpect(
        js,
        '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const a = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = new B(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.tplOperator(\
`Hello ${__datadog_test_2}`, __datadog_test_2));\n}'
      )
    })

    it('with String.raw + tagged template + \\n', () => {
      const js = readFileSync(path.join(__dirname, 'resources/tmpl-literal.js')).toString()
      const rewritten = rewriteAndExpectNoTransformation(js)

      // eslint-disable-next-line no-eval
      const e1 = eval(js).RAW_AND_NEWLINE
      // eslint-disable-next-line no-eval
      const e2 = eval(rewritten).RAW_AND_NEWLINE
      expect(e1).to.be.equal(e2)
    })

    it('with String.raw + tagged template + new line', () => {
      const js = readFileSync(path.join(__dirname, 'resources/tmpl-literal.js')).toString()
      const rewritten = rewriteAndExpectNoTransformation(js)

      // eslint-disable-next-line no-eval
      const e1 = eval(js).RAW_AND_NEWLINE2
      // eslint-disable-next-line no-eval
      const e2 = eval(rewritten).RAW_AND_NEWLINE2
      expect(e1).to.be.equal(e2)
    })
  })

  describe('Execution tests', () => {
    // Used in rewritten code
    // eslint-disable-next-line no-unused-vars
    const _ddiast = {
      plusOperator: (res) => res,
      tplOperator: (res) => res
    }
    function rewriteAndCompare (origFunc, args) {
      const expectedResult = origFunc(...args)
      const rewritedFuncCode = rewriteAst(origFunc.toString())

      // eslint-disable-next-line no-eval
      const result = eval(`(${rewritedFuncCode})(...args)`)
      expect(result).to.be.equal(expectedResult)
    }

    const testFunctionsWith3Args = [
      {
        description: 'Template literal without literal content',
        testFunction: function test (p1, p2, p3) {
          return `${p1}${p2}${p3}`
        }
      },
      {
        description: 'Template literal with literal at the beginning',
        testFunction: function test (p1, p2, p3) {
          return `T${p1}${p2}${p3}`
        }
      },
      {
        description: 'Template literal with literal at the end',
        testFunction: function test (p1, p2, p3) {
          return `${p1}${p2}${p3}T`
        }
      },
      {
        description: 'Template literal with literal between each param',
        testFunction: function test (p1, p2, p3) {
          return `${p1}T1${p2}T2${p3}`
        }
      },
      {
        description: 'Template literal with literal between first params',
        testFunction: function test (p1, p2, p3) {
          return `${p1}T1${p2}${p3}`
        }
      },
      {
        description: 'Template literal without literals and operations into expression',
        testFunction: function test (p1, p2, p3) {
          return `${p1}${p2 + p3}`
        }
      },
      {
        description: 'Template literal with literals at the beginning and operations into expression',
        testFunction: function test (p1, p2, p3) {
          return `T${p1}${p2 + p3}`
        }
      },
      {
        description: 'Template literal with literals at the end and operations into expression',
        testFunction: function test (p1, p2, p3) {
          return `${p1}${p2 + p3}T`
        }
      },
      {
        description: 'Template literal with literals between expressions and operations into expression',
        testFunction: function test (p1, p2, p3) {
          return `${p1}T${p2 + p3}`
        }
      }
    ]

    testFunctionsWith3Args.forEach(({ description, testFunction }) => {
      describe(description, () => {
        it('should return the same with string arguments', () => {
          rewriteAndCompare(testFunction, ['a', 'b', 'c'])
        })

        it('should return the same with number arguments', () => {
          rewriteAndCompare(testFunction, [1, 2, 3])
        })

        it('should return the same with number in strings', () => {
          rewriteAndCompare(testFunction, ['1', '2', '3'])
        })

        it('should return the same with string and number arguments', () => {
          rewriteAndCompare(testFunction, ['a', 'b', 3])
          rewriteAndCompare(testFunction, [1, 2, 'c'])
          rewriteAndCompare(testFunction, ['a', 2, 3])
          rewriteAndCompare(testFunction, [1, 'b', 'c'])
        })
      })
    })

    it('nested template literal with +', () => {
      const testFunction = function test (a, b) {
        return `Hello ${a} ${'how are u ' + `${'bye ' + b}`} world`
      }
      rewriteAndCompare(testFunction, [1, 2])
      rewriteAndCompare(testFunction, ['a', 'b'])
      rewriteAndCompare(testFunction, [1, 'b'])
      rewriteAndCompare(testFunction, ['a', 2])
    })

    it('issue 101', () => {
      // eslint-disable-next-line no-unused-vars
      const _ddiast = {
        plusOperator: (res) => res,
        tplOperator: (res) => res
      }

      const js = readFileSync(path.join(__dirname, 'resources/issue-101.js')).toString()

      // eslint-disable-next-line no-eval
      const issue101 = eval(js)

      const rewritten = rewriteAst(js)

      // eslint-disable-next-line no-eval
      const rewrittenIssue101 = eval(rewritten)

      expect(issue101.names(false)).to.be.equal(rewrittenIssue101.names(false))
      expect(issue101.namesBlock(false)).to.be.equal(rewrittenIssue101.namesBlock(false))
      expect(issue101.namesNested(false)).to.be.equal(rewrittenIssue101.namesNested(false))
      // expect(issue101.namesDoubleResult(false)).to.be.equal(rewrittenIssue101.namesDoubleResult(false))
      expect(issue101.namesNoFlag(false)).to.be.equal(rewrittenIssue101.namesNoFlag(false))
      // expect(issue101.namesNoFlagDoubleResult(false)).to.be.equal(rewrittenIssue101.namesNoFlagDoubleResult(false))
      expect(issue101.paren(false)).to.be.equal(rewrittenIssue101.paren(false))
    })
  })
})
