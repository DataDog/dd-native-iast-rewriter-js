/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
/* eslint-disable no-unused-expressions */

const path = require('path')
const { expect } = require('chai')
const { rewriteWithOpts } = require('./util')

const FILE_PATH = path.join(process.cwd(), 'index.spec.js')

describe('hardcoded literals', () => {
  it('does double quoted literals found', () => {
    const js = 'const secret = "this_is_a_secret";'
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.file).to.be.eq(FILE_PATH)
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: 'secret', line: 1, column: 16 }] }
    ])
  })

  it('does not found literals if disabled by conf', () => {
    const js = 'const secret = "this_is_a_secret";'
    const result = rewriteWithOpts(js, {
      hardcodedSecret: false
    })

    expect(result.literalsResult).to.undefined
  })

  it('does return single quoted literals found', () => {
    const js = "const secret = 'this_is_a_secret';"
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.file).to.be.eq(FILE_PATH)
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: 'secret', line: 1, column: 16 }] }
    ])
  })

  it('does return single quoted literals found with correct line', () => {
    const js = `
    /*
     comment
    */

    const secret = 'this_is_a_secret';`
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.file).to.be.eq(FILE_PATH)
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: 'secret', line: 6, column: 20 }] }
    ])
  })

  it('does return multiple literals found', () => {
    const js = "const secret1 = 'this_is_a_secret'; const secret2 = 'another_secret'"
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.literals).to.deep.include({
      value: 'this_is_a_secret',
      locations: [{ ident: 'secret1', line: 1, column: 17 }]
    })
    expect(result.literalsResult.literals).to.deep.include({
      value: 'another_secret',
      locations: [{ ident: 'secret2', line: 1, column: 53 }]
    })
  })

  it('does return literals found inside a block', () => {
    const js = "function auth() { const secret = 'this_is_a_secret'; }"
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: 'secret', line: 1, column: 34 }] }
    ])
  })

  it('does return parameter literals in a call', () => {
    const js = "function login() { return auth('this_is_a_secret'); }"
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: undefined, line: 1, column: 32 }] }
    ])
  })

  it('does return literals in an object definition with ident as key', () => {
    const js = "const TOKENS = { secret: 'this_is_a_secret' }"
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: 'secret', line: 1, column: 26 }] }
    ])
  })

  it('does return literals in an object definition without ident', () => {
    const js = "const TOKENS = { [secret]: 'this_is_a_secret' }"
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.literals).to.deep.eq([
      { value: 'this_is_a_secret', locations: [{ ident: undefined, line: 1, column: 28 }] }
    ])
  })

  it('does not return literals with less or eq than 8 chars length', () => {
    const js = 'const secret = "12345678";'
    const result = rewriteWithOpts(js)

    expect(result.literalsResult).to.not.undefined
    expect(result.literalsResult.literals).to.deep.eq([])
  })
})
