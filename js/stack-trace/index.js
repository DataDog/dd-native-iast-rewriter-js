const { getSourcePathAndLineFromSourceMaps } = require('../source-map')

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
  return (error, structuredStackTrace) => {
    const parsedCallSites = structuredStackTrace.map((callSite) => new WrappedCallSite(callSite))
    return originalPrepareStackTrace(error, parsedCallSites)
  }
}

module.exports = {
  getPrepareStackTrace
}
