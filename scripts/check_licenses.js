'use strict'

const fs = require('fs')
const path = require('path')
const pkg = require('../package.json')

const filePath = path.join(__dirname, '..', '/LICENSE-3rdparty.csv')
const thirdPartyCsv = fs.readFileSync(filePath).toString()
const csvLines = thirdPartyCsv.split('\n')
const deps = new Set(Object.keys(pkg.dependencies || {}))
const devDeps = new Set(Object.keys(pkg.devDependencies || {}))
const cargoDeps = readCargoDeps()

cargoDeps.dependencies.forEach((name) => {
  deps.add(name)
})

cargoDeps.devDependencies.forEach((name) => {
  devDeps.add(name)
})

let index = 0

const licenses = {
  require: new Set(),
  dev: new Set(),
  file: new Set()
}

for (let i = 0; i < csvLines.length; i++) {
  const line = csvLines[i]
  if (index !== 0) {
    const columns = line.split(',')
    const type = columns[0]
    const license = columns[1]
    if (type && license) {
      licenses[type].add(license)
    }
  }

  index++
}
const requiresOk = checkLicenses(deps, 'require')
const devOk = checkLicenses(devDeps, 'dev')
if (!requiresOk || !devOk) {
  process.exit(1)
}

function checkLicenses (typeDeps, type) {
  /* eslint-disable no-console */

  const missing = []
  const extraneous = []

  for (const dep of typeDeps) {
    if (!licenses[type].has(dep)) {
      missing.push(dep)
    }
  }

  for (const dep of licenses[type]) {
    if (!typeDeps.has(dep)) {
      extraneous.push(dep)
    }
  }

  if (missing.length) {
    console.log(`Missing ${type} 3rd-party license for ${missing.join(', ')}.`)
  }

  if (extraneous.length) {
    console.log(`Extraneous ${type} 3rd-party license for ${extraneous.join(', ')}.`)
  }

  return missing.length === 0 && extraneous.length === 0
}

function readCargoDeps () {
  const STATE_NONE = null
  const STATE_DEP = 0
  const STATE_DEV_DEP = 1
  const cargoToml = fs.readFileSync(path.join(__dirname, '..', 'Cargo.toml')).toString()
  const cargoLines = cargoToml.split('\n').map((x) => x.trim())
  let currentState = STATE_NONE // options = 'dep' | 'devDep'
  const dependencies = []
  const devDependencies = []
  cargoLines.forEach((line) => {
    if (line === '[dependencies]') {
      currentState = STATE_DEP
    } else if (line === '[dev-dependencies]' || line === '[build-dependencies]') {
      currentState = STATE_DEV_DEP
    } else if (line.startsWith('[')) {
      currentState = STATE_NONE
    } else if (currentState !== STATE_NONE) {
      if (line.includes('=')) {
        const dependencyName = line.split('=')[0].trim()
        currentState === STATE_DEP ? dependencies.push(dependencyName) : devDependencies.push(dependencyName)
      }
    }
  })
  return { dependencies, devDependencies }
}
