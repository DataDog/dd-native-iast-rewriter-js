const { it, describe } = require('mocha')
const { expect } = require('chai')

describe('empty test', () => {
    it('\'hello\' should be equals to \'hello\'', () => {
      expect('hello').to.be.equal('hello')
    })
})

