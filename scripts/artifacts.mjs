/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
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
