const { getSourcePathAndLineFromSourceMaps } = require('../source-map')

const kSymbolPrepareStackTrace = Symbol('_ddiastPrepareStackTrace')

class WrappedCallSite {
  constructor (callSite) {
    const { path, line, column } = getSourcePathAndLineFromSourceMaps(
      callSite.getFileName(),
      callSite.getLineNumber(),
      callSite.getColumnNumber()
    )
    this.source = path
    this.lineNumber = line
    this.columnNumber = column
    this.callSite = callSite
  }

  getThis () {
    return this.callSite
  }

  getTypeName () {
    return this.callSite.getTypeName()
  }

  getFunction () {
    return this.callSite.getFunction()
  }

  getFunctionName () {
    return this.callSite.getFunctionName()
  }

  getMethodName () {
    return this.callSite.getMethodName()
  }

  getFileName () {
    return this.source
  }

  getScriptNameOrSourceURL () {
    return null
  }

  getLineNumber () {
    return this.lineNumber
  }

  getColumnNumber () {
    return this.columnNumber
  }

  getEvalOrigin () {
    return this.callSite.getEvalOrigin()
  }

  isToplevel () {
    return this.callSite.isToplevel()
  }

  isEval () {
    return this.callSite.isEval()
  }

  isNative () {
    return this.callSite.isNative()
  }

  isConstructor () {
    return this.callSite.isConstructor()
  }
}

function getPrepareStackTrace (originalPrepareStackTrace) {
  if (originalPrepareStackTrace && originalPrepareStackTrace[kSymbolPrepareStackTrace]) {
    return originalPrepareStackTrace
  }

  const wrappedPrepareStackTrace = (error, structuredStackTrace) => {
    if (originalPrepareStackTrace) {
      const parsedCallSites = structuredStackTrace.map((callSite) => new WrappedCallSite(callSite))
      return originalPrepareStackTrace(error, parsedCallSites)
    }
    return error.stack
      .split('\n')
      .map((stackFrame) => {
        const start = stackFrame.indexOf('(/')
        if (start > -1) {
          const end = stackFrame.indexOf(')')
          const interesting = stackFrame.substring(start, end)
          const [filename, originalLine, originalColumn] = interesting.split(':')
          const { path, line, column } = getSourcePathAndLineFromSourceMaps(filename, originalLine, originalColumn)
          const startPart = stackFrame.substring(0, start)
          const endPart = stackFrame.substring(end)
          return `${startPart}${path}:${line}:${column}${endPart}`
        } else {
          return stackFrame
        }
      })
      .join('\n')
  }
  Object.defineProperty(wrappedPrepareStackTrace, kSymbolPrepareStackTrace, {
    value: true
  })
  return wrappedPrepareStackTrace
}

module.exports = {
  getPrepareStackTrace
}
