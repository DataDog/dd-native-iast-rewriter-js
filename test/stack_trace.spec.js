/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

'use strict'

const fs = require('fs')
const path = require('path')
const proxyquire = require('proxyquire')

const { getPrepareStackTrace } = require('../js/stack-trace')
const { getSourcePathAndLineFromSourceMaps } = require('../js/source-map')

class CallSiteMock {
  constructor (fileName, lineNumber, columnNumber) {
    this.fileName = fileName
    this.lineNumber = lineNumber
    this.columnNumber = columnNumber
  }

  getColumnNumber () {
    return this.columnNumber
  }

  getLineNumber () {
    return this.lineNumber
  }

  getFileName () {
    return this.fileName
  }

  isNative () {
    return false
  }
}

describe('V8 prepareStackTrace', () => {
  const TEST_PATH = ['test', 'packages', 'dist', 'server', 'app.js'].join(path.sep)
  const TEST_LINE = 99
  const TEST_COLUMN = 15

  it('should call original prepareStackTrace', () => {
    const originalStackTrace = sinon.spy()
    const prepareStackTrace = getPrepareStackTrace(originalStackTrace)
    const callsites = []
    callsites.push(new CallSiteMock(TEST_PATH, TEST_LINE, TEST_COLUMN))
    prepareStackTrace(null, callsites)
    // eslint-disable-next-line no-unused-expressions
    expect(originalStackTrace).to.be.calledOnce
  })
})

const sourceMapResourcesPath = path.join(__dirname, 'resources', 'stacktrace-sourcemap')

describe('getFilenameFromSourceMap', () => {
  it('should return original object if file does not exist', () => {
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, 'does-not-exist.js'),
      line: 12
    }
    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 0)
    expect(pathAndLine.path).to.be.equals(originalPathAndLine.path)
    expect(pathAndLine.line).to.be.equals(originalPathAndLine.line)
  })

  it('should translate with map file', () => {
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, 'test-file.js'),
      line: 5
    }
    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 12)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-file.ts'))
    expect(pathAndLine.line).to.be.equals(2)
  })

  it('should translate with inlined map', () => {
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, 'test-inline.js'),
      line: 5
    }
    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-inline.ts'))
    expect(pathAndLine.line).to.be.equals(2)
  })

  it('should translate minified file with correct column', () => {
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, 'test-min.min.js'),
      line: 1
    }
    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 23)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-min.js'))
    expect(pathAndLine.line).to.be.equals(2)
  })
})

describe('getFilenameFromSourceMap cache', () => {
  const nodeSourceMap = require('../js/source-map/node_source_map')
  afterEach(() => {
    sinon.restore()
  })

  it('should not create two SourceMap file for the same file', () => {
    const sourceMapSpy = sinon.spy(nodeSourceMap, 'SourceMap')
    const { getSourcePathAndLineFromSourceMaps } = proxyquire('../js/source-map', {
      './node_source_map': nodeSourceMap
    })

    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, 'test-file.js'),
      line: 5
    }
    getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 0)
    getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 0)
    // eslint-disable-next-line no-unused-expressions
    expect(sourceMapSpy).to.have.been.calledOnce
  })

  it('should has a maximum cached items', () => {
    const sourceMapSpy = sinon.spy(nodeSourceMap, 'SourceMap')
    const readFileSync = function (filename) {
      if (filename.indexOf('.map') > 0) {
        return fs.readFileSync(path.join(sourceMapResourcesPath, 'test-file.js.map'))
      } else if (filename.indexOf('.js') > 0) {
        return fs.readFileSync(path.join(sourceMapResourcesPath, 'test-file.js'))
      }
    }

    const { getSourcePathAndLineFromSourceMaps } = proxyquire('../js/source-map', {
      './node_source_map': nodeSourceMap,
      fs: { readFileSync }
    })

    for (let i = 0; i < 101; i++) {
      getSourcePathAndLineFromSourceMaps(`/source/file-${i}.js`, 5) // new entry
      getSourcePathAndLineFromSourceMaps(`/source/file-${i}.js`, 5) // from cache
    }
    expect(sourceMapSpy).to.have.been.callCount(101)
    getSourcePathAndLineFromSourceMaps('/source/file-0.js', 5)
    expect(sourceMapSpy).to.have.been.callCount(102)
  })
})
