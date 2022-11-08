/* eslint no-unused-expressions: 0 */
const { isFileEdited } = require('./edited-files-cache')

describe('Test stack traces', () => {
  describe('detects line and column correctly', () => {
    it('when error is created out of the methods', () => {
      const { error } = require('./requires/errors')
      const firstStackLine = error.stack.split('\n')[1]
      expect(firstStackLine).to.contain('errors.js:9:15')
      expect(isFileEdited('errors.js')).to.be.true
    })
    it('when error is created out of the methods in eval', () => {
      const { evalError } = require('./requires/errors')
      const firstStackLine = evalError.stack.split('\n')[1]
      expect(firstStackLine).to.contain('errors.js:16:19')
      expect(isFileEdited('errors.js')).to.be.true
    })

    it('when error is created in one method', () => {
      const { createError } = require('./requires/errors')
      const error = createError()
      const firstStackLine = error.stack.split('\n')[1]
      expect(firstStackLine).to.contain('errors.js:12:10')
      expect(isFileEdited('errors.js')).to.be.true
    })

    it('when error is created in one method in eval', () => {
      const { createErrorInEval } = require('./requires/errors')
      const error = createErrorInEval()
      const firstStackLine = error.stack.split('\n')[1]
      expect(firstStackLine).to.contain('errors.js:20:10')
      expect(isFileEdited('errors.js')).to.be.true
    })
  })
})
