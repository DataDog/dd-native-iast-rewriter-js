/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

const os = require('os')
const path = require('path')

const rewriterPackage = process.env.NPM_REWRITER === 'true' ? '@datadog/native-iast-rewriter' : '../index'
const { Rewriter } = require(rewriterPackage)

const removeSourceMap = (code) => {
  return code
    .split('\n')
    .filter((l) => !l.trim().startsWith('//# sourceMappingURL='))
    .join('\n')
}

const rewriteAst = (code, opts) => {
  opts = opts || {}
  const rewriter = opts.rewriter ?? new Rewriter({ chainSourceMap: opts.chainSourceMap ?? false })
  const file = opts.file ?? path.join(process.cwd(), 'index.spec.js')
  const rewrited = rewriter.rewrite(code, file)
  return opts.keepSourceMap ? rewrited : removeSourceMap(rewrited)
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

module.exports = {
  rewriteAst,
  rewriteAndExpectNoTransformation,
  rewriteAndExpect,
  rewriteAndExpectError,
  wrapBlock,
  Rewriter
}
