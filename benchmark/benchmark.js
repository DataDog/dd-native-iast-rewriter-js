/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

/* eslint no-console: 0 */
'use strict'
const fs = require('fs')
const path = require('path')
const pathToTest = path.join(__dirname, '..', 'node_modules')
const MAX_FILES = 200
console.log('WARMUP Start')
for (let i = 0; i < 2; i++) {
  iterateDeepJsInFolder(pathToTest, { rewrite: () => {} })
}
const filesToRewrite = getFileList(pathToTest)
console.log('WARMUP End')

function getFileList (dir) {
  const list = []
  iterateDeepJsInFolder(dir, {
    rewrite: (content, path) => {
      if (list.length > 364) {
        console.log({ path })
      }
      list.push({ content, path })
    }
  })
  return list
}

function iterateDeepJsInFolder (dir, rewriter, list = []) {
  const dirContent = fs.readdirSync(dir)
  for (let i = 0; i < dirContent.length; i++) {
    if (list.length >= MAX_FILES) {
      break
    }
    const fullpath = path.join(dir, dirContent[i])
    if (dirContent[i].endsWith('.js')) {
      rewriter.rewrite(fs.readFileSync(fullpath).toString(), fullpath)
      list.push(fullpath)
    } else {
      const stats = fs.statSync(fullpath)
      if (stats.isDirectory()) {
        iterateDeepJsInFolder(fullpath, rewriter, list)
      }
    }
  }
}

function rewriteFilesToRewrite (rewriter) {
  for (let i = 0; i < filesToRewrite.length; i++) {
    const { content, path } = filesToRewrite[i]
    rewriter.rewrite(content, path)
  }
}

function testDefault () {
  console.log('start default')
  const { Rewriter } = require('../main')
  const start = process.hrtime.bigint()
  const rewriter = new Rewriter()
  rewriteFilesToRewrite(rewriter)
  const end = process.hrtime.bigint()
  const totalTime = end - start
  const ms = totalTime / 1000000n
  console.log('Default result', ms + 'ms')
}

function testWasm () {
  console.log('start wasm')
  const { Rewriter } = require('../wasm/wasm_js_rewriter.js')
  const start = process.hrtime.bigint()
  const rewriter = new Rewriter()
  rewriteFilesToRewrite(rewriter)
  const end = process.hrtime.bigint()
  const totalTime = end - start
  const ms = totalTime / 1000000n
  console.log('WASM result', ms + 'ms')
}

function testBabel () {
  console.log('start babel')
  const { Rewriter } = require('./babel-rewrite')
  const start = process.hrtime.bigint()
  const rewriter = new Rewriter()
  rewriteFilesToRewrite(rewriter)
  const end = process.hrtime.bigint()
  const totalTime = end - start
  const ms = totalTime / 1000000n
  console.log('babel result', ms + 'ms')
}

function test () {
  testWasm()
  testBabel()
  testDefault()
}
test()
