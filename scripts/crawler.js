/* eslint-disable no-multi-str */
/* eslint-disable no-console */

const fs = require('fs')
const path = require('path')
const os = require('os')
const { exit } = require('process')
const { Rewriter } = require('../index')

const INCLUDED_FILES = /(.*)\.m?js$/
const ENCODING = 'utf8'
const USE_STRICT = /^"use strict";$/gm
const REWRITTEN_FILE_TOKEN_NAME = '___rewritten'
const REWRITTEN_FILE_BACKUP_NAME = REWRITTEN_FILE_TOKEN_NAME + '_original'
const DD_IAST_GLOBAL_METHODS_FILE_ENV = 'DD_IAST_GLOBAL_METHODS_FILE'
const V8_NATIVE_CALL_REGEX = /%(\w+\(\S*?|\s*\))/gm
const V8_NATIVE_CALL_REPLACEMENT_PREFIX = '__v8_native_remainder'
const V8_NATIVE_CALL_REPLACEMENT_REGEX = /__v8_native_remainder(\w+\(\S*?|\s*\))/gm
const V8_NATIVE_CALL_FLAGS_COMMENT_REGEX = /\/\/\s*Flags:.*(--allow-natives-syntax)+/gm

const GLOBAL_METHODS =
  "(function(globals){globals._ddiast = globals._ddiast || {plusOperator(res) {return res;}};}((1,eval)('this')));"

const DEFAULT_OPTIONS = {
  restore: false,
  rootPath: null,
  filePattern: '.*',
  override: false,
  globals: true,
  globalsFile: null,
  rewrite: true,
  modules: false,
  natives: true,
  help: false
}

const green = console.log.bind(this, '\x1b[32m%s\x1b[0m')
const red = console.log.bind(this, '\x1b[31m%s\x1b[0m')
const blue = console.log.bind(this, '\x1b[34m%s\x1b[0m')
const cyan = console.log.bind(this, '\x1b[35m%s\x1b[0m')

const rewriter = new Rewriter({ comments: true })

const crawl = (dirPath, options, visitor) => {
  blue(dirPath)
  const files = fs.readdirSync(dirPath)

  files.forEach((file) => {
    const filePath = path.join(dirPath, file)
    if (!fs.existsSync(filePath)) {
      return
    }
    if (fs.statSync(filePath).isDirectory()) {
      if (!options.modules && file === 'node_modules') {
        return
      }
      crawl(filePath, options, visitor)
    } else {
      if (options.restore) {
        restore(dirPath, file, options)
      } else if (file.match(options.filePattern)) {
        visit(dirPath, file, options, visitor)
      }
    }
  })
}

const restore = (dirPath, file, options) => {
  const backupFile = path.join(dirPath, file + REWRITTEN_FILE_BACKUP_NAME)
  const rewrittenFile = path.join(dirPath, rewrittenName(path.join(dirPath, file)))
  if (fs.existsSync(backupFile)) {
    const filePath = path.join(dirPath, file)
    fs.unlinkSync(filePath)
    fs.renameSync(backupFile, filePath)
    green(`Restored ${file} ${dirPath}`)
  } else if (fs.existsSync(rewrittenFile)) {
    fs.unlinkSync(rewrittenFile)
    green(`Deleted ${rewrittenFile}`)
  }
}

const rewrittenName = (filePath) =>
  path.basename(filePath, path.extname(filePath)) + '.' + REWRITTEN_FILE_TOKEN_NAME + path.extname(filePath)

const visit = (dirPath, file, options, visitor) => {
  if (file.match(INCLUDED_FILES) && file.indexOf(REWRITTEN_FILE_TOKEN_NAME) === -1) {
    try {
      let filePath = path.join(dirPath, '/', file)
      let readFilePath = filePath

      // if backup file exists take its content to avoid rewriting a rewritten file
      if (fs.existsSync(filePath + REWRITTEN_FILE_BACKUP_NAME)) {
        readFilePath = filePath + REWRITTEN_FILE_BACKUP_NAME
      }

      const fileContentOriginal = fs.readFileSync(readFilePath, ENCODING)
      const fileContent = visitor.visit(fileContentOriginal, file, filePath)
      if (!fileContent) {
        return
      }

      if (options.override) {
        fs.writeFileSync(filePath + REWRITTEN_FILE_BACKUP_NAME, fileContentOriginal)
      } else {
        filePath = path.join(path.dirname(filePath), rewrittenName(filePath))
      }
      fs.writeFileSync(filePath, fileContent)
    } catch (e) {
      red(e)
    }
  }
}

const parseOptions = (args) => {
  const options = DEFAULT_OPTIONS
  for (let i = 2; i < args.length; i++) {
    const arg = args[i]
    const dashes = arg.indexOf('--')
    if (dashes === 0) {
      let key = arg.substring(dashes + 2)
      const value = key.indexOf('no-') === -1
      key = value ? key : key.substring('no-'.length)
      options[key] = value
    } else {
      if (!options.rootPath) {
        options.rootPath = arg
      } else {
        options.filePattern = arg
      }
    }
  }

  if (process.env[DD_IAST_GLOBAL_METHODS_FILE_ENV]) {
    options.globalsFile = process.env[DD_IAST_GLOBAL_METHODS_FILE_ENV]
  }

  return options
}

const showHelp = () => {
  console.log('Usage: node crawler.js path/to/crawl [file_name_pattern]', os.EOL)

  console.log('Options:')
  console.log('  --override                          Original file is overrided with rewritten modifications')
  console.log(
    '  --no-override                       Default value if not specified. Original file is not modified \
and rewritten file is saved with a suffix next to original file'
  )
  console.log('  --restore                           Restores all js files to their original state')
  console.log('  --no-globals                        Do not inject default global._ddiast.* methods')
  console.log('  --no-rewrite                        Search for files but do not rewrite')
  console.log('  --no-natives                        Disable v8 native calls substitution', os.EOL)

  console.log('Environment variables:')
  console.log(
    '  DD_IAST_GLOBAL_METHODS_FILE         Path to the file containing methods to inject in the \
rewritten file header'
  )
}

const options = parseOptions(process.argv)
if (options.help) {
  showHelp()
  exit()
}
if (!options.rootPath) {
  red('Error. Missing path!', os.EOL)
  showHelp()
  exit()
}
if (options.filePattern) {
  try {
    options.filePattern = new RegExp(options.filePattern)
  } catch (e) {
    red(e)
    exit()
  }
}
crawl(options.rootPath, options, {
  visit (code, fileName, path) {
    if (options.rewrite) {
      try {
        if (options.natives) {
          code = this.replaceNativeV8Calls(code, fileName)
        }
        let rewrited = rewriter.rewrite(code, path)

        green(`     -> ${fileName}`)

        if (options.natives) {
          rewrited = this.replaceNativeV8Calls(rewrited, fileName, true)
        }
        return this.addGlobalMethods(rewrited, options)
      } catch (e) {
        red(`     -> ${fileName}: ${e}`)
      }
    } else {
      cyan(`     -> ${fileName}`)
    }
  },
  replaceNativeV8Calls (code, fileName, restore) {
    if (!code.match(V8_NATIVE_CALL_FLAGS_COMMENT_REGEX)) {
      return code
    }

    const regex = restore ? V8_NATIVE_CALL_REPLACEMENT_REGEX : V8_NATIVE_CALL_REGEX
    const replacement = restore ? '%' : V8_NATIVE_CALL_REPLACEMENT_PREFIX
    code = code.replace(regex, replacement + '$1')
    return code
  },
  addGlobalMethods (code, options) {
    let globalMethods = GLOBAL_METHODS
    if (options.globalsFile) {
      try {
        globalMethods = fs.readFileSync(options.globalsFile, ENCODING)
      } catch (e) {
        red(e)
      }
    }
    if (options.globals) {
      if (code.match(USE_STRICT)) {
        return code.replace(USE_STRICT, '"use strict";' + os.EOL + globalMethods + os.EOL)
      } else {
        return globalMethods + os.EOL + code
      }
    }

    return code
  }
})
