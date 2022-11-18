/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

const babelParser = require('@babel/parser')
const babelParse = babelParser.parse
const { parse, print } = require('recast')
const { visit } = require('ast-types')
function Rewriter () {}
Rewriter.prototype.rewrite = function (code) {
  const ast = this.parse(code)
  //*
  visit(ast, {
    visitBinaryExpression: function (path) {
      if (path.value.type === 'BinaryExpression' && path.value.operator === '+') {
        path.value.operator = '+'
      }
      this.traverse(path)
    }
  })
  //* /
  const result = print(ast, { wrapColumn: Number.MAX_SAFE_INTEGER, sourceMapName: 'mapname' })
  return result.code
}

Rewriter.prototype.parse = function (code) {
  return parse(code, {
    parser: {
      parse (source) {
        const ast = babelParse(source, {
          createParenthesizedExpressions: true,
          sourceType: 'module',
          strictMode: false,
          plugins: [
            'classProperties',
            'classPrivateProperties',
            'classPrivateMethods',
            ['decorators', { decoratorsBeforeExport: true }],
            'throwExpressions',
            'topLevelAwait'
          ]
        })
        ast.tokens = []
        return ast
      }
    }
  })
}
module.exports = {
  Rewriter
}
