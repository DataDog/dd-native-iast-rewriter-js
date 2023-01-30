/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

const os = require('os')
const path = require('path')

const rewriterPackage = process.env.NPM_REWRITER === 'true' ? '@datadog/native-iast-rewriter' : '../main'
const { Rewriter } = require(rewriterPackage)

const TELEMETRY_VERBOSITY = 'Debug'

const removeSourceMap = (code) => {
  return code
    .split('\n')
    .filter((l) => !l.trim().startsWith('//# sourceMappingURL='))
    .join('\n')
}

const csiMethods = [
  { src: 'plusOperator', operator: true },
  { src: 'substring', dst: 'stringSubstring' },
  { src: 'trim' },
  { src: 'trimStart' },
  { src: 'trimEnd' },
  { src: 'toLowerCase' },
  { src: 'toLocaleLowerCase' },
  { src: 'toUpperCase' },
  { src: 'toLocaleUpperCase' },
  { src: 'replace' },
  { src: 'replaceAll' },
  { src: 'slice' },
  { src: 'concat' }
]

const rewriteAst = (code, opts) => {
  opts = opts || {}
  const rewriter =
    opts.rewriter ??
    new Rewriter({
      localVarPrefix: 'test',
      csiMethods,
      chainSourceMap: opts.chainSourceMap ?? false,
      telemetryVerbosity: TELEMETRY_VERBOSITY
    })
  const file = opts.file ?? path.join(process.cwd(), 'index.spec.js')
  const rewritten = rewriter.rewrite(code, file)
  return opts.keepSourceMap ? rewritten.content : removeSourceMap(rewritten.content)
}

const wrapBlock = (code) => `{${os.EOL}${code}${os.EOL}}`

const rewriteAndExpectNoTransformation = (code, opts) => {
  rewriteAndExpect(wrapBlock(code), wrapBlock(code), true, opts)
}

const rewriteAndExpect = (code, expect, block, opts) => {
  code = !block ? `{${code}}` : code
  const rewrited = rewriteAst(code, opts)
  expectAst(rewrited, expect)
}

const rewriteAndExpectError = (code) => {
  expect(() => {
    rewriteAndExpect(code, code)
  }).to.throw(Error, /Variable name duplicated/)
}

const GLOBAL_METHODS_TEMPLATE = `;(function(globals){
  globals._ddiast = globals._ddiast || { __CSI_METHODS__ };
}((1,eval)('this')));`

const getGlobalMethods = function (methods) {
  const fnSignAndBody = '(res) {return res;}'
  return GLOBAL_METHODS_TEMPLATE.replace('__CSI_METHODS__', methods.join(`${fnSignAndBody},`) + fnSignAndBody)
}

const expectAst = (received, expected) => {
  const rLines = received
    .split('\n') // it seems that rewriter do not take into account OS line endings
    .map((l) => l.trim())
    .join('\n')
  const eLines = expected
    .split('\n')
    .map((l) => l.trim())
    .join('\n')

  expect(rLines).to.be.eq(eLines)
}

const rewriteAndExpectAndExpectEval = (js, expected) => {
  const rewriter = new Rewriter({ localVarPrefix: 'test', csiMethods, telemetryVerbosity: TELEMETRY_VERBOSITY })
  rewriteAndExpect(js, expected, true, { rewriter })

  const globalMethods = getGlobalMethods(rewriter.csiMethods())

  // eslint-disable-next-line no-eval
  expect(eval(js)).equal(eval(`${globalMethods}\n${expected}`))
}

const alphabet = Array.from(Array(26)).map((e, i) => String.fromCharCode(i + 97))
class FnBuilder {
  constructor () {
    this.tmpl = `(function(__ARGS_SIGN__) {
    __BODY__
  })(__ARGS__);
  `
  }

  body (body) {
    this.bodyValue = body
    return this
  }

  args (...value) {
    this.argsValue = value
    return this
  }

  build (body) {
    if (body) {
      this.body(body)
    }
    if (!this.argsValue) {
      this.argsValue = []
    }

    return this.tmpl
      .replace('__ARGS_SIGN__', alphabet.slice(0, this.argsValue.length).join(', '))
      .replace('__BODY__', this.bodyValue)
      .replace(
        '__ARGS__',
        Array.from(this.argsValue)
          .map((arg) => {
            switch (typeof arg) {
              case 'string':
                return `'${arg}'`
              case 'object':
                return JSON.stringify(arg, null, 2)
              case 'function':
                return arg.toString().replace(/ /g, '')
              default:
                return arg
            }
          })
          .join(', ')
      )
  }
}

const fn = () => new FnBuilder()

module.exports = {
  rewriteAst,
  rewriteAndExpectNoTransformation,
  rewriteAndExpect,
  rewriteAndExpectError,
  wrapBlock,
  Rewriter,
  csiMethods,
  rewriteAndExpectAndExpectEval,
  fn
}
