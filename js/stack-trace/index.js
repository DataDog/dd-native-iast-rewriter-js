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
    const splittedStack = error.stack.split('\n')
    let firstIndex = -1
    for (let i = 0; i < splittedStack.length; i++) {
      if (splittedStack[i].match(/^\s*at/gm)) {
        firstIndex = i
        break
      }
    }
    return splittedStack
      .map((stackFrame, index) => {
        if (index < firstIndex) {
          return stackFrame
        }
        index = index - firstIndex
        if (!structuredStackTrace[index]) {
          return stackFrame
        }
        const filename = structuredStackTrace[index].getFileName()
        const originalLine = structuredStackTrace[index].getLineNumber()
        const originalColumn = structuredStackTrace[index].getColumnNumber()
        const { path, line, column } = getSourcePathAndLineFromSourceMaps(filename, originalLine, originalColumn)
        if (path !== filename || line !== originalLine || column !== originalColumn) {
          return stackFrame.replace(`${filename}:${originalLine}:${originalColumn}`, `${path}:${line}:${column}`)
        }
        return stackFrame
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
