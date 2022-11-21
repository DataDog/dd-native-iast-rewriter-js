/* eslint-disable no-console */
const fs = require('fs')
const path = require('path')
const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json')).toString())
const packageLockJson = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package-lock.json')).toString())
const cargoToml = fs.readFileSync(path.join(__dirname, '..', 'Cargo.toml')).toString()
const cargoLock = fs.readFileSync(path.join(__dirname, '..', 'Cargo.lock')).toString()

const definedVersion = packageJson.version
const packageLockVersion = packageLockJson.version
let isOk = true
if (definedVersion !== packageLockVersion) {
  isOk = false
}
const cargoTomlVersion = readVersionFromCargoToml(cargoToml)
if (definedVersion !== cargoTomlVersion) {
  isOk = false
}

const cargoLockVersion = readVersionFromCargoLock(cargoLock)
if (definedVersion !== cargoLockVersion) {
  isOk = false
}

if (!isOk) {
  console.error(`Version are not synchronized:
  package.json: ${definedVersion}
  package-lock.json: ${packageLockVersion}
  Cargo.toml: ${cargoTomlVersion}
  Cargo.lock: ${cargoLockVersion}
  `)
  process.exit(1)
}

function readVersionFromCargoToml (cargoTomlContent) {
  const lines = cargoTomlContent.split('\n')
  let inPackage = false
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    if (inPackage) {
      if (line.indexOf('version') === 0) {
        return JSON.parse(line.split('=')[1].trim())
      }
    }
    if (line === '[package]') {
      inPackage = true
    }
  }
}

function readVersionFromCargoLock (cargoLockContent) {
  const lines = cargoLockContent.split('\n')
  let inPackage = false
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    if (inPackage) {
      if (line.indexOf('version =') === 0) {
        return JSON.parse(line.split('=')[1].trim())
      }
    } else if (line === 'name = "native-iast-rewriter"') {
      inPackage = true
    }
  }
}
