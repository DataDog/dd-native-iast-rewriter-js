/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

'use strict'

const chai = require('chai')
const sinon = require('sinon')
const sinonChai = require('sinon-chai')
const { Rewriter, getPrepareStackTrace } = require('../')
const path = require('path')
const Module = require('module')
const { addEditedFile } = require('./edited-files-cache')
let rewriter
let originalPrepareStackTrace = Error.prepareStackTrace

chai.use(sinonChai)

global.expect = chai.expect
global.sinon = sinon

initRewriter()

function initRewriter () {
  rewriter = new Rewriter()
  if (rewriter) {
    Object.defineProperty(global.Error, 'prepareStackTrace', getPrepareStackTraceAccessor())
    Module.prototype._compile = getCompileMethodFn(Module.prototype._compile)
  }
}
function getPrepareStackTraceAccessor () {
  let actual = getPrepareStackTrace(originalPrepareStackTrace)
  return {
    get () {
      return actual
    },
    set (value) {
      actual = getPrepareStackTrace(value)
      originalPrepareStackTrace = value
    }
  }
}

function getCompileMethodFn (compileMethod) {
  return function (content, filename) {
    try {
      if (filename.indexOf(path.join('integration-test', 'requires')) > -1) {
        content = rewriter.rewrite(content, filename)
        addEditedFile(filename)
      }
    } catch (e) {
      // eslint-disable-next-line no-console
      console.error(e)
    }
    return compileMethod.apply(this, [content, filename])
  }
}
