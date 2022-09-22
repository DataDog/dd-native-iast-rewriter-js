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
    rewriteAndExpect(js, '{\nconst result = global._ddiast.plusOperator(`Hello${a}World!`, `Hello`, a, `World!`);\n}')
  })

  it('start', () => {
    const js = 'const result = `${a}Hello World!`;'
    rewriteAndExpect(js, '{\nconst result = global._ddiast.plusOperator(`${a}Hello World!`, a, `Hello World!`);\n}')
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
    res.send((__datadog_test_0 = req.query.param, global._ddiast.plusOperator(\`<html lang="en">
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
    rewriteAndExpect(js, '{\nconst result = global._ddiast.plusOperator(`Hello World!${a}`, `Hello World!`, a);\n}')
  })

  it('with binary operations', () => {
    const js = 'const result = `Hello World!${a + b}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nconst result = (__datadog_test_0 = \
global._ddiast.plusOperator(a + b, a, b), global._ddiast.plusOperator(`Hello World!${__datadog_test_0}`, \
`Hello World!`, __datadog_test_0));\n}'
    )
  })

  it('with call', () => {
    const js = 'const result = `Hello World!${a()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\nconst result = (__datadog_test_0 = a(), \
global._ddiast.plusOperator(`Hello World!${__datadog_test_0}`, `Hello World!`, __datadog_test_0));\n}'
    )
  })

  it('with binary operations and call', () => {
    const js = 'const result = `Hello World!${a + b()}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1;\nconst result = (__datadog_test_1 = \
(__datadog_test_0 = b(), global._ddiast.plusOperator(a + __datadog_test_0, a, __datadog_test_0)), \
global._ddiast.plusOperator(`Hello World!${__datadog_test_1}`, `Hello World!`, __datadog_test_1));\n}'
    )
  })

  it('with binary operations and object property access', () => {
    const js = 'const result = `Hello World!${a + b.x}`;'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0, __datadog_test_1;\nconst result = \
(__datadog_test_1 = (__datadog_test_0 = b.x, global._ddiast.plusOperator(a + __datadog_test_0, a, __datadog_test_0))\
, global._ddiast.plusOperator(`Hello World!${__datadog_test_1}`, `Hello World!`, __datadog_test_1));\n}'
    )
  })

  it('inside if test', () => {
    const js = 'const c = a === `Hello${b}` ? "world" : "moon";'
    rewriteAndExpect(
      js,
      '{\nconst c = a === global._ddiast.plusOperator(`Hello${b}`, `Hello`, b) ? \
"world" : "moon";\n}'
    )
  })

  it('inside if cons', () => {
    const js = 'const c = a === "hello" ? `World ${b}` : "Moon";'
    rewriteAndExpect(
      js,
      '{\nconst c = a === "hello" ? global._ddiast.plusOperator(`World ${b}`, \
`World `, b) : "Moon";\n}'
    )
  })

  it('typeof is considered literal', () => {
    const js = 'const a = `He${typeof b}llo`;'
    rewriteAndExpectNoTransformation(js)
  })

  it('typeof among variables is replaced by a variable', () => {
    const js = 'const a = `He${typeof b}llo wor${a}ld`'
    rewriteAndExpect(
      js,
      '{\nlet __datadog_test_0;\n    const a = (__datadog_test_0 = typeof b, global._ddiast.plusOperator(\
`He${__datadog_test_0}llo wor${a}ld`, `He`, __datadog_test_0, `llo wor`, a, `ld`));\n}'
    )
  })

  it('tagged are not mofified', () => {
    const js = 'const a = func`Hello${b}World`;'
    rewriteAndExpectNoTransformation(js)
  })

  it('tagged with child expressions are mofified', () => {
    const js = 'const a = func`Hello${b + c}World`;'
    rewriteAndExpect(js, '{\nconst a = func`Hello${global._ddiast.plusOperator(b + c, b, c)}World`;\n}')
  })

  it('nested template literal with +', () => {
    const js = "const a = `Hello ${c} ${'how are u ' + `${'bye ' + d}`} world`;"
    rewriteAndExpect(
      js,
      "{\nlet __datadog_test_0, __datadog_test_1;\nconst a = (__datadog_test_1 = (__datadog_test_0 \
= `${global._ddiast.plusOperator('bye ' + d, 'bye ', d)}`, global._ddiast.plusOperator('how are u ' + \
__datadog_test_0, 'how are u ', __datadog_test_0)), global._ddiast.plusOperator(`Hello ${c} ${__datadog_test_1} \
world`, `Hello `, c, ` `, __datadog_test_1, ` world`));\n}"
    )
  })
})