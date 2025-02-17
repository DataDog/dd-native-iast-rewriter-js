/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-multi-str */
/* eslint-disable no-unused-expressions */
'use strict'

const { expect } = require('chai')
const { Rewriter } = require('./util')

describe('logger', () => {
  let logger
  beforeEach(() => {
    logger = {
      error: sinon.spy(),
      debug: sinon.spy()
    }
  })

  afterEach(() => {
    sinon.restore()
  })

  it('should set logger as rewriter logger with ERROR level', () => {
    const rewriter = new Rewriter({ logLevel: 'ERROR', logger })

    rewriter.rewrite('function concat(a, b){ return a + b }', 'test.js')

    expect(logger.error).to.have.been.calledOnceWith('Wasm js rewriter logger configured OK')
  })

  it('should set logger as rewriter logger with DEBUG level', () => {
    const rewriter = new Rewriter({ localVarPrefix: 'logger-test', logLevel: 'DEBUG', logger })

    rewriter.rewrite('function concat(a, b){ return a + b }', 'test.js')

    expect(logger.debug).to.have.been.called
    expect(logger.debug.firstCall.args).to.deep.eq(['Wasm js rewriter logger configured OK'])
    expect(logger.debug.secondCall.args).to.deep.eq([
      'Rewriting js file: test.js with config: \
Config { chain_source_map: false, print_comments: false, local_var_prefix: "logger-test", csi_methods: \
CsiMethods { methods: [], plus_operator: None, tpl_operator: None, method_with_literal_callers: [] }, \
verbosity: Information, literals: true }'
    ])
  })

  it('should set console as the default logger', () => {
    const debug = sinon.stub(global.console, 'debug')

    const rewriter = new Rewriter({ localVarPrefix: 'logger-test', logLevel: 'DEBUG' })

    rewriter.rewrite('function concat(a, b){ return a + b }', 'test.js')

    expect(debug).to.have.been.called
    expect(debug.firstCall.args).to.deep.eq(['Wasm js rewriter logger configured OK'])
    expect(debug.secondCall.args).to.deep.eq([
      'Rewriting js file: test.js with config: \
Config { chain_source_map: false, print_comments: false, local_var_prefix: "logger-test", csi_methods: \
CsiMethods { methods: [], plus_operator: None, tpl_operator: None, method_with_literal_callers: [] }, \
verbosity: Information, literals: true }'
    ])
  })

  it('should not log any message', () => {
    const debug = sinon.stub(global.console, 'debug')

    const rewriter = new Rewriter({ logLevel: 'OFF' })

    rewriter.rewrite('function concat(a, b){ return a + b }', 'test.js')

    expect(debug).to.have.not.been.called
  })
})
