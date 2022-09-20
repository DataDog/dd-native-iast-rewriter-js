/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-template-curly-in-string */
/* eslint-disable no-multi-str */
const { rewriteAndExpectNoTransformation, rewriteAndExpect } = require('./util')

describe('template literal', () => {
  it('empty', () => {
    const js = 'const result = `Hello World!`;'
    rewriteAndExpectNoTransformation(js)
  })

  it('literal', () => {
    const js = 'const result = `Hello${" "}World!`;'
    rewriteAndExpectNoTransformation(js)
  })

  it('middle', () => {
    const js = 'const result = `Hello${a}World!`;'
    rewriteAndExpect(js, '{\nconst result = _ddiast.plusOperator(`Hello${a}World!`, `Hello`, a, `World!`);\n}')
  })

  it('start', () => {
    const js = 'const result = `${a}Hello World!`;'
    rewriteAndExpect(js, '{\nconst result = _ddiast.plusOperator(`${a}Hello World!`, a, `Hello World!`);\n}')
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
    res.send((__datadog_test_0 = req.query.param, _ddiast.plusOperator(\`<html lang="en">
    <body>
        <h1>XSS vulnerability</h1>
        <p>Received param: \${__datadog_test_0}</p>
    </body>
</body>
</html>\`, \`<html lang="en">
    <body>
        <h1>XSS vulnerability</h1>
        <p>Received param: \`, __datadog_test_0, \`</p>
    </body>
</body>
</html>\`)));
});\n}`
    rewriteAndExpect(js, expected)
  })

  it('end', () => {
    const js = 'const result = `Hello World!${a}`;'
    rewriteAndExpect(js, '{\nconst result = _ddiast.plusOperator(`Hello World!${a}`, `Hello World!`, a);\n}')
  })

  it('with binary operations', () => {
    const js = 'const result = `Hello World!${a + b}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(\
`Hello World!${__datadog_test_2}`, `Hello World!`, __datadog_test_2));\n}'
    )
  })

  it('with call', () => {
    const js = 'const result = `Hello World!${a()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nconst result = (__datadog_test_0 = a(), \
_ddiast.plusOperator(`Hello World!${__datadog_test_0}`, `Hello World!`, __datadog_test_0));\n}'
    )
  })

  it('with binary operations and call', () => {
    const js = 'const result = `Hello World!${a + b()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(\
`Hello World!${__datadog_test_2}`, `Hello World!`, __datadog_test_2));\n}'
    )
  })

  it('with binary operations and object property access', () => {
    const js = 'const result = `Hello World!${a + b.x}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b.x, _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(\
`Hello World!${__datadog_test_2}`, `Hello World!`, __datadog_test_2));\n}'
    )
  })

  it('with binary operations and chained object property access', () => {
    const js = 'const result = `Hello World!${a + b.x.y.z}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b.x.y.z, _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(\
`Hello World!${__datadog_test_2}`, `Hello World!`, __datadog_test_2));\n}'
    )
  })

  it('inside if test', () => {
    const js = 'const c = a === `Hello${b}` ? "world" : "moon";'
    rewriteAndExpect(js, '{\nconst c = a === _ddiast.plusOperator(`Hello${b}`, `Hello`, b) ? \
"world" : "moon";\n}')
  })

  it('inside if cons', () => {
    const js = 'const c = a === "hello" ? `World ${b}` : "Moon";'
    rewriteAndExpect(js, '{\nconst c = a === "hello" ? _ddiast.plusOperator(`World ${b}`, \
`World `, b) : "Moon";\n}')
  })

  it('typeof is considered literal', () => {
    const js = 'const a = `He${typeof b}llo`;'
    rewriteAndExpectNoTransformation(js)
  })

  it('typeof among variables is replaced by a variable', () => {
    const js = 'const a = `He${typeof b}llo wor${a}ld`'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\n    const a = (__datadog_test_0 = typeof b, _ddiast.plusOperator(\
`He${__datadog_test_0}llo wor${a}ld`, `He`, __datadog_test_0, `llo wor`, a, `ld`));\n}'
    )
  })

  it('tagged are not mofified', () => {
    const js = 'const a = func`Hello${b}World`;'
    rewriteAndExpectNoTransformation(js)
  })

  it('tagged with child expressions are mofified', () => {
    const js = 'const a = func`Hello${b + c}World`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1;\n\
      const a = func`Hello${(__datadog_test_0 = b, __datadog_test_1 = c, _ddiast.plusOperator(__datadog_test_0 + \
__datadog_test_1, __datadog_test_0, __datadog_test_1))}World`;\n}'
    )
  })

  it('nested template literal with +', () => {
    const js = "const a = `Hello ${c} ${'how are u ' + `${'bye ' + d}`} world`;"
    rewriteAndExpect(
      js,
      "{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;\n\
        const a = (__datadog_test_3 = (__datadog_test_2 = (__datadog_test_1 = (__datadog_test_0 = d, \
_ddiast.plusOperator('bye ' + __datadog_test_0, 'bye ', __datadog_test_0)), _ddiast.plusOperator(\
`${__datadog_test_1}`, __datadog_test_1)), _ddiast.plusOperator('how are u ' + __datadog_test_2, 'how are u ', \
__datadog_test_2)), _ddiast.plusOperator(`Hello ${c} ${__datadog_test_3} world`, `Hello `, c, ` `, __datadog_test_3, \
` world`));\n}"
    )
  })

  it('with update expression ++', () => {
    const js = 'const a = `Hello ${c++}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nconst a = (__datadog_test_0 = c++, _ddiast.plusOperator(`Hello \
${__datadog_test_0}`, `Hello `, __datadog_test_0));\n}'
    )
  })

  it('with update expression --', () => {
    const js = 'const a = `Hello ${--c}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nconst a = (__datadog_test_0 = --c, _ddiast.plusOperator(`Hello \
${__datadog_test_0}`, `Hello `, __datadog_test_0));\n}'
    )
  })

  it('with await expression', () => {
    const js = 'const a = `Hello ${await b()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nconst a = (__datadog_test_0 = await b(), _ddiast.plusOperator(`Hello \
${__datadog_test_0}`, `Hello `, __datadog_test_0));\n}'
    )
  })

  it('with await expression inside +', () => {
    const js = 'const a = `Hello ${b + await c()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const a = (__datadog_test_2 = (__datadog_test_0 = b, __datadog_test_1 = await c(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(`Hello \
${__datadog_test_2}`, `Hello `, __datadog_test_2));\n}'
    )
  })

  it('with conditional', () => {
    const js = 'const a = `Hello ${b + c ? d : e}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const a = (__datadog_test_2 = (__datadog_test_0 = b, __datadog_test_1 = c, _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)) ? d : e, _ddiast.plusOperator(\
`Hello ${__datadog_test_2}`, `Hello `, __datadog_test_2));\n}'
    )
  })

  it('with new', () => {
    const js = 'const a = `Hello ${a + new B()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1, __datadog_test_2;\n\
        const a = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = new B(), _ddiast.plusOperator(\
__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(\
`Hello ${__datadog_test_2}`, `Hello `, __datadog_test_2));\n}'
    )
  })
})
