/** comments */
/** comments */
/** comments */

function functionToForceRewrite (a, b) {
  // just to force rewrite
  return a + b
}

const error = new Error()

function createError () {
  return new Error()
}

// eslint-disable-next-line no-eval
const evalError = eval('new Error()')

function createErrorInEval () {
  // eslint-disable-next-line no-eval
  return eval('new Error')
}

module.exports = {
  error,
  evalError,
  createError,
  createErrorInEval,
  functionToForceRewrite
}
