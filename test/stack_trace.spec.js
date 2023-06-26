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

  it('should not wrap an already wrapped prepareStackTrace', () => {
    const originalStackTrace = sinon.spy()
    const prepareStackTrace = getPrepareStackTrace(originalStackTrace)
    const anotherPrepareStackTrace = getPrepareStackTrace(prepareStackTrace)
    expect(prepareStackTrace).to.be.equals(anotherPrepareStackTrace)
  })
})

const sourceMapResourcesPath = path.join(__dirname, 'resources', 'stacktrace-sourcemap')
const nodeSourceMap = require('../js/source-map/node_source_map')
const { expect } = require('chai')
const readFileSync = function (filename) {
  if (filename.indexOf('.map') > 0) {
    return fs.readFileSync(path.join(sourceMapResourcesPath, path.basename(filename)))
  } else if (filename.indexOf('.js') > 0) {
    return fs.readFileSync(path.join(sourceMapResourcesPath, path.basename(filename)))
  }
}

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
    const fileName = 'test-file.js'
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, fileName),
      line: 5
    }

    const { getSourcePathAndLineFromSourceMaps, cacheRewrittenSourceMap } = proxyquire('../js/source-map', {
      './node_source_map': nodeSourceMap,
      fs: { readFileSync }
    })

    const fileContent = fs.readFileSync(path.join(sourceMapResourcesPath, fileName)).toString()
    cacheRewrittenSourceMap(originalPathAndLine.path, fileContent)

    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 12)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-file.ts'))
    expect(pathAndLine.line).to.be.equals(2)
  })

  it('should translate with inlined map', () => {
    const fileName = 'test-inline.js'
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, fileName),
      line: 5
    }
    const { getSourcePathAndLineFromSourceMaps, cacheRewrittenSourceMap } = proxyquire('../js/source-map', {
      './node_source_map': nodeSourceMap,
      fs: { readFileSync }
    })

    const fileContent = fs.readFileSync(path.join(sourceMapResourcesPath, fileName)).toString()
    cacheRewrittenSourceMap(originalPathAndLine.path, fileContent)

    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 10)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-inline.ts'))
    expect(pathAndLine.line).to.be.equals(2)
  })

  it('should translate minified file with correct column', () => {
    const fileName = 'test-min.min.js'
    const originalPathAndLine = {
      path: path.join(sourceMapResourcesPath, fileName),
      line: 1
    }
    const { getSourcePathAndLineFromSourceMaps, cacheRewrittenSourceMap } = proxyquire('../js/source-map', {
      './node_source_map': nodeSourceMap,
      fs: { readFileSync }
    })

    const fileContent = fs.readFileSync(path.join(sourceMapResourcesPath, fileName)).toString()
    cacheRewrittenSourceMap(originalPathAndLine.path, fileContent)

    const pathAndLine = getSourcePathAndLineFromSourceMaps(originalPathAndLine.path, originalPathAndLine.line, 23)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-min.js'))
    expect(pathAndLine.line).to.be.equals(2)
  })
})

describe('getOriginalPathAndLine', () => {
  const sourceMapsMap = new Map()
  const setLRU = sinon.spy()
  const getLRU = sinon.spy()
  class LRU {
    set (key, value) {
      setLRU(key, value)
      return sourceMapsMap.set(key, value)
    }

    get (key) {
      getLRU(key)
      return sourceMapsMap.get(key)
    }
  }

  const { getOriginalPathAndLine } = proxyquire('../js/source-map', {
    'lru-cache': LRU,
    './node_source_map': nodeSourceMap,
    fs: {
      readFileSync: (filename) => {
        switch (filename) {
          case 'no-sourcemap':
            return ''
          case 'error':
            throw new Error('errr')
          default:
            return readFileSync(filename).toString()
        }
      }
    }
  })

  const fileName = 'test-inline.js'
  const originalPathAndLine = {
    path: path.join(sourceMapResourcesPath, fileName),
    line: 5
  }

  afterEach(() => {
    sinon.reset()
    sourceMapsMap.clear()
  })

  it('should add SourceMap to cache', () => {
    const pathAndLine = getOriginalPathAndLine(originalPathAndLine.path, originalPathAndLine.line, 12)

    expect(setLRU).to.be.calledOnceWith(originalPathAndLine.path)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-inline.ts'))
    expect(pathAndLine.line).to.be.equals(2)
  })

  it('should reuse previously cached SourceMap', () => {
    const pathAndLine = getOriginalPathAndLine(originalPathAndLine.path, originalPathAndLine.line, 12)

    getOriginalPathAndLine(originalPathAndLine.path, originalPathAndLine.line, 12)
    getOriginalPathAndLine(originalPathAndLine.path, originalPathAndLine.line, 12)

    expect(setLRU).to.be.calledOnceWith(originalPathAndLine.path)
    expect(pathAndLine.path).to.be.equals(path.join(sourceMapResourcesPath, 'test-inline.ts'))
    expect(pathAndLine.line).to.be.equals(2)
  })

  it('should not try to load again a previously failed SourceMap', () => {
    getOriginalPathAndLine('no-sourcemap', originalPathAndLine.line, 12)
    getOriginalPathAndLine('no-sourcemap', originalPathAndLine.line, 12)
    getOriginalPathAndLine('no-sourcemap', originalPathAndLine.line, 12)

    expect(setLRU).to.be.calledOnceWith('no-sourcemap', null)
  })

  it('should return the original path and line if there is an Error', () => {
    const pathAndLine = getOriginalPathAndLine('error', 42, 12)
    expect(pathAndLine.path).to.be.equals('error')
    expect(pathAndLine.line).to.be.equals(42)
  })
})
