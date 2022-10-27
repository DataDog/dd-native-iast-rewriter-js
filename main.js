const { Rewriter } = require('./index')
const { getPrepareStackTrace } = require('./js/stack-trace/')

module.exports = {
  Rewriter: Rewriter,
  getPrepareStackTrace: getPrepareStackTrace
}
