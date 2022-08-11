import * as fs from 'fs'
import path from 'path'

import { Node, parse, traverse } from '@babel/core'
import { Comment, File } from '@babel/types'
import { expect } from '@jest/globals'
import { BasicSourceMapConsumer, SourceMapConsumer } from 'source-map'
import * as tmp from 'tmp'

import Dict = NodeJS.Dict

const { Rewriter } = require(process.env['NPM_REWRITER'] ? '@datadog/native-iast-rewriter' : '../index')

enum ExpectMode {
    STRING,
    AST,
}

interface Position {
    line: number
    column: number
}

const findPosition = (text: string, to_search: string): Position => {
    let result: Position = { line: -1, column: -1 }
    text.split('\n').find((content, lineNumber) => {
        const column = content.indexOf(to_search)
        if (column > -1) {
            result = { line: lineNumber + 1, column: column }
            return true
        }
        return false
    })
    return result
}

const temporalFile = (): Promise<string> => {
    return new Promise((resolve, reject) => {
        tmp.file((err, file) => {
            if (err) {
                reject(err)
            } else {
                resolve(file)
            }
        })
    })
}

interface RewriteOpts {
    file?: string
    sourceMap?: string
    rewriter?: typeof Rewriter
}

const rewriteAst = (code: string, opts: RewriteOpts = {}): string => {
    const rewriter = opts.rewriter ?? new Rewriter()
    const file = opts.file ?? path.join(process.cwd(), 'index.js')
    const sourceMap = opts.sourceMap
    return rewriter.rewrite(code, file, sourceMap)
}

const remove = (item: Node | Comment) => {
    if (item) {
        item.start = null
        item.end = null
        item.loc = null
        if (Object.prototype.hasOwnProperty.call(item, 'leadingComments')) {
            const node = item as Node
            if (node.leadingComments) {
                node.leadingComments.forEach(remove)
            }
            if (node.trailingComments) {
                node.trailingComments.forEach(remove)
            }
            if (node.innerComments) {
                node.innerComments.forEach(remove)
            }
        }
        if (Object.prototype.hasOwnProperty.call(item, 'comments')) {
            const file = item as File
            if (file.comments) {
                file.comments.forEach(remove)
            }
        }
    }
}
const removeLoc = (ast: any): any => {
    traverse(ast, {
        enter(path) {
            remove(path.node)
            remove(path.parent)
        },
    })
    return ast
}

const expectAst = (received: string, expected: string, ast: ExpectMode = ExpectMode.STRING) => {
    if (ast === ExpectMode.STRING) {
        expect(received.trim()).toBe(expected.trim())
    } else {
        const receivedAst = removeLoc(parse(received)!)
        const expectedAst = removeLoc(parse(expected)!)
        expect(receivedAst).toStrictEqual(expectedAst)
    }
}

describe('binary expression', () => {
    it('sub', () => {
        const js = 'const result = a - " hey!";'
        expectAst(rewriteAst(js), js)
    })

    it('add', () => {
        const js = 'const result = a + " hey!";'
        expectAst(rewriteAst(js), 'const result = global._ddiast.twoItemsPlusOperator(a, " hey!");')
    })

    it('does not modify parameters of other functions when literals', () => {
        const js = 'const result = 1 + otherMethod(2);'
        expectAst(rewriteAst(js), 'const result = global._ddiast.twoItemsPlusOperator(1, otherMethod(2));')
    })

    it('does modify parameters of other functions when they arent literasls', () => {
        const js = "const a = 1 + 2 + otherMethod(3, 4, 5 + 8 + varname + 10 + 'test') + 6 + 7;"
        expectAst(
            rewriteAst(js),
            "const a = global._ddiast.threeItemsPlusOperator(1 + 2, otherMethod(3, 4, global._ddiast.threeItemsPlusOperator(5 + 8, varname, 10 + 'test')), 6 + 7);",
        )
    })

    it.each([
        'const result = "a" + "b";',
        'const result = "a" + "b" + "c";',
        'const result = "a" + "b" + "c" + "d";',
        'const result = "a" + "b" + "c" + "d" + "e";',
        'const result = "a" + "b" + "c" + "d" + "e" + "f";',
    ])('does not change sum of literals', (js) => {
        expectAst(rewriteAst(js), js)
    })

    it.each([
        ['const result = a + b;', 'const result = global._ddiast.twoItemsPlusOperator(a, b);'],
        ['const result = a + b + c;', 'const result = global._ddiast.threeItemsPlusOperator(a, b, c);'],
        ['const result = a + b + c + d;', 'const result = global._ddiast.fourItemsPlusOperator(a, b, c, d);'],
        ['const result = a + b + c + d + e;', 'const result = global._ddiast.fiveItemsPlusOperator(a, b, c, d, e);'],
        ['const result = a + b + c + d + e + f;', 'const result = global._ddiast.anyPlusOperator(a, b, c, d, e, f);'],
    ])('does change "%s" to "%s" when using identifiers', (input, expected) => {
        expectAst(rewriteAst(input), expected)
    })

    it.each([
        ['const result = a() + b();', 'const result = global._ddiast.twoItemsPlusOperator(a(), b());'],
        ['const result = a() + b() + c();', 'const result = global._ddiast.threeItemsPlusOperator(a(), b(), c());'],
        [
            'const result = a() + b() + c() + d();',
            'const result = global._ddiast.fourItemsPlusOperator(a(), b(), c(), d());',
        ],
        [
            'const result = a() + b() + c() + d() + e();',
            'const result = global._ddiast.fiveItemsPlusOperator(a(), b(), c(), d(), e());',
        ],
        [
            'const result = a() + b() + c() + d() + e() + f();',
            'const result = global._ddiast.anyPlusOperator(a(), b(), c(), d(), e(), f());',
        ],
    ])('does change "%s" to "%s" when using identifiers', (input, expected) => {
        expectAst(rewriteAst(input), expected)
    })

    it.each([
        //
        //Literals expanding from the beginning
        //
        ['const result = "a" + b;', 'const result = global._ddiast.twoItemsPlusOperator("a", b);'],
        ['const result = "a" + b + c;', 'const result = global._ddiast.threeItemsPlusOperator("a", b, c);'],
        ['const result = "a" + b + c + d;', 'const result = global._ddiast.fourItemsPlusOperator("a", b, c, d);'],
        [
            'const result = "a" + b + c + d + e;',
            'const result = global._ddiast.fiveItemsPlusOperator("a", b, c, d, e);',
        ],
        [
            'const result = "a" + b + c + d + e + f;',
            'const result = global._ddiast.anyPlusOperator("a", b, c, d, e, f);',
        ],
        ['const result = "a" + "b" + c;', 'const result = global._ddiast.twoItemsPlusOperator("a" + "b", c);'],
        [
            'const result = "a" + "b" + "c" + d;',
            'const result = global._ddiast.twoItemsPlusOperator("a" + "b" + "c", d);',
        ],
        [
            'const result = "a" + "b" + "c" + "d" + e;',
            'const result = global._ddiast.twoItemsPlusOperator("a" + "b" + "c" + "d", e);',
        ],
        [
            'const result = "a" + "b" + "c" + "d" + "e" + f;',
            'const result = global._ddiast.twoItemsPlusOperator("a" + "b" + "c" + "d" + "e", f);',
        ],
        //
        //Literals expanding from the end
        //
        ['const result = a + "b";', 'const result = global._ddiast.twoItemsPlusOperator(a, "b");'],
        ['const result = a + b + "c";', 'const result = global._ddiast.threeItemsPlusOperator(a, b, "c");'],
        ['const result = a + b + c + "d";', 'const result = global._ddiast.fourItemsPlusOperator(a, b, c, "d");'],
        [
            'const result = a + b + c + d + "e";',
            'const result = global._ddiast.fiveItemsPlusOperator(a, b, c, d, "e");',
        ],
        [
            'const result = a + b + c + d + e + "f";',
            'const result = global._ddiast.anyPlusOperator(a, b, c, d, e, "f");',
        ],
        [
            'const result = a + b + c + d + e + "f" + "g";',
            'const result = global._ddiast.anyPlusOperator(a, b, c, d, e, "f" + "g");',
        ],
        [
            'const result = a + b + c + d + e + "f" + g;',
            'const result = global._ddiast.anyPlusOperator(a, b, c, d, e, "f", g);',
        ],
        ['const result = a + "b" + "c";', 'const result = global._ddiast.twoItemsPlusOperator(a, "b" + "c");'],
        ['const result = a + b + "c" + "d";', 'const result = global._ddiast.threeItemsPlusOperator(a, b, "c" + "d");'],
        [
            'const result = a + "b" + "c" + "d";',
            'const result = global._ddiast.twoItemsPlusOperator(a, "b" + "c" + "d");',
        ],
        [
            'const result = a + b + c + d + "e";',
            'const result = global._ddiast.fiveItemsPlusOperator(a, b, c, d, "e");',
        ],
        [
            'const result = a + b + c + "d" + "e";',
            'const result = global._ddiast.fourItemsPlusOperator(a, b, c, "d" + "e");',
        ],
        [
            'const result = a + b + "c" + "d" + "e";',
            'const result = global._ddiast.threeItemsPlusOperator(a, b, "c" + "d" + "e");',
        ],
        [
            'const result = a + "b" + "c" + "d" + "e";',
            'const result = global._ddiast.twoItemsPlusOperator(a, "b" + "c" + "d" + "e");',
        ],
        //
        //Literals expanding Middle positions
        //
        ['const result = a + "b" + c;', 'const result = global._ddiast.threeItemsPlusOperator(a, "b", c);'],
        ['const result = a + "b" + c + d;', 'const result = global._ddiast.fourItemsPlusOperator(a, "b", c, d);'],
        ['const result = a + b + "c" + d;', 'const result = global._ddiast.fourItemsPlusOperator(a, b, "c", d);'],
        ['const result = a + "b" + "c" + d;', 'const result = global._ddiast.threeItemsPlusOperator(a, "b" + "c", d);'],
        //
        //Mix combinations
        //
        [
            'const result = a + "b" + "c" + d + e;',
            'const result = global._ddiast.fourItemsPlusOperator(a, "b" + "c", d, e);',
        ],
        [
            'const result = "a" + "b" + c + "d" + "e";',
            'const result = global._ddiast.threeItemsPlusOperator("a" + "b", c, "d" + "e");',
        ],
        [
            'const result = a + b + "c" + d + e;',
            'const result = global._ddiast.fiveItemsPlusOperator(a, b, "c", d, e);',
        ],
        [
            'const result = a + b + "c" + d + e + "f";',
            'const result = global._ddiast.anyPlusOperator(a, b, "c", d, e, "f");',
        ],
        [
            'const result = a + b + c() + d() + "e" + "f";',
            'const result = global._ddiast.fiveItemsPlusOperator(a, b, c(), d(), "e" + "f");',
        ],
        ['const result = a + b * c;', 'const result = global._ddiast.twoItemsPlusOperator(a, b * c);'],
        ['const result = a * b + c;', 'const result = global._ddiast.twoItemsPlusOperator(a * b, c);'],
        [
            'const result = a * b + c * "d" + f;',
            'const result = global._ddiast.threeItemsPlusOperator(a * b, c * "d", f);',
        ],
        //Assignations
        ['a += b;', 'a = global._ddiast.twoItemsPlusOperator(a, b);'],
        ['a += b + c;', 'a = global._ddiast.twoItemsPlusOperator(a, global._ddiast.twoItemsPlusOperator(b, c));'],
        [
            'a += b + c + d;',
            'a = global._ddiast.twoItemsPlusOperator(a, global._ddiast.threeItemsPlusOperator(b, c, d));',
        ],
        [
            'a += b + c + d + e;',
            'a = global._ddiast.twoItemsPlusOperator(a, global._ddiast.fourItemsPlusOperator(b, c, d, e));',
        ],
        [
            'a += b + c + d + e + f;',
            'a = global._ddiast.twoItemsPlusOperator(a, global._ddiast.fiveItemsPlusOperator(b, c, d, e, f));',
        ],
        [
            'a += b + c + d + e + f + g;',
            'a = global._ddiast.twoItemsPlusOperator(a, global._ddiast.anyPlusOperator(b, c, d, e, f, g));',
        ],
        ['a += "b";', 'a = global._ddiast.twoItemsPlusOperator(a, "b");'],
        ['a += "b" + "c";', 'a = global._ddiast.twoItemsPlusOperator(a, "b" + "c");'],
        ['a += "b" + "c" + "d";', 'a = global._ddiast.twoItemsPlusOperator(a, "b" + "c" + "d");'],
        ['a += "b" + "c" + "d" + "e";', 'a = global._ddiast.twoItemsPlusOperator(a, "b" + "c" + "d" + "e");'],
        [
            'a += "b" + "c" + "d" + "e" + "f";',
            'a = global._ddiast.twoItemsPlusOperator(a, "b" + "c" + "d" + "e" + "f");',
        ],
    ])('does change "%s" to "%s" when using mix of literals and identifiers', (input, expected) => {
        expectAst(rewriteAst(input), expected)
    })
})

describe('template literal', () => {
    it('empty', () => {
        const js = 'const result = `Hello World!`;'
        expectAst(rewriteAst(js), 'const result = `Hello World!`;')
    })

    it('literal', () => {
        const js = 'const result = `Hello${" "}World!`;'
        expectAst(rewriteAst(js), js)
    })

    it('middle', () => {
        const js = 'const result = `Hello${a}World!`;'
        expectAst(rewriteAst(js), 'const result = global._ddiast.templateLiteralOperator(`Hello`, a, `World!`);')
    })

    it('start', () => {
        const js = 'const result = `${a}Hello World!`;'
        expectAst(rewriteAst(js), 'const result = global._ddiast.templateLiteralOperator(a, `Hello World!`);')
    })

    it('multiline string', () => {
        const js = `router.get('/xss', function(req, res) {
  res.header('content-type', 'text/html');
  res.send(\`<html lang="en">
    <body>
        <h1>XSS vulnerability</h1>
        <p>Received param: \${req.query.param}</p>
    </body>
</body>
</html>\`);
});`
        const expected = `router.get('/xss', function(req, res) {
    res.header('content-type', 'text/html');
    res.send(global._ddiast.templateLiteralOperator(\`<html lang="en">
    <body>
        <h1>XSS vulnerability</h1>
        <p>Received param: \`, req.query.param, \`</p>
    </body>
</body>
</html>\`));
});`
        expectAst(rewriteAst(js), expected)
    })

    it('end', () => {
        const js = 'const result = `Hello World!${a}`;'
        expectAst(rewriteAst(js), 'const result = global._ddiast.templateLiteralOperator(`Hello World!`, a);')
    })
})

describe('tagged template', () => {
    it('[tagged template]', () => {
        const js = 'const result = tagged`Hello ${"World!"}`;'
        expectAst(rewriteAst(js), js)
    })
})

describe('assign', () => {
    it('literals', () => {
        const js = 'a += "Hello World!";'
        expectAst(rewriteAst(js), 'a = global._ddiast.twoItemsPlusOperator(a, "Hello World!");')
    })

    it('variable', () => {
        const js = 'a += b;'
        expectAst(rewriteAst(js), 'a = global._ddiast.twoItemsPlusOperator(a, b);')
    })
})

const expectSourceMapKeepsTrackOfMovement = (rewrittenCode: RewrittenCode, inputJs: string, to_search: string) => {
    const output: Position = findPosition(rewrittenCode.code, to_search)
    const input: Position = findPosition(inputJs, to_search)
    expect(
        rewrittenCode.sourceMapConsumer.originalPositionFor({
            line: output.line,
            column: output.column,
        }),
    ).toMatchObject({
        line: input.line,
        column: input.column,
    })
}

interface RewrittenCode {
    code: string
    sourceMapConsumer: BasicSourceMapConsumer
}

const getRewrittenCode = async (inputJs: string, options: Dict<any> = {}): Promise<RewrittenCode> => {
    let code = ''
    const sourceMap = await temporalFile().then((sourceMap) => {
        options['sourceMap'] = sourceMap
        code = rewriteAst(inputJs, options)
        return JSON.parse(fs.readFileSync(sourceMap, 'utf-8'))
    })
    return { code, sourceMapConsumer: await new SourceMapConsumer(sourceMap) }
}

describe('source maps', () => {
    it('with source map chain', async () => {
        const sourceMap = await temporalFile().then((sourceMap) => {
            const file = path.join(process.cwd(), 'samples', 'login.js')
            const source = fs.readFileSync(file, 'utf-8')
            const rewritten = rewriteAst(source, { file: file, sourceMap: sourceMap })
            expect(rewritten).toContain('global._ddiast.templateLiteralOperator(`SELECT')
            return JSON.parse(fs.readFileSync(sourceMap, 'utf-8'))
        })
        const source = await new SourceMapConsumer(sourceMap)
        const position = source.originalPositionFor({
            line: 35,
            column: 25,
        })
        expect(position).toMatchObject({
            line: 31,
            source: '../../routes/login.ts',
        })
    })
    it('Our plus item operators inline parameters', async () => {
        const var01 = 'someVar'
        const func01 = 'otherMethod'
        const numberLiteral = 123
        const textLiteral = '"test"'
        const identifier = 'someIdentifier'
        const firstLiteral = 567
        const inputJs = `const ${var01} = 1;
const a = ${firstLiteral} + ${numberLiteral}
+ ${func01}(3, 4,
5 + 8 + 9
+ 10 + ${textLiteral})
+ 6 + ${identifier};`
        const rewrittenCode = await getRewrittenCode(inputJs)

        expectSourceMapKeepsTrackOfMovement(rewrittenCode, inputJs, var01)
        expectSourceMapKeepsTrackOfMovement(rewrittenCode, inputJs, func01)
        expectSourceMapKeepsTrackOfMovement(rewrittenCode, inputJs, numberLiteral.toString())
        expectSourceMapKeepsTrackOfMovement(rewrittenCode, inputJs, textLiteral)
        expectSourceMapKeepsTrackOfMovement(rewrittenCode, inputJs, identifier)

        // __hdiv_global__ is not in the input JS, so it needs to be asserted this way
        const expectedPosition = findPosition(inputJs, firstLiteral.toString())
        expect(
            rewrittenCode.sourceMapConsumer.originalPositionFor(findPosition(rewrittenCode.code, '_ddiast')),
        ).toMatchObject({
            line: expectedPosition.line,
            column: expectedPosition.column,
        })
    })
})

describe('bug', () => {
    it('juice shop: this -> void 0', () => {
        const js = `const extractFilename = (url)=>{
    let file = decodeURIComponent(url.substring(url.lastIndexOf('/') + 1));
    if (this.contains(file, '?')) {
        file = file.substring(0, file.indexOf('?'));
    }
    return file;
};`
        const expected = `const extractFilename = (url)=>{
    let file = decodeURIComponent(url.substring(global._ddiast.twoItemsPlusOperator(url.lastIndexOf('/'), 1)));
    if (this.contains(file, '?')) {
        file = file.substring(0, file.indexOf('?'));
    }
    return file;
};`
        expectAst(rewriteAst(js), expected, ExpectMode.AST)
    })
})
