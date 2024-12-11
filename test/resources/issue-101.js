'use strict'

function names(arg) {
  const flag = arg
  const addPrefix = (value) => (flag ? `"${value}"` : `"my_prefix.${value}"`)
  const result = `
      ${addPrefix('NAME_0')}
      ${addPrefix('NAME_1')}
      ${addPrefix('NAME_2')}
      ${addPrefix('NAME_3')}
    `
  return result
}

function namesBlock(arg) {
  const flag = arg
  const addPrefix = (value) => {
    return flag ? `"${value}"` : `"my_prefix.${value}"`
  }
  const result = `
      ${addPrefix('NAME_0')}
      ${addPrefix('NAME_1')}
      ${addPrefix('NAME_2')}
      ${addPrefix('NAME_3')}
    `
  return result
}

function namesNested(arg) {
  const flag = arg
  const addSufix = (value) => `(${!value}_suffix`
  const addPrefix = (value) => (flag ? `"${value}"` : `"my_prefix.${addSufix(value)}"`)
  const result = `
      ${addPrefix('NAME_0')}
      ${addPrefix('NAME_1')}
      ${addPrefix('NAME_2')}
      ${addPrefix('NAME_3')}
    `
  return result
}

function paren(arg) {
  const flag = arg
  let a, b, c
  const addPrefix = (value) => (flag ? `"${value}"` : `"my_prefix.${value}"`)
  const result =
    ((a = 'NAME_0'),
    (b = 'NAME_1'),
    (c = 'NAME_2'),
    `
    ${addPrefix(a)}
    ${addPrefix(b)}
    ${addPrefix(c)}
  `)
  return result
}

module.exports = {
  names,
  namesBlock,
  namesNested,
  paren,
}
