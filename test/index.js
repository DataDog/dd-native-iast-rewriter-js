const { expect } = require('chai')
const { it, describe } = require('mocha')
const path = require('path')

const { Rewriter } = require('../index')

const rewriteAst = (code, opts) => {
  opts = opts || {};
  const rewriter = opts.rewriter ?? new Rewriter()
  const file = opts.file ?? path.join(process.cwd(), 'index.js')
  const sourceMap = opts.sourceMap
  return rewriter.rewrite(code, file, sourceMap)
}

const removeBlock = (code) => {
  return code.replace(/^\{\n*\t*/, '').replace(/\n*\t*\}$/, '')
}

const rewriteAndExpectBlock = (code, expect) => {
  const rewrited = rewriteAst(`{${code}}`);
  expectAst(removeBlock(rewrited), removeBlock(expect));
}

const expectAst = (received, expected) => {
  expect(received.trim()).to.be.equal(expected.trim())
}

describe('binary expression', () => {
  it('sub', () => {
      const js = 'const result = a - " hey!";'
      rewriteAndExpectBlock(js, js);
  })

  it('add', () => {
      const js = 'const result = a + " hey!";'
      rewriteAndExpectBlock(js, 'const result = (global._ddiast.twoItemsPlusOperator(a + " hey!", a, " hey!"));')
  })

})