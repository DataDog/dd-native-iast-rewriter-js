import * as fs from 'fs'
import * as path from 'path'

const fetchClassifiers = () => {
    const source = path.join(process.cwd(), 'npm')
    return fs.promises.readdir(source)
}

const copyFile = async (src, dest) => {
    return fs.promises.access(src, fs.constants.F_OK).then(
        () => fs.promises.copyFile(src, dest),
        (_) => Promise.resolve(),
    )
}

const copyArtifact = (classifier) => {
    const filename = `iast-rewriter.${classifier}.node`
    const sourceNode = path.join(process.cwd(), filename)
    const destNode = path.join(`${process.cwd()}`, 'npm', classifier, filename)
    return copyFile(sourceNode, destNode)
}

const prepare = () => {
    return fetchClassifiers().then((classifiers) => Promise.all(classifiers.map(copyArtifact)))
}

await prepare()
