/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-unused-expressions */

const path = require('path')
const { expect } = require('chai')
const { rewriteWithOpts } = require('./util')

const FILE_PATH = path.join(process.cwd(), 'index.spec.js')

describe('hardcoded secrets', () => {
  it('does double quoted literals found', () => {
    const js = 'const secret = "this_is_a_secret";'
    const result = rewriteWithOpts(js)

    expect(result.hardcodedSecretResult).to.not.undefined
    expect(result.hardcodedSecretResult.file).to.be.eq(FILE_PATH)
    expect(result.hardcodedSecretResult.literals).to.deep.eq(['this_is_a_secret'])
  })

  it('does not found literals if disabled by conf', () => {
    const js = 'const secret = "this_is_a_secret";'
    const result = rewriteWithOpts(js, {
      hardcodedSecret: false
    })

    expect(result.hardcodedSecretResult).to.undefined
  })

  it('does return single quoted literals found', () => {
    const js = "const secret = 'this_is_a_secret';"
    const result = rewriteWithOpts(js)

    expect(result.hardcodedSecretResult).to.not.undefined
    expect(result.hardcodedSecretResult.file).to.be.eq(FILE_PATH)
    expect(result.hardcodedSecretResult.literals).to.deep.eq(['this_is_a_secret'])
  })

  it('does return multiple literals found', () => {
    const js = "const secret1 = 'this_is_a_secret'; const secret2 = 'another_secret'"
    const result = rewriteWithOpts(js)

    expect(result.hardcodedSecretResult).to.not.undefined
    expect(result.hardcodedSecretResult.literals).to.deep.eq(['this_is_a_secret', 'another_secret'])
  })

  it('does return literals found inside a block', () => {
    const js = "function auth() { const secret = 'this_is_a_secret'; }"
    const result = rewriteWithOpts(js)

    expect(result.hardcodedSecretResult).to.not.undefined
    expect(result.hardcodedSecretResult.literals).to.deep.eq(['this_is_a_secret'])
  })

  it('does return parameter literals in a call', () => {
    const js = "function login() { return auth('this_is_a_secret'); }"
    const result = rewriteWithOpts(js)

    expect(result.hardcodedSecretResult).to.not.undefined
    expect(result.hardcodedSecretResult.literals).to.deep.eq(['this_is_a_secret'])
  })

  it('does not return literals with less or eq than 8 chars length', () => {
    const js = 'const secret = "12345678";'
    const result = rewriteWithOpts(js)

    expect(result.hardcodedSecretResult).to.not.undefined
    expect(result.hardcodedSecretResult.literals).to.deep.eq([])
  })
})
